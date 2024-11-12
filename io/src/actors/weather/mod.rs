use crate::{
    event::{WeatherOnDay, WeatherOnHour},
    Event,
};
use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime};
use reqwest::Client;
use std::sync::mpsc::Sender;

mod api;
use api::{get_weather, CurrentResponse, DailyResponse, HourlyResponse, Response};
mod code;
pub use code::{
    Drizzle, Fog, FreezingDrizzle, FreezingRain, Rain, RainShowers, SnowFall, SnowShowers,
    ThunderstormWithHail, WeatherCode,
};

pub(crate) async fn spawn(tx: Sender<Event>) {
    let client = Client::new();

    loop {
        if let Err(err) = tick(&tx, &client).await {
            log::error!("{:?}", err);
        }
        tokio::time::sleep(std::time::Duration::from_secs(100)).await;
    }
}

async fn tick(tx: &Sender<Event>, client: &Client) -> Result<()> {
    let Response {
        current,
        hourly,
        daily,
    } = get_weather(client).await.context("failed to get weather")?;

    let current = map_current(current);
    tx.send(current)?;

    let forecast = map_forecast(hourly, daily).context("failed to map forecast")?;
    tx.send(forecast)?;

    Ok(())
}

fn map_current(current: CurrentResponse) -> Event {
    Event::WeatherCurrent {
        temperature: current.temperature_2m,
        code: WeatherCode::from(current.weather_code),
    }
}

fn map_forecast(hourly: HourlyResponse, daily: DailyResponse) -> Result<Event> {
    let hourly = map_hourly(hourly)?;
    let daily = map_daily(daily)?;

    Ok(Event::WeatherForecast { hourly, daily })
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
                hour: time.format("%H").to_string(),
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
                day: date.format("%m-%d").to_string(),
                temperature: min..max,
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
