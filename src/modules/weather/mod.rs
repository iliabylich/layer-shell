use std::{sync::Arc, time::Duration};

use crate::scheduler::{Module, RepeatingModule};
use anyhow::Result;

mod client;
mod code;
mod mapper;

pub use code::WeatherCode;
use ureq::Agent;

pub(crate) struct Weather {
    agent: Agent,
}

impl Module for Weather {
    const NAME: &str = "Weather";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
        let tls_connector = Arc::new(ureq::native_tls::TlsConnector::new()?);
        let agent = ureq::AgentBuilder::new()
            .tls_connector(tls_connector)
            .build();

        Ok(Some(Box::new(Weather { agent })))
    }
}

impl RepeatingModule for Weather {
    fn tick(&mut self) -> Result<Duration> {
        let res = client::get_weather(&self.agent)?;

        let event = mapper::map_current(res.current);
        event.emit();

        let event = mapper::map_forecast(res.hourly, res.daily)?;
        event.emit();

        Ok(Duration::from_secs(2))
    }
}
