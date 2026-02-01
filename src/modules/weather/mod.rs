use crate::{
    Event, https::HttpsConnection, liburing::IoUring,
    modules::weather::weather_response::WeatherResponse, timerfd::Tick, user_data::ModuleId,
};
use anyhow::Result;
pub(crate) use weather_code::WeatherCode;

mod weather_code;
mod weather_response;

pub(crate) struct Weather {
    latlng: Option<(f64, f64)>,
    https: Option<HttpsConnection>,
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
        ModuleId::Weather,
    )
}

impl Weather {
    pub(crate) fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            latlng: None,
            https: None,
        }))
    }

    pub(crate) fn init(&mut self, lat: f64, lng: f64, ring: &mut IoUring) -> Result<()> {
        self.latlng = Some((lat, lng));
        let mut https = get_weather(lat, lng)?;
        https.init(ring)?;
        self.https = Some(https);
        Ok(())
    }

    pub(crate) fn process(
        &mut self,
        op: u8,
        res: i32,
        ring: &mut IoUring,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        let Some(https) = self.https.as_mut() else {
            return Ok(());
        };
        if let Some(response) = https.process(op, res, ring)? {
            let event: Event = WeatherResponse::parse(response)?.try_into()?;
            events.push(event);
        }
        Ok(())
    }

    pub(crate) fn tick(&mut self, tick: Tick, ring: &mut IoUring) -> Result<()> {
        let Some((lat, lng)) = self.latlng else {
            return Ok(());
        };

        if tick.is_multiple_of(120) {
            let mut https = get_weather(lat, lng)?;
            https.init(ring)?;
            self.https = Some(https);
        }
        Ok(())
    }
}
