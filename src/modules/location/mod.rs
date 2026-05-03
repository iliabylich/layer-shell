use crate::{
    modules::Module,
    sansio::{HttpRequest, Https, Satisfy, Wants},
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
            https: Https::new(HttpRequest::get(HOST, "/".to_string())),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<(f64, f64)>> {
        let Some(response) = self.https.satisfy(satisfy, res) else {
            return Ok(None);
        };
        let location = LocationResponse::parse(&response)?;
        Ok(Some(location))
    }
}

impl Module for Location {
    type Output = Option<(f64, f64)>;

    fn wants(&mut self) -> Result<Option<Wants>> {
        self.https.wants()
    }

    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Self::Output {
        match self.try_satisfy(satisfy, res) {
            Ok(location) => location,
            Err(err) => {
                log::error!("Location module crashed: {satisfy:?} {res} {err:?}");
                None
            }
        }
    }
}
