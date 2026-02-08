use super::fsm::{FSM, Request, Response, Wants};
use crate::{
    liburing::IoUring,
    macros::define_op,
    user_data::{ModuleId, UserData},
};
use anyhow::{Result, bail};
use libc::{AF_INET, SOCK_STREAM, addrinfo, freeaddrinfo, gai_strerror, sockaddr_in};
use rustls::pki_types::ServerName;
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr::null_mut,
};

pub(crate) struct HttpsConnection {
    fsm: Option<FSM>,
    addr: sockaddr_in,
    healthy: bool,

    fd: i32,
    module_id: ModuleId,

    response: Option<Response>,
}

define_op!("HttpsConnection", Socket, Connect, Read, Write, Close,);

impl HttpsConnection {
    pub(crate) fn get(hostname: &str, port: u16, path: &str, module_id: ModuleId) -> Self {
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

        let addr = match getaddrinfo(hostname) {
            Ok(mut addr) => {
                addr.sin_port = port.to_be();
                addr
            }
            Err(err) => {
                log::error!("{err:?}");
                healthy = false;
                unsafe { std::mem::zeroed() }
            }
        };

        Self {
            fsm,
            addr,
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
            (&self.addr as *const sockaddr_in).cast(),
            std::mem::size_of::<sockaddr_in>() as u32,
        );
        sqe.set_user_data(UserData::new(self.module_id, Op::Connect));
    }

    pub(crate) fn init(&self) {
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

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Option<Response> {
        if !self.healthy {
            return None;
        }

        let op = Op::from(op);

        macro_rules! assert_or_unhealthy {
            ($cond:expr, $($arg:tt)*) => {
                if !$cond {
                    log::error!("HttpsConnection::{op:?}");
                    log::error!($($arg)*);
                    self.healthy = false;
                    return None;
                }
            };
        }

        match op {
            Op::Socket => {
                assert_or_unhealthy!(res > 0, "res is {res}");
                self.fd = res;
                self.schedule_connect();
                None
            }
            Op::Connect => {
                assert_or_unhealthy!(res >= 0, "res is {res}");
                self.call_fsm();
                None
            }
            Op::Read => {
                assert_or_unhealthy!(res >= 0, "res is {res}");
                let read: usize = res as usize;
                unsafe { self.fsm.as_mut().unwrap_unchecked() }.done_reading(read);

                self.call_fsm();
                None
            }
            Op::Write => {
                assert_or_unhealthy!(res >= 0, "res is {res}");
                let written = res as usize;
                unsafe { self.fsm.as_mut().unwrap_unchecked() }.done_writing(written);

                self.call_fsm();
                None
            }
            Op::Close => {
                log::warn!(target: "HTTPS", "HttpsConnection closed");
                self.response.take()
            }
        }
    }
}

fn getaddrinfo(hostname: &str) -> Result<sockaddr_in> {
    let node = CString::new(hostname)?;
    let mut hints = unsafe { MaybeUninit::<addrinfo>::zeroed().assume_init() };
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;

    let mut result = null_mut();

    let res = unsafe { libc::getaddrinfo(node.as_ptr(), null_mut(), &hints, &mut result) };
    if res != 0 {
        bail!("{}", unsafe { CStr::from_ptr(gai_strerror(res)) }.to_str()?)
    }

    let mut rp = result;
    while !rp.is_null() {
        if unsafe { *rp }.ai_family == AF_INET {
            let ip = unsafe { *(*rp).ai_addr.cast::<sockaddr_in>() };
            unsafe { freeaddrinfo(rp) }
            return Ok(ip);
        }

        rp = (unsafe { *rp }).ai_next;
    }
    unsafe { freeaddrinfo(rp) }

    bail!("failed to resolve DNS name: {hostname}")
}
