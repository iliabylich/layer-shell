mod client_config;
mod request;
mod response;

use anyhow::Result;
use client_config::get_client_config;
pub(crate) use request::Request;
pub(crate) use response::Response;
use rustls::{
    client::UnbufferedClientConnection,
    pki_types::ServerName,
    unbuffered::{
        AppDataRecord, ConnectionState, EncodeError, EncryptError, InsufficientSizeError,
        UnbufferedStatus,
    },
};

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct FSM {
    conn: Option<UnbufferedClientConnection>,
    healthy: bool,

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
}

pub(crate) enum Wants<'a> {
    Read(&'a mut [u8]),
    Write(&'a [u8]),
    Done(Response),
}
impl std::fmt::Debug for Wants<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(buf) => f.debug_tuple("Read").field(&buf.len()).finish(),
            Self::Write(buf) => f.debug_tuple("Write").field(&buf.len()).finish(),
            Self::Done(res) => f.debug_tuple("Done").field(res).finish(),
        }
    }
}

impl FSM {
    pub(crate) fn new(server_name: ServerName<'static>, request: Request) -> Result<Self> {
        let (conn, healthy) =
            match UnbufferedClientConnection::new(get_client_config(), server_name) {
                Ok(conn) => (Some(conn), true),
                Err(err) => {
                    eprintln!("failed to create UnbufferedClientConnection: {err:?}");
                    (None, false)
                }
            };

        Ok(Self {
            conn,
            healthy,

            request: request.into_bytes(),
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
        })
    }

    pub(crate) fn wants(&mut self) -> Option<Wants<'_>> {
        if !self.healthy {
            return None;
        }

        macro_rules! crash {
            ($($arg:tt)*) => {{
                eprintln!($($arg)*);
                self.healthy = false;
                return None;
            }};
        }

        loop {
            let UnbufferedStatus { discard, state } =
                unsafe { self.conn.as_mut().unwrap_unchecked() }.process_tls_records(
                    &mut self.incoming_tls[self.incoming_start..self.incoming_end],
                );

            self.incoming_start += discard;

            let state = match state {
                Ok(ok) => ok,
                Err(err) => crash!("malformed internal state: {err:?}"),
            };

            match state {
                ConnectionState::ReadTraffic(mut state) => {
                    while let Some(res) = state.next_record() {
                        let AppDataRecord { discard, payload } = match res {
                            Ok(ok) => ok,
                            Err(err) => crash!("failed to get AppDataRecord: {err:?}"),
                        };

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
                            match state.encode(&mut self.outgoing_tls[self.outgoing_end..]) {
                                Ok(ok) => ok,
                                Err(err) => crash!("{err:?}"),
                            }
                        }

                        Err(err) => {
                            crash!("{err:?}")
                        }
                    };

                    self.outgoing_end += written;
                }

                ConnectionState::TransmitTlsData(mut state) => {
                    if let Some(mut may_encrypt) = state.may_encrypt_app_data()
                        && !self.sent_request
                    {
                        let written = match may_encrypt
                            .encrypt(&self.request, &mut self.outgoing_tls[self.outgoing_end..])
                        {
                            Ok(ok) => ok,
                            Err(err) => {
                                crash!("encrypted request does not fit in `outgoing_tls`: {err:?}")
                            }
                        };
                        self.outgoing_end += written;
                        self.sent_request = true;
                    }

                    if self.outgoing_start == self.outgoing_end {
                        state.done();
                    } else {
                        return Some(self.wants_write());
                    }
                }

                ConnectionState::BlockedHandshake { .. } => {
                    self.resize_incoming_if_needed();
                    return Some(self.wants_read());
                }

                ConnectionState::WriteTraffic(mut may_encrypt) => {
                    if !self.sent_request {
                        panic!("dead branch hit");
                    } else if !self.received_response {
                        // this happens in the TLS 1.3 case. the app-data was sent in the preceding
                        // `TransmitTlsData` state. the server should have already written a
                        // response which we can read out from the socket
                        self.resize_incoming_if_needed();

                        return Some(self.wants_read());
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
                                match may_encrypt
                                    .queue_close_notify(&mut self.outgoing_tls[self.outgoing_end..])
                                {
                                    Ok(ok) => ok,
                                    Err(err) => crash!("{err:?}"),
                                }
                            }

                            Err(err) => {
                                crash!("{err:?}")
                            }
                        };

                        self.outgoing_end += written;

                        self.we_closed = true;
                        return Some(self.wants_write());
                    } else {
                        self.resize_incoming_if_needed();

                        return Some(self.wants_read());
                    }
                }

                ConnectionState::PeerClosed => {}

                ConnectionState::Closed => {
                    if !self.sent_request {
                        crash!("request is not sent");
                    }
                    if !self.received_response {
                        crash!("response is not received");
                    }
                    if self.incoming_start != self.incoming_end {
                        crash!(
                            "self.incoming_start ({}) != self.incoming_end ({})",
                            self.incoming_start,
                            self.incoming_end
                        );
                    }

                    let response = std::mem::take(&mut self.response);
                    let response = match Response::parse(response) {
                        Ok(ok) => ok,
                        Err(err) => crash!("{err:?}"),
                    };
                    return Some(Wants::Done(response));
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

    fn wants_write(&self) -> Wants<'_> {
        Wants::Write(&self.outgoing_tls[self.outgoing_start..self.outgoing_end])
    }

    fn wants_read(&mut self) -> Wants<'_> {
        Wants::Read(&mut self.incoming_tls[self.incoming_end..])
    }

    pub(crate) fn done_reading(&mut self, read: usize) {
        self.incoming_end += read;
    }

    pub(crate) fn done_writing(&mut self, written: usize) {
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
