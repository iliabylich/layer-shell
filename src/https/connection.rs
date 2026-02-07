use super::fsm::{FSM, Request, Response, Wants};
use crate::{
    liburing::IoUring,
    user_data::{ModuleId, UserData},
};
use anyhow::{Result, bail, ensure};
use libc::{AF_INET, SOCK_STREAM, addrinfo, freeaddrinfo, gai_strerror, sockaddr_in};
use rustls::pki_types::ServerName;
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr::null_mut,
};

pub(crate) struct HttpsConnection {
    fsm: FSM,

    addr: sockaddr_in,
    fd: i32,
    module_id: ModuleId,

    response: Option<Response>,
}

#[repr(u8)]
enum Op {
    Socket,
    Connect,
    Read,
    Write,
    Close,
}
const MAX_OP: u8 = Op::Close as u8;

impl TryFrom<u8> for Op {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        ensure!(value <= MAX_OP);
        unsafe { Ok(std::mem::transmute::<u8, Self>(value)) }
    }
}

impl HttpsConnection {
    pub(crate) fn get(hostname: &str, port: u16, path: &str, module_id: ModuleId) -> Result<Self> {
        let fsm = {
            let server_name = ServerName::try_from(hostname)?.to_owned();

            let mut request = Request::get(path);
            request.add_header("Host", hostname);
            request.add_header("Connection", "close");

            FSM::new(server_name, request)?
        };

        let mut addr = getaddrinfo(hostname)?;
        addr.sin_port = port.to_be();

        Ok(Self {
            fsm,
            addr,
            fd: -1,
            module_id,
            response: None,
        })
    }

    pub(crate) fn init(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_socket(AF_INET, SOCK_STREAM, 0, 0);
        sqe.set_user_data(UserData::new(self.module_id, Op::Socket as u8));
    }

    fn call_fsm(&mut self) -> Result<()> {
        match self.fsm.wants()? {
            Wants::Read(buf) => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_read(self.fd, buf.as_mut_ptr(), buf.len());
                sqe.set_user_data(UserData::new(self.module_id, Op::Read as u8));
            }
            Wants::Write(buf) => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_write(self.fd, buf.as_ptr(), buf.len());
                sqe.set_user_data(UserData::new(self.module_id, Op::Write as u8));
            }
            Wants::Done(response) => {
                assert!(self.response.is_none());
                self.response = Some(response);

                let mut sqe = IoUring::get_sqe();
                sqe.prep_close(self.fd);
                sqe.set_user_data(UserData::new(self.module_id, Op::Close as u8));
            }
        }
        Ok(())
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Result<Option<Response>> {
        match Op::try_from(op)? {
            Op::Socket => {
                let fd = res;
                ensure!(fd > 0);
                self.fd = fd;

                let mut sqe = IoUring::get_sqe();
                sqe.prep_connect(
                    self.fd,
                    (&self.addr as *const sockaddr_in).cast(),
                    std::mem::size_of::<sockaddr_in>() as u32,
                );
                sqe.set_user_data(UserData::new(self.module_id, Op::Connect as u8));

                Ok(None)
            }
            Op::Connect => {
                ensure!(res >= 0);

                self.call_fsm()?;

                Ok(None)
            }
            Op::Read => {
                let read = res;
                ensure!(read >= 0);
                let read: usize = read as usize;
                self.fsm.done_reading(read);

                self.call_fsm()?;
                Ok(None)
            }
            Op::Write => {
                let written = res;
                ensure!(written >= 0);
                let written = written as usize;
                self.fsm.done_writing(written);

                self.call_fsm()?;
                Ok(None)
            }
            Op::Close => {
                eprintln!("HttpsConnection closed");
                Ok(self.response.take())
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
