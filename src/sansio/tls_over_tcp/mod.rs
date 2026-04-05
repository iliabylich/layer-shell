mod client_config;

use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use client_config::get_client_config;
use rustls::{
    client::UnbufferedClientConnection,
    pki_types::ServerName,
    unbuffered::{
        AppDataRecord, ConnectionState, EncodeError, EncryptError, InsufficientSizeError,
        UnbufferedStatus,
    },
};

pub(crate) struct TlsOverTcp {
    addr: libc::sockaddr_in,
    conn: UnbufferedClientConnection,

    request: Vec<u8>,
    response: Vec<u8>,

    incoming_tls: Vec<u8>,
    incoming_start: usize,
    incoming_end: usize,

    outgoing_tls: Vec<u8>,
    outgoing_start: usize,
    outgoing_end: usize,

    we_closed: bool,
    sent_request: bool,
    received_response: bool,

    fd: i32,
    state: State,
}

#[derive(Debug)]
enum State {
    CanSocket,
    WaitingForSocket,
    CanConnect,
    WaitingForConnect,
    CanRead,
    WaitingForRead,
    CanWrite,
    WaitingForWrite,
    CanClose,
    WaitingForClose,
    Done,
}

impl TlsOverTcp {
    pub(crate) fn new(
        addr: libc::sockaddr_in,
        server_name: ServerName<'static>,
        request: Vec<u8>,
    ) -> Result<Self> {
        Ok(Self {
            addr,
            conn: UnbufferedClientConnection::new(get_client_config(), server_name)?,

            request,
            response: vec![],

            incoming_tls: vec![0; INCOMING_TLS_BUFSIZE],
            incoming_start: 0,
            incoming_end: 0,

            outgoing_tls: vec![0; OUTGOING_TLS_INITIAL_BUFSIZE],
            outgoing_start: 0,
            outgoing_end: 0,

            we_closed: false,
            sent_request: false,
            received_response: false,

            state: State::CanSocket,
            fd: -1,
        })
    }

    fn process_tls(&mut self) -> Result<()> {
        loop {
            let UnbufferedStatus { discard, state } = self.conn.process_tls_records(
                &mut self.incoming_tls[self.incoming_start..self.incoming_end],
            );

            self.incoming_start += discard;

            match state? {
                ConnectionState::ReadTraffic(mut state) => {
                    while let Some(res) = state.next_record() {
                        let AppDataRecord { discard, payload } = res?;
                        self.incoming_start += discard;
                        self.response.extend_from_slice(payload);
                        self.received_response = true;
                    }
                }

                ConnectionState::EncodeTlsData(mut state) => {
                    let written = match state.encode(&mut self.outgoing_tls[self.outgoing_end..]) {
                        Ok(written) => written,

                        Err(EncodeError::InsufficientSize(InsufficientSizeError {
                            required_size,
                        })) => {
                            let new_len = self.outgoing_end + required_size;
                            self.outgoing_tls.resize(new_len, 0);
                            state.encode(&mut self.outgoing_tls[self.outgoing_end..])?
                        }

                        Err(err) => {
                            return Err(anyhow::Error::from(err));
                        }
                    };

                    self.outgoing_end += written;
                }

                ConnectionState::TransmitTlsData(mut state) => {
                    if let Some(mut may_encrypt) = state.may_encrypt_app_data()
                        && !self.sent_request
                    {
                        let written = may_encrypt
                            .encrypt(&self.request, &mut self.outgoing_tls[self.outgoing_end..])?;
                        self.outgoing_end += written;
                        self.sent_request = true;
                    }

                    if self.outgoing_start == self.outgoing_end {
                        state.done();
                    } else {
                        self.state = State::CanWrite;
                        return Ok(());
                    }
                }

                ConnectionState::BlockedHandshake { .. } => {
                    self.resize_incoming_if_needed();
                    self.state = State::CanRead;
                    return Ok(());
                }

                ConnectionState::WriteTraffic(mut may_encrypt) => {
                    if !self.sent_request {
                        panic!("dead branch hit");
                    } else if !self.received_response {
                        // this happens in the TLS 1.3 case. the app-data was sent in the preceding
                        // `TransmitTlsData` state. the server should have already written a
                        // response which we can read out from the socket
                        self.resize_incoming_if_needed();

                        self.state = State::CanRead;
                        return Ok(());
                    } else if !self.we_closed {
                        let written = match may_encrypt
                            .queue_close_notify(&mut self.outgoing_tls[self.outgoing_end..])
                        {
                            Ok(written) => written,

                            Err(EncryptError::InsufficientSize(InsufficientSizeError {
                                required_size,
                            })) => {
                                let new_len = self.outgoing_end + required_size;
                                self.outgoing_tls.resize(new_len, 0);
                                may_encrypt.queue_close_notify(
                                    &mut self.outgoing_tls[self.outgoing_end..],
                                )?
                            }

                            Err(err) => return Err(anyhow::Error::from(err)),
                        };

                        self.outgoing_end += written;

                        self.we_closed = true;
                        self.state = State::CanWrite;
                        return Ok(());
                    } else {
                        self.resize_incoming_if_needed();

                        self.state = State::CanRead;
                        return Ok(());
                    }
                }

                ConnectionState::PeerClosed => {}

                ConnectionState::Closed => {
                    if !self.sent_request {
                        bail!("request is not sent");
                    }
                    if !self.received_response {
                        bail!("response is not received");
                    }
                    if self.incoming_start != self.incoming_end {
                        bail!(
                            "self.incoming_start ({}) != self.incoming_end ({})",
                            self.incoming_start,
                            self.incoming_end
                        );
                    }

                    self.state = State::CanClose;
                    return Ok(());
                }

                _ => unreachable!(),
            }
        }
    }

