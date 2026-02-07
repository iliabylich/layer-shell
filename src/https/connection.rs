use super::fsm::{FSM, Request, Response, Wants};
use crate::{
    liburing::IoUring,
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

#[repr(u8)]
#[derive(Debug)]
enum Op {
    Socket,
    Connect,
    Read,
    Write,
    Close,
}
const MAX_OP: u8 = Op::Close as u8;

impl From<u8> for Op {
    fn from(value: u8) -> Self {
        if value > MAX_OP {
            eprintln!("unsupported op in HttpsConnection: {value}");
            std::process::exit(1);
        }
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

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
                        eprintln!("{err:?}");
                        healthy = false;
                        None
                    }
                }
            }
            Err(err) => {
                eprintln!("{err:?}");
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
                eprintln!("{err:?}");
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

    pub(crate) fn init(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_socket(AF_INET, SOCK_STREAM, 0, 0);
        sqe.set_user_data(UserData::new(self.module_id, Op::Socket as u8));
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
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Option<Response> {
        if !self.healthy {
            return None;
        }

        let op = Op::from(op);

        macro_rules! crash {
            ($($arg:tt)*) => {{
                eprintln!($($arg)*);
                return None;
            }};
        }

        match op {
            Op::Socket => {
                let fd = res;
                if res <= 0 {
                    crash!("{op:?}: fd < 0: {fd}");
                }
                self.fd = fd;

                let mut sqe = IoUring::get_sqe();
                sqe.prep_connect(
                    self.fd,
                    (&self.addr as *const sockaddr_in).cast(),
                    std::mem::size_of::<sockaddr_in>() as u32,
                );
                sqe.set_user_data(UserData::new(self.module_id, Op::Connect as u8));

                None
            }
            Op::Connect => {
                if res < 0 {
                    crash!("{op:?}: res < 0: {res}")
                }

                self.call_fsm();

                None
            }
            Op::Read => {
                let read = res;
                if res < 0 {
                    crash!("{op:?}: res < 0: {res}")
                }
                let read: usize = read as usize;
                unsafe { self.fsm.as_mut().unwrap_unchecked() }.done_reading(read);

                self.call_fsm();
                None
            }
            Op::Write => {
                let written = res;
                if written < 0 {
                    crash!("{op:?}: written < 0: {written}")
                }
                let written = written as usize;
                unsafe { self.fsm.as_mut().unwrap_unchecked() }.done_writing(written);

                self.call_fsm();
                None
            }
            Op::Close => {
                eprintln!("HttpsConnection closed");
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
