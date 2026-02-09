use super::fsm::{FSM, Request, Response, Wants};
use crate::{
    liburing::IoUring,
    macros::define_op,
    user_data::{ModuleId, UserData},
};
use anyhow::{Context as _, Result, ensure};
use libc::{AF_INET, SOCK_STREAM, sockaddr_in};
use rustls::pki_types::ServerName;

pub(crate) struct HttpsConnection {
    fsm: Option<FSM>,
    address: sockaddr_in,
    healthy: bool,

    fd: i32,
    module_id: ModuleId,

    response: Option<Response>,
}

define_op!("HttpsConnection", Socket, Connect, Read, Write, Close,);

impl HttpsConnection {
    pub(crate) fn new(hostname: &str, path: &str, module_id: ModuleId) -> Self {
        let mut healthy = true;

        let fsm = match ServerName::try_from(hostname) {
            Ok(server_name) => {
                let server_name = server_name.to_owned();

                let mut request = Request::get(path);
                request.add_header("Host", hostname);
                request.add_header("Connection", "close");

                match FSM::new(server_name, request) {
                    Ok(ok) => Some(ok),
                    Err(err) => {
                        log::error!("{err:?}");
                        healthy = false;
                        None
                    }
                }
            }
            Err(err) => {
                log::error!("{err:?}");
                healthy = false;
                None
            }
        };

        Self {
            fsm,
            address: unsafe { std::mem::zeroed() },
            healthy,

            fd: -1,
            module_id,
            response: None,
        }
    }

    fn schedule_socket(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_socket(AF_INET, SOCK_STREAM, 0, 0);
        sqe.set_user_data(UserData::new(self.module_id, Op::Socket));
    }

    fn schedule_connect(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_connect(
            self.fd,
            (&self.address as *const sockaddr_in).cast(),
            std::mem::size_of::<sockaddr_in>() as u32,
        );
        sqe.set_user_data(UserData::new(self.module_id, Op::Connect));
    }

    pub(crate) fn init(&mut self, mut address: sockaddr_in, port: u16) {
        address.sin_port = port.to_be();
        self.address = address;
        self.schedule_socket();
    }

    fn call_fsm(&mut self) {
        if !self.healthy {
            return;
        }

        let Some(wants) = unsafe { self.fsm.as_mut().unwrap_unchecked() }.wants() else {
            return;
        };
        match wants {
            Wants::Read(buf) => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_read(self.fd, buf.as_mut_ptr(), buf.len());
                sqe.set_user_data(UserData::new(self.module_id, Op::Read));
            }
            Wants::Write(buf) => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_write(self.fd, buf.as_ptr(), buf.len());
                sqe.set_user_data(UserData::new(self.module_id, Op::Write));
            }
            Wants::Done(response) => {
                assert!(self.response.is_none());
                self.response = Some(response);

                let mut sqe = IoUring::get_sqe();
                sqe.prep_close(self.fd);
                sqe.set_user_data(UserData::new(self.module_id, Op::Close));
            }
        }
    }

    fn try_process(&mut self, op: Op, res: i32) -> Result<Option<Response>> {
        match op {
            Op::Socket => {
                ensure!(res > 0);
                self.fd = res;
                self.schedule_connect();
                Ok(None)
            }
            Op::Connect => {
                ensure!(res >= 0);
                self.call_fsm();
                Ok(None)
            }
            Op::Read => {
                ensure!(res >= 0);
                let read: usize = res as usize;
                self.fsm
                    .as_mut()
                    .context("internal error: no FSM")?
                    .done_reading(read);

                self.call_fsm();
                Ok(None)
            }
            Op::Write => {
                ensure!(res >= 0);
                let written = res as usize;
                self.fsm
                    .as_mut()
                    .context("internal error: no FSM")?
                    .done_writing(written);

                self.call_fsm();
                Ok(None)
            }
            Op::Close => {
                log::warn!(target: "HTTPS", "HttpsConnection closed");
                Ok(self.response.take())
            }
        }
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Option<Response> {
        if !self.healthy {
            return None;
        }

        let op = Op::from(op);

        match self.try_process(op, res) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("HttpsConnection::{op:?}({res} {err:?}");
                self.healthy = false;
                None
            }
        }
    }
}
