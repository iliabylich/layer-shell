use crate::{
    modules::FallibleModule,
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
}

impl FallibleModule for Location {
    const NAME: &str = "Location";
    type Output = (f64, f64);

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        self.https.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        let Some(response) = self.https.satisfy(satisfy, res) else {
            return Ok(None);
        };
        let location = LocationResponse::parse(&response)?;
        Ok(Some(location))
    }
}
