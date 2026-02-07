use crate::{https::HttpsConnection, user_data::ModuleId};
use anyhow::Result;
use response::LocationResponse;

mod response;

pub(crate) struct Location {
    https: HttpsConnection,
}

impl Location {
    pub(crate) fn new() -> Result<Box<Self>> {
        let https = HttpsConnection::get("myip.ibylich.dev", 443, "/", ModuleId::GeoLocation)?;

        Ok(Box::new(Self { https }))
    }

    pub(crate) fn init(&mut self) {
        self.https.init()
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Result<Option<(f64, f64)>> {
        let Some(response) = self.https.process(op, res)? else {
            return Ok(None);
        };

        Ok(Some(LocationResponse::parse(response)?))
    }
}
