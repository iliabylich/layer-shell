use crate::{
    Event,
    event::{WeatherOnDay, WeatherOnHour},
    modules::weather::WeatherCode,
};
use anyhow::{Context as _, Result};
use chrono::{NaiveDate, NaiveDateTime};
use nanoserde::DeJson;

#[derive(DeJson, Debug)]
struct Response {
    current: CurrentResponse,
    hourly: HourlyResponse,
    daily: DailyResponse,
}

#[derive(DeJson, Debug)]
struct CurrentResponse {
    temperature_2m: f32,
    weather_code: u32,
}

#[derive(DeJson, Debug)]
struct HourlyResponse {
    time: Vec<String>,
    temperature_2m: Vec<f32>,
    weather_code: Vec<u32>,
}

#[derive(DeJson, Debug)]
struct DailyResponse {
    time: Vec<String>,
    temperature_2m_min: Vec<f32>,
    temperature_2m_max: Vec<f32>,
    weather_code: Vec<u32>,
}

pub(crate) fn map(response: String) -> Result<(Event, Event)> {
    let response: Response =
        DeJson::deserialize_json(&response).context("failed to parse response body")?;

    let current = map_current(response.current);
    let forecast = map_forecast(response.hourly, response.daily)?;
    Ok((current, forecast))
}

fn map_current(current: CurrentResponse) -> Event {
    Event::CurrentWeather {
        temperature: current.temperature_2m,
        code: WeatherCode::from(current.weather_code),
    }
}

fn map_forecast(hourly: HourlyResponse, daily: DailyResponse) -> Result<Event> {
    let hourly = map_hourly(hourly)?;
    let daily = map_daily(daily)?;

    Ok(Event::ForecastWeather { hourly, daily })
}

fn map_hourly(
    HourlyResponse {
        time,
        temperature_2m,
        weather_code,
    }: HourlyResponse,
) -> Result<Vec<WeatherOnHour>> {
    let now = chrono::Local::now().naive_local();

    let mut hourly = vec![];
    for ((temp, code), time) in temperature_2m.into_iter().zip(weather_code).zip(time) {
        let code = WeatherCode::from(code);
        let time = NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M")
            .context("invalid date format")?;

        if time > now {
            hourly.push(WeatherOnHour {
                hour: time.format("%H:%M").to_string(),
                temperature: temp,
                code,
            });
        }
        if hourly.len() == 10 {
            break;
        }
    }

    assert_eq!(hourly.len(), 10);
    Ok(hourly)
}

fn map_daily(
    DailyResponse {
        time,
        temperature_2m_min,
        temperature_2m_max,
        weather_code,
    }: DailyResponse,
) -> Result<Vec<WeatherOnDay>> {
    let today = chrono::Local::now().date_naive();

    let mut daily = vec![];
    for (((min, max), code), time) in temperature_2m_min
        .into_iter()
        .zip(temperature_2m_max)
        .zip(weather_code)
        .zip(time)
    {
        let code = WeatherCode::from(code);
        let date = NaiveDate::parse_from_str(&time, "%Y-%m-%d").context("invalid date format")?;
        if date > today {
            daily.push(WeatherOnDay {
                day: date.format("%b-%d").to_string(),
                temperature_min: min,
                temperature_max: max,
                code,
            });
        }
        if daily.len() == 6 {
            break;
        }
    }

    assert_eq!(daily.len(), 6);
    Ok(daily)
}