    fn resize_incoming_if_needed(&mut self) {
        if self.incoming_end == self.incoming_tls.len() {
            let new_len = self.incoming_tls.len() + INCOMING_TLS_BUFSIZE;
            self.incoming_tls.resize(new_len, 0);
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match self.state {
            State::CanSocket => {
                self.state = State::WaitingForSocket;
                Wants::Socket {
                    domain: libc::AF_INET,
                    r#type: libc::SOCK_STREAM,
                }
            }
            State::WaitingForSocket => Wants::Nothing,

            State::CanConnect => {
                self.state = State::WaitingForConnect;
                Wants::Connect {
                    fd: self.fd,
                    addr: (&self.addr as *const libc::sockaddr_in).cast(),
                    addrlen: core::mem::size_of::<libc::sockaddr_in>() as u32,
                }
            }
            State::WaitingForConnect => Wants::Nothing,

            State::CanRead => {
                self.state = State::WaitingForRead;
                let buf = &mut self.incoming_tls[self.incoming_end..];
                Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                }
            }
            State::WaitingForRead => Wants::Nothing,

            State::CanWrite => {
                self.state = State::WaitingForWrite;
                let buf = &self.outgoing_tls[self.outgoing_start..self.outgoing_end];
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }
            State::WaitingForWrite => Wants::Nothing,

            State::CanClose => {
                self.state = State::WaitingForClose;
                Wants::Close { fd: self.fd }
            }
            State::WaitingForClose => Wants::Nothing,

            State::Done => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Vec<u8>>> {
        match (&mut self.state, satisfy) {
            (State::WaitingForSocket, Satisfy::Socket) => {
                ensure!(res >= 0, "TlsOverTcp::Socket failed: {res}");
                self.fd = res;
                self.state = State::CanConnect;
                Ok(None)
            }

            (State::WaitingForConnect, Satisfy::Connect) => {
                ensure!(res >= 0, "TlsOverTcp::Connect failed: {res}");
                self.process_tls()?;
                Ok(None)
            }

            (State::WaitingForWrite, Satisfy::Write) => {
                ensure!(res >= 0, "TlsOverTcp::Write failed: {res}");
                let bytes_written = res as usize;
                self.done_writing(bytes_written);
                self.process_tls()?;
                Ok(None)
            }

            (State::WaitingForRead, Satisfy::Read) => {
                ensure!(res >= 0, "TlsOverTcp::Read failed: {res}");
                let bytes_read = res as usize;
                self.done_reading(bytes_read);
                self.process_tls()?;
                Ok(None)
            }

            (State::WaitingForClose, Satisfy::Close) => {
                ensure!(res >= 0, "TlsOverTcp::Close failed: {res}");
                self.state = State::Done;
                Ok(Some(core::mem::take(&mut self.response)))
            }

            (state, satisfy) => {
                bail!("malformed TLS state: {state:?} vs {satisfy:?}")
            }
        }
    }

    fn done_reading(&mut self, read: usize) {
        self.incoming_end += read;
    }

    fn done_writing(&mut self, written: usize) {
        self.outgoing_start += written;
        if self.outgoing_start == self.outgoing_end {
            self.outgoing_start = 0;
            self.outgoing_end = 0;
        }
    }
}

const KB: usize = 1024;
const INCOMING_TLS_BUFSIZE: usize = 16 * KB;
const OUTGOING_TLS_INITIAL_BUFSIZE: usize = KB;
