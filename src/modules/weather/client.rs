use anyhow::{bail, Context, Result};
use http_req::{request::Request, uri::Uri};
use serde::Deserialize;

pub(crate) fn get_weather() -> Result<Response> {
    let uri = format!("https://api.open-meteo.com/v1/forecast?latitude=52.2298&longitude=21.0118&current={CURRENT_FIELDS}&hourly={HOURLY_FIELDS}&daily={DAILY_FIELDS}&timezone=Europe/Warsaw");
    let uri = Uri::try_from(uri.as_str()).context("invalid URI")?;
    let mut req = Request::new(&uri);
    let mut body = vec![];
    let res = req.send(&mut body).context("failed to send request")?;
    let body = String::from_utf8(body).context("non-utf8 response body")?;

    if !res.status_code().is_success() {
        bail!(
            "Failed to send weather request\ncode: {:?}\nbody: {:?}",
            res.status_code(),
            body
        );
    }

    serde_json::from_str(&body).context("failed to parse response body")
}

#[derive(Deserialize, Debug)]
pub(crate) struct Response {
    pub(crate) current: CurrentResponse,
    pub(crate) hourly: HourlyResponse,
    pub(crate) daily: DailyResponse,
}

const CURRENT_FIELDS: &str = "temperature_2m,weather_code";
#[derive(Deserialize, Debug)]
pub(crate) struct CurrentResponse {
    pub(crate) temperature_2m: f32,
    pub(crate) weather_code: u32,
}

const HOURLY_FIELDS: &str = "temperature_2m,weather_code";
#[derive(Deserialize, Debug)]
pub(crate) struct HourlyResponse {
    pub(crate) time: Vec<String>,
    pub(crate) temperature_2m: Vec<f32>,
    pub(crate) weather_code: Vec<u32>,
}

const DAILY_FIELDS: &str = "temperature_2m_min,temperature_2m_max,weather_code";
#[derive(Deserialize, Debug)]
pub(crate) struct DailyResponse {
    pub(crate) time: Vec<String>,
    pub(crate) temperature_2m_min: Vec<f32>,
    pub(crate) temperature_2m_max: Vec<f32>,
    pub(crate) weather_code: Vec<u32>,
}
