use super::fsm::{FSM, Request, Response, Wants};
use crate::{liburing::IoUring, user_data::UserData};
use anyhow::{Result, bail, ensure};
use libc::{AF_INET, SOCK_STREAM, addrinfo, freeaddrinfo, gai_strerror, sockaddr_in};
use rustls::pki_types::ServerName;
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr::null_mut,
};

#[derive(Debug)]
enum State {
    CanAcquireSocket,
    AcquiringSocket,

    CanConnect,
    Connecting,

    CanReadWrite,
    Reading,
    Writing,

    CanClose,
    Closing,
    Closed,
}

pub(crate) struct HttpsConnection {
    fsm: FSM,

    addr: sockaddr_in,
    fd: i32,
    state: State,

    socket_user_data: UserData,
    connect_user_data: UserData,
    read_user_data: UserData,
    write_user_data: UserData,
    close_user_data: UserData,

    response: Option<Response>,
}

impl HttpsConnection {
    pub(crate) fn get(
        hostname: &str,
        port: u16,
        path: &str,
        socket_user_data: UserData,
        connect_user_data: UserData,
        read_user_data: UserData,
        write_user_data: UserData,
        close_user_data: UserData,
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
            addr,
            fd: -1,
            state: State::CanAcquireSocket,
            socket_user_data,
            connect_user_data,
            read_user_data,
            write_user_data,
            close_user_data,
            response: None,
        })
    }

    fn response_received(&mut self, response: Response) {
        assert!(self.response.is_none());
        self.response = Some(response);
        self.state = State::CanClose;
    }

    pub(crate) fn drain_once(&mut self, ring: &mut IoUring) -> Result<bool> {
        match &self.state {
            State::CanAcquireSocket { .. } => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_socket(AF_INET, SOCK_STREAM, 0, 0);
                sqe.set_user_data(self.socket_user_data.as_u64());

                self.state = State::AcquiringSocket;
                Ok(true)
            }
            State::AcquiringSocket => Ok(false),

            State::CanConnect => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_connect(
                    self.fd,
                    (&self.addr as *const sockaddr_in).cast(),
                    std::mem::size_of::<sockaddr_in>() as u32,
                );
                sqe.set_user_data(self.connect_user_data.as_u64());
                self.state = State::Connecting;

                Ok(true)
            }
            State::Connecting => Ok(false),

            State::CanReadWrite => match self.fsm.wants()? {
                Wants::Read(buf) => {
                    let mut sqe = ring.get_sqe()?;
                    sqe.prep_read(self.fd, buf.as_mut_ptr(), buf.len());
                    sqe.set_user_data(self.read_user_data.as_u64());

                    self.state = State::Reading;
                    Ok(true)
                }
                Wants::Write(buf) => {
                    let mut sqe = ring.get_sqe()?;
                    sqe.prep_write(self.fd, buf.as_ptr(), buf.len());
                    sqe.set_user_data(self.write_user_data.as_u64());

                    self.state = State::Writing;
                    Ok(true)
                }
                Wants::Done(response) => {
                    self.response_received(response);

                    let mut sqe = ring.get_sqe()?;
                    sqe.prep_close(self.fd);
                    sqe.set_user_data(self.close_user_data.as_u64());

                    self.state = State::Closing;
                    Ok(true)
                }
            },

            State::Reading => Ok(false),
            State::Writing => Ok(false),

            State::CanClose => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_close(self.fd);
                sqe.set_user_data(self.close_user_data.as_u64());

                self.state = State::Closing;
                Ok(true)
            }
            State::Closing => Ok(false),

            State::Closed => Ok(false),
        }
    }

    pub(crate) fn feed(&mut self, user_data: UserData, res: i32) -> Result<Option<Response>> {
        if user_data == self.socket_user_data {
            ensure!(
                matches!(self.state, State::AcquiringSocket),
                "malformed state, expected AcquiringSocket, got {:?}",
                self.state
            );

            let fd = res;
            ensure!(fd > 0);
            self.fd = fd;
            self.state = State::CanConnect;
            return Ok(None);
        }

        if user_data == self.connect_user_data {
            ensure!(
                matches!(self.state, State::Connecting),
                "malformed state, expected Connecting, got {:?}",
                self.state
            );

            ensure!(res >= 0);
            self.state = State::CanReadWrite;
            return Ok(None);
        }

        if user_data == self.read_user_data {
            ensure!(
                matches!(self.state, State::Reading),
                "malformed state, expected Reading, got {:?}",
                self.state
            );

            let read = res;
            ensure!(read >= 0);
            let read = read as usize;
            self.fsm.done_reading(read);

            match self.fsm.wants()? {
                Wants::Read(_) | Wants::Write(_) => {
                    self.state = State::CanReadWrite;
                }
                Wants::Done(response) => {
                    self.response_received(response);
                }
            }
            return Ok(None);
        }

        if user_data == self.write_user_data {
            ensure!(
                matches!(self.state, State::Writing),
                "malformed state, expected Writing, got {:?}",
                self.state
            );

            let written = res;
            ensure!(written >= 0);
            let written = written as usize;
            self.fsm.done_writing(written);

            match self.fsm.wants()? {
                Wants::Read(_) | Wants::Write(_) => {
                    self.state = State::CanReadWrite;
                }
                Wants::Done(response) => {
                    self.response_received(response);
                }
            }
            return Ok(None);
        }

        if user_data == self.close_user_data {
            ensure!(
                matches!(self.state, State::Closing),
                "malformed state, expected Closing, got {:?}",
                self.state
            );

            eprintln!("HttpsConnection closed");
            self.state = State::Closed;
            return Ok(self.response.take());
        }

        Ok(None)
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
