use crate::{
    Event, https::HttpsConnection, modules::weather::weather_response::WeatherResponse,
    timerfd::Tick, user_data::ModuleId,
};
pub(crate) use weather_code::WeatherCode;
pub use weather_response::{WeatherOnDay, WeatherOnHour};

mod weather_code;
mod weather_response;

pub(crate) struct Weather {
    latlng: Option<(f64, f64)>,
    https: Option<HttpsConnection>,
}

fn get_weather(lat: f64, lng: f64) -> HttpsConnection {
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
        ModuleId::Weather,
    )
}

impl Weather {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            latlng: None,
            https: None,
        })
    }

    pub(crate) fn init(&mut self, lat: f64, lng: f64) {
        self.latlng = Some((lat, lng));
        let https = get_weather(lat, lng);
        https.init();
        self.https = Some(https);
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        let Some(https) = self.https.as_mut() else {
            return;
        };
        let Some(response) = https.process(op, res) else {
            return;
        };
        let response = match WeatherResponse::parse(response) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("{err:?}");
                return;
            }
        };
        let event = match Event::try_from(response) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("{err:?}");
                return;
            }
        };
        events.push(event);
    }

    pub(crate) fn tick(&mut self, tick: Tick) {
        let Some((lat, lng)) = self.latlng else {
            return;
        };

        if tick.is_multiple_of(120) {
            let https = get_weather(lat, lng);
            https.init();
            self.https = Some(https);
        }
    }
}
