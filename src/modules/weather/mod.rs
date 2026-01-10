use crate::{
    Event, UserData,
    https::HttpsConnection,
    liburing::{IoUring, IoUringActor},
    modules::weather::weather_response::WeatherResponse,
    timerfd::Tick,
};
use anyhow::Result;
use location_response::LocationResponse;
pub(crate) use weather_code::WeatherCode;

mod location_response;
mod weather_code;
mod weather_response;

enum State {
    WaitingForTimer,
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
        UserData::GetLocationSocket,
        UserData::GetLocationConnect,
        UserData::GetLocationRead,
        UserData::GetLocationWrite,
        UserData::GetLocationClose,
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
        UserData::GetWeatherSocket,
        UserData::GetWeatherConnect,
        UserData::GetWeatherRead,
        UserData::GetWeatherWrite,
        UserData::GetWeatherClose,
    )
}

impl Weather {
    pub(crate) fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            state: State::WaitingForTimer,
        }))
    }
}

impl IoUringActor for Weather {
    fn drain_once(&mut self, ring: &mut IoUring, _events: &mut Vec<Event>) -> Result<bool> {
        match &mut self.state {
            State::WaitingForTimer => Ok(false),
            State::GettingLocation(https) => https.drain_once(ring),
            State::GettingWeather(https) => https.drain_once(ring),
        }
    }

    fn feed(
        &mut self,
        _ring: &mut IoUring,
        user_data: UserData,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        match &mut self.state {
            State::WaitingForTimer => {}
            State::GettingLocation(https) => {
                if let Some(response) = https.feed(user_data, res)? {
                    let (lat, lng) = LocationResponse::parse(response)?;
                    self.state = State::GettingWeather(get_weather(lat, lng)?);
                }
            }
            State::GettingWeather(https) => {
                if let Some(response) = https.feed(user_data, res)? {
                    let event: Event = WeatherResponse::parse(response)?.try_into()?;
                    events.push(event);
                }
            }
        }

        Ok(())
    }

    fn on_tick(&mut self, tick: Tick) -> Result<()> {
        if tick.is_multiple_of(120) {
            self.state = State::GettingLocation(get_location()?);
        }
        Ok(())
    }
}
