use crate::{
    Event,
    event_queue::EventQueue,
    modules::weather::weather_response::WeatherResponse,
    sansio::{Https, HttpsRequest, Satisfy, Wants},
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
    pub(crate) fn new() -> Self {
        Self::WaitingForLocation
    }

    pub(crate) fn setup(&mut self, lat: f64, lng: f64) {
        *self = Self::Ready {
            lat,
            lng,
            https: Https::new(HttpsRequest::get(HOST, path(lat, lng))),
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::Weather
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self {
            Weather::WaitingForLocation => None,
            Weather::Ready { https, .. } => https.wants(),
            Weather::Dead { .. } => None,
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        match self {
            Weather::WaitingForLocation => Ok(()),
            Weather::Ready { https, .. } => {
                let Some(response) = https.satisfy(satisfy, res)? else {
                    return Ok(());
                };
                let response = WeatherResponse::parse(response)?;
                let event = Event::try_from(response)?;
                EventQueue::push_back(event);
                Ok(())
            }
            Weather::Dead { .. } => Ok(()),
        }
    }

    fn latlng(&self) -> Option<(f64, f64)> {
        match self {
            Weather::WaitingForLocation => None,
            Weather::Ready { lat, lng, .. } => Some((*lat, *lng)),
            Weather::Dead { latlng } => *latlng,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) {
        if matches!(self, Self::Dead { .. }) {
            return;
        }

        if let Err(err) = self.try_satisfy(satisfy, res) {
            log::error!("Weather module crashed: {satisfy:?} {res} {err:?}");
            *self = Self::Dead {
                latlng: self.latlng(),
            };
        }
    }

    pub(crate) fn tick(&mut self, tick: u64) {
        if tick.is_multiple_of(10)
            && let Some((lat, lng)) = self.latlng()
        {
            *self = Self::Ready {
                lat,
                lng,
                https: Https::new(HttpsRequest::get(HOST, path(lat, lng))),
            }
        }
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
