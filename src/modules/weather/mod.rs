use crate::{
    Event,
    actor::{CanStop, TryWantsTrySatisfy},
    event_queue::EventQueue,
    modules::weather::weather_response::WeatherResponse,
    sansio::{HttpRequest, Https, OpenSslContext, Satisfy, Wants},
    user_data::ModuleId,
    utils::ArrayWriter,
};
use alloc::{
    boxed::Box,
    string::{String, ToString as _},
};
use anyhow::Result;
use core::fmt::Write;
pub use weather_code::WeatherCode;
pub use weather_response::{
    DAILY_WEATHER_FORECAST_LENGTH, HOURLY_WEATHER_FORECAST_LENGTH, WeatherOnDay, WeatherOnHour,
};

mod weather_code;
mod weather_response;

const HOST: &str = "api.open-meteo.com";

pub(crate) enum Weather {
    WaitingForLocation,
    Ready {
        lat: f64,
        lng: f64,
        https: Box<Https>,
    },
    Stopped {
        latlng: Option<(f64, f64)>,
    },
}

impl Weather {
    pub(crate) const fn new() -> Self {
        Self::WaitingForLocation
    }

    pub(crate) fn start(&mut self, lat: f64, lng: f64, ctx: &OpenSslContext) -> Result<()> {
        *self = Self::Ready {
            lat,
            lng,
            https: Box::new(Https::new(HttpRequest::get(HOST, path(lat, lng)?), ctx)?),
        };
        Ok(())
    }

    const fn latlng(&self) -> Option<(f64, f64)> {
        match self {
            Self::WaitingForLocation => None,
            Self::Ready { lat, lng, .. } => Some((*lat, *lng)),
            Self::Stopped { latlng } => *latlng,
        }
    }

    pub(crate) fn tick(&mut self, tick: u64, ctx: &OpenSslContext) -> Result<()> {
        if !tick.is_multiple_of(60) {
            return Ok(());
        }

        if let Some((lat, lng)) = self.latlng() {
            *self = Self::Ready {
                lat,
                lng,
                https: Box::new(Https::new(HttpRequest::get(HOST, path(lat, lng)?), ctx)?),
            };
        }

        Ok(())
    }
}

impl TryWantsTrySatisfy for Weather {
    const ID: ModuleId = ModuleId::Weather;
    type Output = ();

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        match self {
            Self::Ready { https, .. } => https.try_wants(),
            Self::WaitingForLocation | Self::Stopped { .. } => Ok(None),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<Self::Output> {
        match self {
            Self::Ready { https, .. } => {
                let Some(response) = https.try_satisfy(satisfy)? else {
                    return Ok(());
                };
                let response = WeatherResponse::parse(&response)?;
                let event = Event::try_from(response)?;
                events.push_back(event);
                Ok(())
            }
            Self::Stopped { .. } | Self::WaitingForLocation => Ok(()),
        }
    }
}

impl CanStop for Weather {
    fn stopped(&mut self) -> Self {
        Self::Stopped {
            latlng: self.latlng(),
        }
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
