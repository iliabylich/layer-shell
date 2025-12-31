use crate::{
    Event, UserData,
    https::HttpsConnection,
    liburing::{Actor, Cqe, IoUring},
    weather::weather_response::WeatherResponse,
};
use anyhow::Result;
use location_response::LocationResponse;
pub(crate) use weather_code::WeatherCode;

mod location_response;
mod weather_code;
mod weather_response;

enum State {
    GettingLocation(HttpsConnection),
    GettingWeather(HttpsConnection),
}

pub(crate) struct Weather {
    state: State,
}

fn get_location() -> Result<HttpsConnection> {
    HttpsConnection::get(
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

fn get_weather(lat: f64, lng: f64) -> Result<HttpsConnection> {
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

    HttpsConnection::get(
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
        Ok(Self {
            state: State::GettingLocation(get_location()?),
        })
    }

    pub(crate) fn reset(&mut self) -> Result<()> {
        self.state = State::GettingLocation(get_location()?);
        Ok(())
    }
}

impl Actor for Weather {
    fn drain_once(&mut self, ring: &mut IoUring, _events: &mut Vec<Event>) -> Result<bool> {
        match &mut self.state {
            State::GettingLocation(https) => https.drain_once(ring),
            State::GettingWeather(https) => https.drain_once(ring),
        }
    }

    fn feed(&mut self, _ring: &mut IoUring, cqe: Cqe, events: &mut Vec<Event>) -> Result<()> {
        match &mut self.state {
            State::GettingLocation(https) => {
                if let Some(response) = https.feed(cqe)? {
                    let (lat, lng) = LocationResponse::parse(response)?;
                    self.state = State::GettingWeather(get_weather(lat, lng)?);
                }
            }
            State::GettingWeather(https) => {
                if let Some(response) = https.feed(cqe)? {
                    let event: Event = WeatherResponse::parse(response)?.try_into()?;
                    events.push(event);
                }
            }
        }

        Ok(())
    }
}
