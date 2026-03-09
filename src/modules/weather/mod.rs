use crate::{
    Event,
    modules::{Module, weather::weather_response::WeatherResponse},
    sansio::{Https, HttpsRequest, Satisfy, Wants},
    user_data::ModuleId,
};
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

impl Module for Weather {
    type Input = (f64, f64);
    type Output = ();
    type Error = anyhow::Error;

    const MODULE_ID: ModuleId = ModuleId::Weather;

    fn new((lat, lng): (f64, f64)) -> Self {
        Self {
            lat,
            lng,
            https: Https::new(HttpsRequest::get(HOST, path(lat, lng))),
        }
    }

    fn wants(&mut self) -> Wants {
        self.https.wants()
    }

    fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<Self::Output, Self::Error> {
        let Some(response) = self.https.satisfy(satisfy, res)? else {
            return Ok(());
        };
        let response = WeatherResponse::parse(response)?;
        let event = Event::try_from(response)?;
        events.push(event);
        Ok(())
    }

    fn tick(&mut self, tick: u64) {
        if tick.is_multiple_of(120) {
            self.https = Https::new(HttpsRequest::get(HOST, path(self.lat, self.lng)))
        }
    }
}
