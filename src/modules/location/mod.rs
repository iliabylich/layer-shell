use crate::{
    modules::Module,
    sansio::{Https, HttpsRequest, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;
use response::LocationResponse;

mod response;

const HOST: &str = "myip.ibylich.dev";

pub(crate) struct Location {
    https: Https,
}

impl Location {
    pub(crate) fn new() -> Self {
        Self {
            https: Https::new(HttpsRequest::get(HOST, "/")),
        }
    }
}

impl Module for Location {
    type Output = Option<(f64, f64)>;
    type Error = anyhow::Error;
    const MODULE_ID: ModuleId = ModuleId::GeoLocation;

    fn wants(&mut self) -> Wants {
        self.https.wants()
    }

    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Self::Output, Self::Error> {
        let Some(response) = self.https.satisfy(satisfy, res)? else {
            return Ok(None);
        };
        let location = LocationResponse::parse(response)?;
        Ok(Some(location))
    }

    fn tick(&mut self, _tick: u64) {}
}
