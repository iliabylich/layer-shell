use crate::{
    Event, UserData,
    https::HttpsActor,
    liburing::{Actor, Cqe, IoUring, Pending},
    weather::weather_response::WeatherResponse,
};
use anyhow::Result;
use location_response::LocationResponse;
pub(crate) use weather_code::WeatherCode;

mod location_response;
mod weather_code;
mod weather_response;

pub(crate) enum Weather {
    GettingLocation {
        conn: HttpsActor,
        location: Option<(f64, f64)>,
    },
    GettingWeather {
        conn: HttpsActor,
    },
}

fn get_location() -> Result<HttpsActor> {
    HttpsActor::get(
        "myip.ibylich.dev",
        443,
        "/",
        UserData::GetLocationSocket as u64,
        UserData::GetLocationConnect as u64,
        UserData::GetLocationRead as u64,
        UserData::GetLocationWrite as u64,
        UserData::GetLocationClose as u64,
    )
}

fn get_weather(lat: f64, lng: f64) -> Result<HttpsActor> {
    let query = format!(
        "{}={}&{}={}&{}={}&{}={}&{}={}&{}={}",
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
        "Europe/Warsaw"
    );

    HttpsActor::get(
        "api.open-meteo.com",
        443,
        &format!("/v1/forecast?{query}"),
        UserData::GetWeatherSocket as u64,
        UserData::GetWeatherConnect as u64,
        UserData::GetWeatherRead as u64,
        UserData::GetWeatherWrite as u64,
        UserData::GetWeatherClose as u64,
    )
}

impl Weather {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self::GettingLocation {
            conn: get_location()?,
            location: None,
        })
    }

    pub(crate) fn reset(&mut self) -> Result<()> {
        *self = Self::GettingLocation {
            conn: get_location()?,
            location: None,
        };
        Ok(())
    }

    pub(crate) fn start_getting_weather(&mut self, lat: f64, lng: f64) -> Result<()> {
        println!("lat = {lat} / lng = {lng}");
        *self = Self::GettingWeather {
            conn: get_weather(lat, lng)?,
        };
        Ok(())
    }
}

impl Actor for Weather {
    fn drain_once(
        &mut self,
        ring: &mut IoUring,
        pending: &mut Pending,
        events: &mut Vec<Event>,
    ) -> Result<bool> {
        match self {
            Weather::GettingLocation { conn, location } => {
                if let Some((lat, lng)) = location.map(|e| e)
                    && conn.is_closed()
                {
                    self.start_getting_weather(lat, lng)?;
                    return self.drain_once(ring, pending, events);
                }

                let (drained, response) = conn.drain(ring, pending)?;
                if let Some(response) = response {
                    let (lat, lng) = LocationResponse::parse(response)?;
                    *location = Some((lat, lng));
                };
                Ok(drained)
            }
            Weather::GettingWeather { conn } => {
                let (drained, response) = conn.drain(ring, pending)?;
                if let Some(response) = response {
                    let event: Event = WeatherResponse::parse(response)?.try_into()?;
                    events.push(event);
                };
                Ok(drained)
            }
        }
    }

    fn feed(&mut self, ring: &mut IoUring, cqe: Cqe, events: &mut Vec<Event>) -> Result<()> {
        match self {
            Weather::GettingLocation { conn, .. } | Weather::GettingWeather { conn } => {
                conn.feed(ring, cqe)?
            }
        }

        Ok(())
    }
}
