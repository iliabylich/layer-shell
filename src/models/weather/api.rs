use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

pub(crate) async fn get_weather(client: &Client) -> Result<Response> {
    client
        .get("https://api.open-meteo.com/v1/forecast")
        .query(&[
            ("latitude", "52.2298"),
            ("longitude", "21.0118"),
            ("current", CURRENT_FIELDS),
            ("hourly", HOURLY_FIELDS),
            ("daily", DAILY_FIELDS),
            ("timezone", "Europe/Warsaw"),
        ])
        .send()
        .await
        .context("failed to build a request")?
        .json::<Response>()
        .await
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
