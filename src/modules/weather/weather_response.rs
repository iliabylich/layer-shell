use super::WeatherCode;
use crate::{
    Event,
    external::{localtime_r, time as external_time, tm},
    sansio::HttpResponse,
    utils::get_json,
};
use alloc::{vec, vec::Vec};
use anyhow::{Context as _, Result, ensure};
use microjson::JSONValue;

#[derive(Debug)]
pub(crate) struct WeatherResponse {
    pub(crate) current: CurrentWeatherResponse,
    pub(crate) hourly: HourlyWeatherResponse,
    pub(crate) daily: DailyWeatherResponse,
}
impl WeatherResponse {
    fn from_json(json: JSONValue) -> Result<Self> {
        Ok(Self {
            current: CurrentWeatherResponse::from_json(get_json!(json, "current"))?,
            hourly: HourlyWeatherResponse::from_json(get_json!(json, "hourly"))?,
            daily: DailyWeatherResponse::from_json(get_json!(json, "daily"))?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct CurrentWeatherResponse {
    pub(crate) temperature_2m: f32,
    pub(crate) weather_code: u32,
}
impl CurrentWeatherResponse {
    fn from_json(json: JSONValue) -> Result<Self> {
        Ok(Self {
            temperature_2m: get_json!(json, "temperature_2m", read_float),
            weather_code: u32::try_from(get_json!(json, "weather_code", read_integer))
                .context("not u32")?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct HourlyWeatherResponse {
    pub(crate) time: Vec<i64>,
    pub(crate) temperature_2m: Vec<f32>,
    pub(crate) weather_code: Vec<u32>,
}
impl HourlyWeatherResponse {
    fn from_json(json: JSONValue) -> Result<Self> {
        Ok(Self {
            time: json_value_to_vec_of_i64(get_json!(json, "time"))?,
            temperature_2m: json_value_to_vec_of_f32(get_json!(json, "temperature_2m"))?,
            weather_code: json_value_to_vec_of_u32(get_json!(json, "weather_code"))?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct DailyWeatherResponse {
    pub(crate) time: Vec<i64>,
    pub(crate) temperature_2m_min: Vec<f32>,
    pub(crate) temperature_2m_max: Vec<f32>,
    pub(crate) weather_code: Vec<u32>,
}
impl DailyWeatherResponse {
    fn from_json(json: JSONValue) -> Result<Self> {
        Ok(Self {
            time: json_value_to_vec_of_i64(get_json!(json, "time"))?,
            temperature_2m_min: json_value_to_vec_of_f32(get_json!(json, "temperature_2m_min"))?,
            temperature_2m_max: json_value_to_vec_of_f32(get_json!(json, "temperature_2m_max"))?,
            weather_code: json_value_to_vec_of_u32(get_json!(json, "weather_code"))?,
        })
    }
}

fn json_value_to_vec_of_f32(json: JSONValue) -> Result<Vec<f32>> {
    json.iter_array()
        .map_err(|err| anyhow::anyhow!(err))?
        .map(|e| e.read_float().map_err(|err| anyhow::anyhow!(err)))
        .collect()
}
fn json_value_to_vec_of_i64(json: JSONValue) -> Result<Vec<i64>> {
    json.iter_array()
        .map_err(|err| anyhow::anyhow!(err))?
        .map(|e| {
            let v = e.read_integer().map_err(|err| anyhow::anyhow!(err))?;
            i64::try_from(v).context("not i64")
        })
        .collect()
}
fn json_value_to_vec_of_u32(json: JSONValue) -> Result<Vec<u32>> {
    json.iter_array()
        .map_err(|err| anyhow::anyhow!(err))?
        .map(|e| {
            let v = e.read_integer().map_err(|err| anyhow::anyhow!(err))?;
            u32::try_from(v).context("not i64")
        })
        .collect()
}

impl WeatherResponse {
    pub(crate) fn parse(response: &HttpResponse) -> Result<Self> {
        ensure!(response.status == 200);
        let json = JSONValue::load(&response.body);
        Self::from_json(json).context("malformed JSON output")
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

        Ok(Self::Weather {
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
    let mut now = 0;
    unsafe { external_time(&raw mut now) };

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

    let mut now = 0;
    unsafe { external_time(&raw mut now) };

    let mut forecast = vec![];
    for (((min, max), code), time) in temperature_2m_min
        .into_iter()
        .zip(temperature_2m_max)
        .zip(weather_code)
        .zip(time)
    {
        let code = WeatherCode::from(code);
        if is_date_greater_than_or_eq(time, now) {
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

fn is_date_greater_than_or_eq(lhs: i64, rhs: i64) -> bool {
    unsafe {
        let mut lhs_tm: tm = core::mem::zeroed();
        let mut rhs_tm: tm = core::mem::zeroed();

        if localtime_r(&raw const lhs, &raw mut lhs_tm).is_null() {
            return false;
        }

        if localtime_r(&raw const rhs, &raw mut rhs_tm).is_null() {
            return false;
        }

        (lhs_tm.tm_year, lhs_tm.tm_mon, lhs_tm.tm_mday)
            >= (rhs_tm.tm_year, rhs_tm.tm_mon, rhs_tm.tm_mday)
    }
}
