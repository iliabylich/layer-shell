use crate::{https::HttpsConnection, liburing::IoUring, user_data::ModuleId};
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

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<bool> {
        let mut drained = false;

        loop {
            let drained_on_current_iteration = self.https.drain_once(ring)?;

            if !drained_on_current_iteration {
                break;
            }

            drained |= drained_on_current_iteration
        }

        Ok(drained)
    }

    pub(crate) fn feed(&mut self, op_id: u8, res: i32) -> Result<Option<(f64, f64)>> {
        if let Some(response) = self.https.feed(op_id, res)? {
            let (lat, lng) = LocationResponse::parse(response)?;
            return Ok(Some((lat, lng)));
        }

        Ok(None)
    }
}
