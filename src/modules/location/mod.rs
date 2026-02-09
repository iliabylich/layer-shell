use crate::{dns::DnsResolver, https::HttpsConnection, user_data::ModuleId};
use response::LocationResponse;

mod response;

const HOST: &str = "myip.ibylich.dev";

pub(crate) struct Location {
    dns: Box<DnsResolver>,
    https: HttpsConnection,
}

impl Location {
    pub(crate) fn new() -> Box<Self> {
        let dns = DnsResolver::new(ModuleId::GeoLocationDNS, HOST.as_bytes());
        let https = HttpsConnection::new(HOST, "/", ModuleId::GeoLocationHTTPS);

        Box::new(Self { dns, https })
    }

    pub(crate) fn init(&mut self) {
        self.dns.init();
    }

    pub(crate) fn process_dns(&mut self, op: u8, res: i32) {
        if let Some(address) = self.dns.process(op, res) {
            self.https.init(address, 443);
        }
    }

    pub(crate) fn process_https(&mut self, op: u8, res: i32) -> Option<(f64, f64)> {
        let response = self.https.process(op, res)?;

        LocationResponse::parse(response).ok()
    }
}
