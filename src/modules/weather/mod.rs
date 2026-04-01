use crate::{
    Event,
    event_queue::EventQueue,
    modules::weather::weather_response::WeatherResponse,
    sansio::{Https, HttpsRequest, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;
pub use weather_code::WeatherCode;
pub use weather_response::{WeatherOnDay, WeatherOnHour};

mod weather_code;
mod weather_response;

const HOST: &str = "api.open-meteo.com";

pub(crate) struct Weather {
    lat: f64,
    lng: f64,
    https: Https,
}

impl Weather {
    pub(crate) fn new(lat: f64, lng: f64) -> Self {
        Self {
            lat,
            lng,
            https: Https::new(HttpsRequest::get(HOST, path(lat, lng))),
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::Weather
    }

    pub(crate) fn wants(&mut self) -> Wants {
        self.https.wants()
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        let Some(response) = self.https.satisfy(satisfy, res)? else {
            return Ok(());
        };
        let response = WeatherResponse::parse(response)?;
        let event = Event::try_from(response)?;
        EventQueue::push_back(event);
        Ok(())
    }

    pub(crate) fn tick(&mut self, tick: u64) {
        if tick.is_multiple_of(120) {
            self.https = Https::new(HttpsRequest::get(HOST, path(self.lat, self.lng)))
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
