mod request;
mod response;

use crate::sansio::{Dns, Satisfy, TlsOverTcp, Wants};
use anyhow::Result;
pub(crate) use request::HttpsRequest;
pub(crate) use response::HttpsResponse;
use rustls::pki_types::ServerName;

pub(crate) struct Https {
    domain: &'static str,
    request: Vec<u8>,
    state: State,
}

enum State {
    Dns(Box<Dns>),
    TlsOverTcp(Box<TlsOverTcp>),
    Done,
    Dead,
}

impl Https {
    pub(crate) fn new(request: HttpsRequest) -> Self {
        let domain = request.host();
        let request = request.into_bytes();
        Self {
            domain,
            request,
            state: State::Dns(Box::new(Dns::new(domain.as_bytes()))),
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match &mut self.state {
            State::Dns(dns) => dns.wants(),
            State::TlsOverTcp(tls_over_tcp) => tls_over_tcp.wants(),
            State::Done => Wants::Nothing,
            State::Dead => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<HttpsResponse>> {
        match (&mut self.state, satisfy) {
            (State::Dead, _) => {}
            (state, Satisfy::Crash) => {
                log::error!("Module HTTPS received Satisfy::Crash, stopping...");
                *state = State::Dead;
            }

            (State::Dns(dns), _) => {
                if let Some(mut addr) = dns.satisfy(satisfy, res)? {
                    addr.sin_port = 443_u16.to_be();
                    let request = core::mem::take(&mut self.request);
                    let server_name = ServerName::try_from(self.domain)?;
                    self.state =
                        State::TlsOverTcp(Box::new(TlsOverTcp::new(addr, server_name, request)?));
                }
            }
            (State::TlsOverTcp(tls_over_tcp), _) => {
                if let Some(response) = tls_over_tcp.satisfy(satisfy, res)? {
                    self.state = State::Done;
                    let response = HttpsResponse::parse(response)?;
                    return Ok(Some(response));
                }
            }
            (State::Done, _) => {}
        }

        Ok(None)
    }
}
