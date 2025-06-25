use crate::{
    CurrentWeatherEvent, DailyWeatherForecastEvent, HourlyWeatherForecastEvent, WeatherCode,
    WeatherEvent,
    event::{WeatherOnDay, WeatherOnHour},
};
use anyhow::{Context as _, Result, ensure};
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct Response {
    pub(crate) current: CurrentResponse,
    pub(crate) hourly: HourlyResponse,
    pub(crate) daily: DailyResponse,
}

#[derive(Deserialize, Debug)]
pub(crate) struct CurrentResponse {
    pub(crate) temperature_2m: f32,
    pub(crate) weather_code: u32,
}

#[derive(Deserialize, Debug)]
pub(crate) struct HourlyResponse {
    pub(crate) time: Vec<String>,
    pub(crate) temperature_2m: Vec<f32>,
    pub(crate) weather_code: Vec<u32>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct DailyResponse {
    pub(crate) time: Vec<String>,
    pub(crate) temperature_2m_min: Vec<f32>,
    pub(crate) temperature_2m_max: Vec<f32>,
    pub(crate) weather_code: Vec<u32>,
}

impl Response {
    pub(crate) fn into_events(self) -> Result<Vec<WeatherEvent>> {
        let Self {
            current,
            hourly,
            daily,
        } = self;

        Ok(vec![
            WeatherEvent::from(current),
            WeatherEvent::try_from(hourly)?,
            WeatherEvent::try_from(daily)?,
        ])
    }
}

impl From<CurrentResponse> for WeatherEvent {
    fn from(res: CurrentResponse) -> Self {
        WeatherEvent::CurrentWeather(CurrentWeatherEvent {
            temperature: res.temperature_2m,
            code: WeatherCode::from(res.weather_code),
        })
    }
}

impl TryFrom<HourlyResponse> for WeatherEvent {
    type Error = anyhow::Error;

    fn try_from(response: HourlyResponse) -> Result<Self> {
        let HourlyResponse {
            time,
            temperature_2m,
            weather_code,
        } = response;
        let now = chrono::Local::now().naive_local();

        let mut forecast = vec![];
        for ((temp, code), time) in temperature_2m.into_iter().zip(weather_code).zip(time) {
            let code = WeatherCode::from(code);
            let time = NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M")
                .context("invalid date format")?;

            if time > now {
                forecast.push(WeatherOnHour {
                    hour: time.format("%H:%M").to_string().into(),
                    temperature: temp,
                    code,
                });
            }
            if forecast.len() == 10 {
                break;
            }
        }

        ensure!(forecast.len() == 10, "bug");
        Ok(Self::HourlyWeatherForecast(HourlyWeatherForecastEvent {
            forecast: forecast.into(),
        }))
    }
}

impl TryFrom<DailyResponse> for WeatherEvent {
    type Error = anyhow::Error;

    fn try_from(response: DailyResponse) -> Result<Self> {
        let DailyResponse {
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
            let date =
                NaiveDate::parse_from_str(&time, "%Y-%m-%d").context("invalid date format")?;
            if date > today {
                forecast.push(WeatherOnDay {
                    day: date.format("%b-%d").to_string().into(),
                    temperature_min: min,
                    temperature_max: max,
                    code,
                });
            }
            if forecast.len() == 6 {
                break;
            }
        }

        ensure!(forecast.len() == 6, "bug");
        Ok(WeatherEvent::DailyWeatherForecast(
            DailyWeatherForecastEvent {
                forecast: forecast.into(),
            },
        ))
    }
}
