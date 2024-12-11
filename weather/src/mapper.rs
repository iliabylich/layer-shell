use crate::{
    client::{CurrentResponse, DailyResponse, HourlyResponse},
    event::{WeatherOnDay, WeatherOnHour},
    Code, CurrentWeather, Event, ForecastWeather,
};
use anyhow::{Context as _, Result};
use chrono::{NaiveDate, NaiveDateTime};

pub(crate) fn map_current(current: CurrentResponse) -> Event {
    Event::CurrentWeather(CurrentWeather {
        temperature: current.temperature_2m,
        code: map_code(current.weather_code),
    })
}

pub(crate) fn map_forecast(hourly: HourlyResponse, daily: DailyResponse) -> Result<Event> {
    let hourly = map_hourly(hourly)?;
    let daily = map_daily(daily)?;

    Ok(Event::ForecastWeather(ForecastWeather { hourly, daily }))
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
        let code = map_code(code);
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

fn map_code(value: u32) -> Code {
    use crate::{
        Drizzle, Fog, FreezingDrizzle, FreezingRain, Rain, RainShowers, SnowFall, SnowShowers,
        ThunderstormWithHail,
    };

    match value {
        0 => Code::ClearSky,
        1 => Code::MainlyClear,
        2 => Code::PartlyCloudy,
        3 => Code::Overcast,
        45 => Code::Fog(Fog::Normal),
        48 => Code::Fog(Fog::DepositingRime),
        51 => Code::Drizzle(Drizzle::Light),
        53 => Code::Drizzle(Drizzle::Moderate),
        55 => Code::Drizzle(Drizzle::Dense),
        56 => Code::FreezingDrizzle(FreezingDrizzle::Light),
        57 => Code::FreezingDrizzle(FreezingDrizzle::Dense),
        61 => Code::Rain(Rain::Slight),
        63 => Code::Rain(Rain::Moderate),
        65 => Code::Rain(Rain::Heavy),
        66 => Code::FreezingRain(FreezingRain::Light),
        67 => Code::FreezingRain(FreezingRain::Heavy),
        71 => Code::SnowFall(SnowFall::Slight),
        73 => Code::SnowFall(SnowFall::Moderate),
        75 => Code::SnowFall(SnowFall::Heavy),
        77 => Code::SnowGrains,
        80 => Code::RainShowers(RainShowers::Slight),
        81 => Code::RainShowers(RainShowers::Moderate),
        82 => Code::RainShowers(RainShowers::Violent),
        85 => Code::SnowShowers(SnowShowers::Slight),
        86 => Code::SnowShowers(SnowShowers::Heavy),
        95 => Code::Thunderstorm,
        96 => Code::ThunderstormWithHail(ThunderstormWithHail::Sight),
        99 => Code::ThunderstormWithHail(ThunderstormWithHail::Heavy),
        _ => Code::Unknown,
    }
}
