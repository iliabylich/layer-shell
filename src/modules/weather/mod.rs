use std::{any::Any, sync::Arc};

use crate::scheduler::Module;
use anyhow::{Context as _, Result};

mod client;
mod code;
mod mapper;

pub use code::WeatherCode;
use ureq::Agent;

pub(crate) struct Weather;

impl Module for Weather {
    const NAME: &str = "Weather";
    const INTERVAL: Option<u64> = Some(120_000);

    fn start() -> Result<Box<dyn Any + Send + 'static>> {
        let tls_connector = Arc::new(ureq::native_tls::TlsConnector::new()?);
        let agent = ureq::AgentBuilder::new()
            .tls_connector(tls_connector)
            .build();

        Ok(Box::new(agent))
    }

    fn tick(state: &mut Box<dyn Any + Send + 'static>) -> Result<()> {
        let agent = state
            .downcast_ref::<Agent>()
            .context("Weather state is malformed")?;

        let res = client::get_weather(agent)?;

        let event = mapper::map_current(res.current);
        event.emit();

        let event = mapper::map_forecast(res.hourly, res.daily)?;
        event.emit();

        Ok(())
    }
}
