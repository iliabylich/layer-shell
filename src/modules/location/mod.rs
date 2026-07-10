use crate::{
    event_queue::EventQueue,
    sansio::{HttpRequest, Https, OpenSslContext, Satisfy, Wants},
};
use alloc::string::ToString as _;
use anyhow::Result;
use response::LocationResponse;
use rustix::net::SocketAddrAny;

mod response;

pub(crate) struct Location {
    https: Https,
}

impl Location {
    pub(crate) const HOST: &str = "myip.ibylich.dev";

    pub(crate) fn new(ctx: &OpenSslContext) -> Result<Self> {
        Ok(Self {
            https: Https::new(HttpRequest::get(Self::HOST, "/".to_string()), ctx)?,
        })
    }

    pub(crate) fn wants(&mut self, remote_server_addr: &SocketAddrAny) -> Option<Wants> {
        self.https.wants(remote_server_addr)
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        _events: &mut EventQueue,
    ) -> Result<Option<(f64, f64)>> {
        let Some(response) = self.https.satisfy(satisfy)? else {
            return Ok(None);
        };

        let (lat, lng) = LocationResponse::parse(&response)?;
        Ok(Some((lat, lng)))
    }
}
