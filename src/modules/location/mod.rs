use crate::sansio::{HttpRequest, Https, Satisfy, Wants};
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

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.https.wants()
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) -> Option<(f64, f64)> {
        let response = self.https.satisfy(satisfy)?;

        match LocationResponse::parse(&response) {
            Ok(location) => Some(location),
            Err(err) => {
                log::error!("{err:?}");
                self.https.stop();
                None
            }
        }
    }
}
