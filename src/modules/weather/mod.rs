use crate::{Event, VerboseSender};
use anyhow::Result;

mod client;
mod code;
mod mapper;

pub use code::WeatherCode;

pub(crate) struct Weather {
    tx: VerboseSender<Event>,
}

impl Weather {
    pub(crate) const INTERVAL: u64 = 120;

    pub(crate) fn new(tx: VerboseSender<Event>) -> Self {
        Self { tx }
    }

    pub(crate) fn tick(&mut self) {
        if let Err(err) = self.try_tick() {
            log::error!("failed to get weather: {:?}", err);
        }
    }

    pub(crate) fn try_tick(&self) -> Result<()> {
        let res = client::get_weather()?;

        let (current, forecast) = mapper::map(res)?;
        self.tx.send(current);
        self.tx.send(forecast);
        Ok(())
    }
}
