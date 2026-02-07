use crate::{https::HttpsConnection, user_data::ModuleId};
use response::LocationResponse;

mod response;

pub(crate) struct Location {
    https: HttpsConnection,
}

impl Location {
    pub(crate) fn new() -> Box<Self> {
        let https = HttpsConnection::get("myip.ibylich.dev", 443, "/", ModuleId::GeoLocation);

        Box::new(Self { https })
    }

    pub(crate) fn init(&mut self) {
        self.https.init()
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Option<(f64, f64)> {
        let response = self.https.process(op, res)?;

        LocationResponse::parse(response).ok()
    }
}
