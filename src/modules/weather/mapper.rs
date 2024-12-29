use crate::{
    event::{WeatherOnDay, WeatherOnHour},
    modules::weather::{
        client::{CurrentResponse, DailyResponse, HourlyResponse},
        WeatherCode,
    },
    Event,
};
use anyhow::{Context as _, Result};
use chrono::{NaiveDate, NaiveDateTime};

pub(crate) fn map_current(current: CurrentResponse) -> Event {
    Event::CurrentWeather {
        temperature: current.temperature_2m,
        code: map_code(current.weather_code),
    }
}

pub(crate) fn map_forecast(hourly: HourlyResponse, daily: DailyResponse) -> Result<Event> {
    let hourly = map_hourly(hourly)?;
    let daily = map_daily(daily)?;

    Ok(Event::ForecastWeather {
        hourly: hourly.into(),
        daily: daily.into(),
    })
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
        let code = map_code(code);
        let time = NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M")
            .context("invalid date format")?;
        if time > now {
            hourly.push(WeatherOnHour {
                hour: time.format("%H").to_string().into(),
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
        let code = map_code(code);
        let date = NaiveDate::parse_from_str(&time, "%Y-%m-%d").context("invalid date format")?;
        if date > today {
            daily.push(WeatherOnDay {
                day: date.format("%m-%d").to_string().into(),
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

fn map_code(value: u32) -> WeatherCode {
    use crate::modules::weather::{
        Drizzle, Fog, FreezingDrizzle, FreezingRain, Rain, RainShowers, SnowFall, SnowShowers,
        ThunderstormWithHail,
    };

    match value {
        0 => WeatherCode::ClearSky,
        1 => WeatherCode::MainlyClear,
        2 => WeatherCode::PartlyCloudy,
        3 => WeatherCode::Overcast,
        45 => WeatherCode::Fog(Fog::Normal),
        48 => WeatherCode::Fog(Fog::DepositingRime),
        51 => WeatherCode::Drizzle(Drizzle::Light),
        53 => WeatherCode::Drizzle(Drizzle::Moderate),
        55 => WeatherCode::Drizzle(Drizzle::Dense),
        56 => WeatherCode::FreezingDrizzle(FreezingDrizzle::Light),
        57 => WeatherCode::FreezingDrizzle(FreezingDrizzle::Dense),
        61 => WeatherCode::Rain(Rain::Slight),
        63 => WeatherCode::Rain(Rain::Moderate),
        65 => WeatherCode::Rain(Rain::Heavy),
        66 => WeatherCode::FreezingRain(FreezingRain::Light),
        67 => WeatherCode::FreezingRain(FreezingRain::Heavy),
        71 => WeatherCode::SnowFall(SnowFall::Slight),
        73 => WeatherCode::SnowFall(SnowFall::Moderate),
        75 => WeatherCode::SnowFall(SnowFall::Heavy),
        77 => WeatherCode::SnowGrains,
        80 => WeatherCode::RainShowers(RainShowers::Slight),
        81 => WeatherCode::RainShowers(RainShowers::Moderate),
        82 => WeatherCode::RainShowers(RainShowers::Violent),
        85 => WeatherCode::SnowShowers(SnowShowers::Slight),
        86 => WeatherCode::SnowShowers(SnowShowers::Heavy),
        95 => WeatherCode::Thunderstorm,
        96 => WeatherCode::ThunderstormWithHail(ThunderstormWithHail::Sight),
        99 => WeatherCode::ThunderstormWithHail(ThunderstormWithHail::Heavy),
        _ => WeatherCode::Unknown,
    }
}
