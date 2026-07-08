use crate::{
    actor::{CanStop, TryWantsTrySatisfy},
    event_queue::EventQueue,
    sansio::{HttpRequest, Https, OpenSslContext, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;
use response::LocationResponse;

mod response;

const HOST: &str = "myip.ibylich.dev";

pub(crate) enum Location {
    Running(Https),
    Stopped,
}

impl Location {
    pub(crate) fn new(ctx: &OpenSslContext) -> Result<Self> {
        Ok(Self::Running(Https::new(
            HttpRequest::get(HOST, "/".to_string()),
            ctx,
        )?))
    }
}

impl TryWantsTrySatisfy for Location {
    const ID: ModuleId = ModuleId::Location;
    type Output = Option<(f64, f64)>;

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        match self {
            Self::Running(https) => https.try_wants(),
            Self::Stopped => Ok(None),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, _events: &mut EventQueue) -> Result<Self::Output> {
        let Self::Running(https) = self else {
            return Ok(None);
        };
        let Some(response) = https.try_satisfy(satisfy)? else {
            return Ok(None);
        };

        let (lat, lng) = LocationResponse::parse(&response)?;
        Ok(Some((lat, lng)))
    }
}

impl CanStop for Location {
    fn stopped(&mut self) -> Self {
        Self::Stopped
    }
}
