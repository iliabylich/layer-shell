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

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<bool> {
        let Some(https) = self.https.as_mut() else {
            return Ok(false);
        };

        let mut drained = false;

        loop {
            let drained_on_current_iteration = https.drain_once(ring)?;

            if !drained_on_current_iteration {
                break;
            }

            drained |= drained_on_current_iteration
        }

        Ok(drained)
    }

    pub(crate) fn set_location(&mut self, lat: f64, lng: f64) -> Result<()> {
        self.latlng = Some((lat, lng));
        self.https = Some(get_weather(lat, lng)?);
        Ok(())
    }

    pub(crate) fn feed(&mut self, op_id: u8, res: i32, events: &mut Vec<Event>) -> Result<()> {
        let Some(https) = self.https.as_mut() else {
            return Ok(());
        };

        if let Some(response) = https.feed(op_id, res)? {
            let event: Event = WeatherResponse::parse(response)?.try_into()?;
            events.push(event);
        }

        Ok(())
    }

    pub(crate) fn tick(&mut self, tick: Tick, ring: &mut IoUring) -> Result<bool> {
        let Some((lat, lng)) = self.latlng else {
            return Ok(false);
        };

        if tick.is_multiple_of(120) {
            self.https = Some(get_weather(lat, lng)?);
            self.drain(ring)
        } else {
            Ok(false)
        }
    }
}
