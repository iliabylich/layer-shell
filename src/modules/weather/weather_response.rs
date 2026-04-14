use super::WeatherCode;
use crate::{Event, sansio::HttpsResponse};
use anyhow::{Context as _, Result, ensure};
use chrono::TimeZone;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct WeatherResponse {
    pub(crate) current: CurrentWeatherResponse,
    pub(crate) hourly: HourlyWeatherResponse,
    pub(crate) daily: DailyWeatherResponse,
}

#[derive(Deserialize, Debug)]
pub(crate) struct CurrentWeatherResponse {
    pub(crate) temperature_2m: f32,
    pub(crate) weather_code: u32,
}

#[derive(Deserialize, Debug)]
pub(crate) struct HourlyWeatherResponse {
    pub(crate) time: Vec<i64>,
    pub(crate) temperature_2m: Vec<f32>,
    pub(crate) weather_code: Vec<u32>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct DailyWeatherResponse {
    pub(crate) time: Vec<i64>,
    pub(crate) temperature_2m_min: Vec<f32>,
    pub(crate) temperature_2m_max: Vec<f32>,
    pub(crate) weather_code: Vec<u32>,
}

impl WeatherResponse {
    pub(crate) fn parse(response: HttpsResponse) -> Result<Self> {
        ensure!(response.status == 200);
        serde_json::from_str(&response.body).context("malformed JSON output")
    }
}

impl TryFrom<WeatherResponse> for Event {
    type Error = anyhow::Error;

    fn try_from(response: WeatherResponse) -> Result<Self> {
        let WeatherResponse {
            current,
            hourly,
            daily,
        } = response;

        Ok(Event::Weather {
            temperature: current.temperature_2m,
            code: WeatherCode::from(current.weather_code),
            hourly_forecast: map_hourly_forecase(hourly)?,
            daily_forecast: map_daily_forecase(daily)?,
        })
    }
}

pub const HOURLY_WEATHER_FORECAST_LENGTH: usize = 10;
pub const DAILY_WEATHER_FORECAST_LENGTH: usize = 6;

fn map_hourly_forecase(
    response: HourlyWeatherResponse,
) -> Result<[WeatherOnHour; HOURLY_WEATHER_FORECAST_LENGTH]> {
    let HourlyWeatherResponse {
        time,
        temperature_2m,
        weather_code,
    } = response;
    let now = chrono::Local::now().timestamp();

    let mut forecast = vec![];
    for ((temp, code), time) in temperature_2m.into_iter().zip(weather_code).zip(time) {
        let code = WeatherCode::from(code);

        if time > now {
            forecast.push(WeatherOnHour {
                unix_seconds: time,
                temperature: temp,
                code,
            });
        }
        if forecast.len() == 10 {
            break;
        }
    }

    forecast.try_into().map_err(|v: Vec<WeatherOnHour>| {
        anyhow::anyhow!(
            "wrong size: {} vs {HOURLY_WEATHER_FORECAST_LENGTH}",
            v.len()
        )
    })
}

fn map_daily_forecase(
    response: DailyWeatherResponse,
) -> Result<[WeatherOnDay; DAILY_WEATHER_FORECAST_LENGTH]> {
    let DailyWeatherResponse {
        time,
        temperature_2m_min,
        temperature_2m_max,
        weather_code,
    } = response;

    let today = chrono::Local::now().date_naive();

    let mut forecast = vec![];
    for (((min, max), code), time) in temperature_2m_min
        .into_iter()
        .zip(temperature_2m_max)
        .zip(weather_code)
        .zip(time)
    {
        let code = WeatherCode::from(code);
        let date = chrono::Local
            .timestamp_opt(time, 0)
            .single()
            .context("invalid unix timestamp")?
            .date_naive();
        if date > today {
            forecast.push(WeatherOnDay {
                unix_seconds: time,
                temperature_min: min,
                temperature_max: max,
                code,
            });
        }
        if forecast.len() == 6 {
            break;
        }
    }

    forecast.try_into().map_err(|v: Vec<WeatherOnDay>| {
        anyhow::anyhow!("wrong size: {} vs {DAILY_WEATHER_FORECAST_LENGTH}", v.len())
    })
}

#[repr(C)]
pub struct WeatherOnHour {
    pub unix_seconds: i64,
    pub temperature: f32,
    pub code: WeatherCode,
}

impl core::fmt::Debug for WeatherOnHour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} - {} - {:?}",
            self.unix_seconds, self.temperature, self.code
        )
    }
}

#[repr(C)]
pub struct WeatherOnDay {
    pub unix_seconds: i64,
    pub temperature_min: f32,
    pub temperature_max: f32,
    pub code: WeatherCode,
}

impl core::fmt::Debug for WeatherOnDay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} - {}..{} - {:?}",
            self.unix_seconds, self.temperature_min, self.temperature_max, self.code
        )
    }
}
