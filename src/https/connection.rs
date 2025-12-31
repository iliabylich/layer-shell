use super::fsm::{FSM, Request, Response, Wants};
use crate::liburing::{Cqe, IoUring, Pending};
use anyhow::{Result, bail, ensure};
use libc::{AF_INET, SOCK_STREAM, addrinfo, freeaddrinfo, gai_strerror, sockaddr_in};
use rustls::pki_types::ServerName;
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr::null_mut,
};

#[derive(Default)]
enum State {
    Initialized {
        addr: sockaddr_in,
    },
    Connecting {
        fd: i32,
        addr: sockaddr_in,
    },
    Connected {
        fd: i32,
    },
    Closing {
        fd: i32,
    },
    #[default]
    Closed,
}

pub(crate) struct HttpsConnection {
    fsm: FSM,
    state: State,

    socket_user_data: u64,
    connect_user_data: u64,
    read_user_data: u64,
    write_user_data: u64,
    close_user_data: u64,
}

impl HttpsConnection {
    pub(crate) fn get(
        hostname: &str,
        port: u16,
        path: &str,
        socket_user_data: u64,
        connect_user_data: u64,
        read_user_data: u64,
        write_user_data: u64,
        close_user_data: u64,
    ) -> Result<Self> {
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
            state: State::Initialized { addr },
            socket_user_data,
            connect_user_data,
            read_user_data,
            write_user_data,
            close_user_data,
        })
    }

    pub(crate) fn push_sqes(
        &mut self,
        ring: &mut IoUring,
        pending: &mut Pending,
    ) -> Result<(bool, Option<Response>)> {
        macro_rules! swallow_dupes {
            ($op:expr, $code:block) => {
                if pending.is($op) {
                    return Ok((false, None));
                } else {
                    $code
                    pending.set($op);
                    return Ok((true, None));
                }
            };
        }

        match &self.state {
            State::Initialized { .. } => {
                swallow_dupes!(self.socket_user_data, {
                    let mut sqe = ring.get_sqe()?;
                    sqe.prep_socket(AF_INET, SOCK_STREAM, 0, 0);
                    sqe.set_user_data(self.socket_user_data);
                })
            }
            State::Connecting { fd, addr, .. } => {
                swallow_dupes!(self.connect_user_data, {
                    let mut sqe = ring.get_sqe()?;
                    sqe.prep_connect(
                        *fd,
                        (addr as *const sockaddr_in).cast(),
                        std::mem::size_of::<sockaddr_in>() as u32,
                    );
                    sqe.set_user_data(self.connect_user_data);
                })
            }
            State::Connected { fd } => match self.fsm.wants()? {
                Wants::Read(buf) => {
                    swallow_dupes!(self.read_user_data, {
                        let mut sqe = ring.get_sqe()?;
                        sqe.prep_read(*fd, buf.as_mut_ptr(), buf.len());
                        sqe.set_user_data(self.read_user_data);
                    })
                }
                Wants::Write(buf) => {
                    swallow_dupes!(self.write_user_data, {
                        let mut sqe = ring.get_sqe()?;
                        sqe.prep_write(*fd, buf.as_ptr(), buf.len());
                        sqe.set_user_data(self.write_user_data);
                    })
                }
                Wants::Done(response) => {
                    self.state = State::Closing { fd: *fd };
                    Ok((false, Some(response)))
                }
            },
            State::Closing { fd } => {
                swallow_dupes!(self.close_user_data, {
                    let mut sqe = ring.get_sqe()?;
                    sqe.prep_close(*fd);
                    sqe.set_user_data(self.close_user_data);
                })
            }
            State::Closed => {
                panic!("closed");
            }
        }
    }

    fn take_state(&mut self) -> State {
        std::mem::take(&mut self.state)
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Result<()> {
        match cqe.user_data() {
            data if data == self.socket_user_data => {
                let fd = cqe.res();
                ensure!(fd > 0);

                let State::Initialized { addr } = self.take_state() else {
                    panic!("malformed state")
                };

                self.state = State::Connecting { fd, addr };
            }
            data if data == self.connect_user_data => {
                ensure!(cqe.res() >= 0);

                let State::Connecting { fd, .. } = self.take_state() else {
                    panic!("malformed state")
                };

                self.state = State::Connected { fd };
            }
            data if data == self.read_user_data => {
                let read = cqe.res();
                ensure!(read >= 0);
                let read = read as usize;

                self.fsm.done_reading(read);
            }
            data if data == self.write_user_data => {
                let written = cqe.res();
                ensure!(written >= 0);
                let written = written as usize;

                self.fsm.done_writing(written);
            }
            data if data == self.close_user_data => {
                println!("CLOSED!!!");
                self.state = State::Closed;
            }

            _ => {}
        }

        Ok(())
    }

    pub(crate) fn is_closed(&self) -> bool {
        matches!(self.state, State::Closed)
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
