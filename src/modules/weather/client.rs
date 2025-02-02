use anyhow::{Context, Result};
use serde::Deserialize;
use ureq::Agent;

pub(crate) fn get_weather(agent: &Agent) -> Result<Response> {
    agent
        .get("https://api.open-meteo.com/v1/forecast")
        .query("latitude", "52.2298")
        .query("longitude", "21.0118")
        .query("current", CURRENT_FIELDS)
        .query("hourly", HOURLY_FIELDS)
        .query("daily", DAILY_FIELDS)
        .query("timezone", "Europe/Warsaw")
        .call()
        .context("failed to send a request")?
        .body_mut()
        .read_json::<Response>()
        .context("failed to parse JSON response")
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
