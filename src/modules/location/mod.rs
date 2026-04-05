use crate::{
    sansio::{Https, HttpsRequest, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;
use response::LocationResponse;

mod response;

const HOST: &str = "myip.ibylich.dev";

pub(crate) struct Location {
    https: Https,
    dead: bool,
}

impl Location {
    pub(crate) fn new() -> Self {
        Self {
            https: Https::new(HttpsRequest::get(HOST, "/")),
            dead: false,
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::GeoLocation
    }

    pub(crate) fn wants(&mut self) -> Wants {
        if self.dead {
            return Wants::Nothing;
        }

        self.https.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<(f64, f64)>> {
        let Some(response) = self.https.satisfy(satisfy, res)? else {
            return Ok(None);
        };
        let location = LocationResponse::parse(response)?;
        Ok(Some(location))
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<(f64, f64)> {
        if self.dead {
            return None;
        }

        match self.try_satisfy(satisfy, res) {
            Ok(location) => location,
            Err(err) => {
                log::error!("Location module crashed: {satisfy:?} {res} {err:?}");
                self.dead = true;
                None
            }
        }
    }
}
