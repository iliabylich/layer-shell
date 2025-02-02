use crate::scheduler::Actor;
use anyhow::Result;
use std::{ops::ControlFlow, sync::Arc, time::Duration};

mod client;
mod code;
mod mapper;

pub use code::WeatherCode;
use ureq::Agent;

#[derive(Debug)]
pub(crate) struct Weather {
    agent: Agent,
}

impl Actor for Weather {
    fn name() -> &'static str {
        "Weather"
    }

    fn start() -> Result<Box<dyn Actor>> {
        let tls_connector = Arc::new(ureq::native_tls::TlsConnector::new()?);
        let agent = ureq::AgentBuilder::new()
            .tls_connector(tls_connector)
            .build();

        Ok(Box::new(Weather { agent }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        let res = client::get_weather(&self.agent)?;

        let event = mapper::map_current(res.current);
        event.emit();

        let event = mapper::map_forecast(res.hourly, res.daily)?;
        event.emit();
        Ok(ControlFlow::Continue(Duration::from_secs(120)))
    }

    fn exec(&mut self, _: &crate::Command) -> Result<ControlFlow<()>> {
        Ok(ControlFlow::Break(()))
    }
}
