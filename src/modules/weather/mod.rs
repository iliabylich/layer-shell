use crate::{
    Event,
    event_queue::EventQueue,
    modules::{FallibleModule, weather::weather_response::WeatherResponse},
    sansio::{HttpRequest, Https, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;
pub use weather_code::WeatherCode;
pub use weather_response::{
    DAILY_WEATHER_FORECAST_LENGTH, HOURLY_WEATHER_FORECAST_LENGTH, WeatherOnDay, WeatherOnHour,
};

mod weather_code;
mod weather_response;

const HOST: &str = "api.open-meteo.com";

pub(crate) enum Weather {
    WaitingForLocation,
    Ready { lat: f64, lng: f64, https: Https },
    Dead { latlng: Option<(f64, f64)> },
}

impl Weather {
    pub(crate) const fn new() -> Self {
        Self::WaitingForLocation
    }

    pub(crate) fn setup(&mut self, lat: f64, lng: f64) {
        *self = Self::Ready {
            lat,
            lng,
            https: Https::new(HttpRequest::get(HOST, path(lat, lng))),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        match self {
            Self::Ready { https, .. } => {
                let Some(response) = https.try_satisfy(satisfy, res)? else {
                    return Ok(());
                };
                let response = WeatherResponse::parse(&response)?;
                let event = Event::try_from(response)?;
                EventQueue::push_back(event);
                Ok(())
            }
            Self::Dead { .. } | Self::WaitingForLocation => Ok(()),
        }
    }

    const fn latlng(&self) -> Option<(f64, f64)> {
        match self {
            Self::WaitingForLocation => None,
            Self::Ready { lat, lng, .. } => Some((*lat, *lng)),
            Self::Dead { latlng } => *latlng,
        }
    }
}

impl FallibleModule for Weather {
    const MODULE_ID: ModuleId = ModuleId::Weather;
    type Output = ();

    fn wants(&mut self) -> Option<Wants> {
        match self {
            Self::Ready { https, .. } => https.wants(),

            Self::WaitingForLocation | Self::Dead { .. } => None,
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        if matches!(self, Self::Dead { .. }) {
            return Ok(None);
        }

        if let Err(err) = self.try_satisfy(satisfy, res) {
            log::error!("Weather module crashed: {satisfy:?} {res} {err:?}");
            *self = Self::Dead {
                latlng: self.latlng(),
            };
        }

        Ok(None)
    }

    fn try_tick(&mut self, tick: u64) -> Result<()> {
        if !tick.is_multiple_of(60) {
            return Ok(());
        }

        if let Self::Ready { https, .. } = self
            && https.is_waiting()
        {
            return Ok(());
        }

        if let Some((lat, lng)) = self.latlng() {
            *self = Self::Ready {
                lat,
                lng,
                https: Https::new(HttpRequest::get(HOST, path(lat, lng))),
            };
        }

        Ok(())
    }
}

fn path(lat: f64, lng: f64) -> String {
    let query = format!(
        "{}={}&{}={}&{}={}&{}={}&{}={}&{}={}&{}={}",
        "latitude",
        lat,
        "longitude",
        lng,
        "current",
        "temperature_2m,weather_code",
        "hourly",
        "temperature_2m,weather_code",
        "daily",
        "temperature_2m_min,temperature_2m_max,weather_code",
        "timezone",
        "Europe/Warsaw",
        "timeformat",
        "unixtime"
    );

    format!("/v1/forecast?{query}")
}
