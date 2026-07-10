use crate::{
    Event,
    event_queue::EventQueue,
    modules::weather::weather_response::WeatherResponse,
    sansio::{HttpRequest, Https, OpenSslContext, Satisfy, Wants},
    utils::ArrayWriter,
};
use alloc::{
    boxed::Box,
    string::{String, ToString as _},
};
use anyhow::Result;
use core::fmt::Write;
use rustix::net::SocketAddrAny;
pub use weather_code::WeatherCode;
pub use weather_response::{
    DAILY_WEATHER_FORECAST_LENGTH, HOURLY_WEATHER_FORECAST_LENGTH, WeatherOnDay, WeatherOnHour,
};

mod weather_code;
mod weather_response;

pub(crate) struct Weather {
    https: Box<Https>,
}

impl Weather {
    pub(crate) const HOST: &str = "api.open-meteo.com";

    pub(crate) fn new(lat: f64, lng: f64, ctx: &OpenSslContext) -> Result<Self> {
        Ok(Self {
            https: Box::new(Https::new(
                HttpRequest::get(Self::HOST, path(lat, lng)?),
                ctx,
            )?),
        })
    }

    pub(crate) fn wants(&mut self, remote_server_addr: &SocketAddrAny) -> Option<Wants> {
        self.https.wants(remote_server_addr)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<()> {
        let Some(response) = self.https.satisfy(satisfy)? else {
            return Ok(());
        };
        let response = WeatherResponse::parse(&response)?;
        let event = Event::try_from(response)?;
        events.push_back(event);
        Ok(())
    }

    pub(crate) fn tick(
        &mut self,
        tick: u64,
        lat: f64,
        lng: f64,
        ctx: &OpenSslContext,
    ) -> Result<()> {
        if !tick.is_multiple_of(60) {
            return Ok(());
        }

        *self = Self::new(lat, lng, ctx)?;

        Ok(())
    }
}

fn path(lat: f64, lng: f64) -> Result<String> {
    let mut buf = [0; 1_024];
    let mut w = ArrayWriter::new(&mut buf);
    write!(&mut w, "/v1/forecast")?;
    write!(&mut w, "?latitude={lat}")?;
    write!(&mut w, "&longitude={lng}")?;
    write!(&mut w, "&current=temperature_2m,weather_code")?;
    write!(&mut w, "&hourly=temperature_2m,weather_code")?;
    write!(
        &mut w,
        "&daily=temperature_2m_min,temperature_2m_max,weather_code"
    )?;
    write!(&mut w, "&timezone=Europe/Warsaw")?;
    write!(&mut w, "&timeformat=unixtime")?;
    Ok(w.as_str()?.to_string())
}
