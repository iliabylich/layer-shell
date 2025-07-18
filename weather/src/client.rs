use crate::response::Response;
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::time::Duration;

pub(crate) struct Client {
    client: reqwest::Client,
}

impl Client {
    pub(crate) fn new() -> Result<Self> {
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(2))
            .read_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(5))
            .build()?;

        Ok(Self { client })
    }

    async fn get_lat_lng(&self) -> Result<(String, String)> {
        #[derive(Deserialize)]
        struct Location {
            lat: f64,
            lon: f64,
        }
        let location = self
            .client
            .get("http://ip-api.com/json/?fields=lon,lat")
            .send()
            .await?
            .json::<Location>()
            .await?;

        Ok((format!("{}", location.lat), format!("{}", location.lon)))
    }

    async fn get_tz(&self) -> Result<String> {
        tokio::fs::read_to_string("/etc/timezone")
            .await
            .context("failed to read /etc/timezone")
            .map(|tz| tz.trim().to_string())
    }

    pub(crate) async fn get(&self) -> Result<Response> {
        let (lat, lng) = self.get_lat_lng().await?;
        let tz = self.get_tz().await?;

        let response = self
            .client
            .get("https://api.open-meteo.com/v1/forecast")
            .query(&[
                ("latitude", lat.as_str()),
                ("longitude", lng.as_str()),
                ("current", "temperature_2m,weather_code"),
                ("hourly", "temperature_2m,weather_code"),
                (
                    "daily",
                    "temperature_2m_min,temperature_2m_max,weather_code",
                ),
                ("timezone", tz.as_str()),
            ])
            .send()
            .await?
            .json::<Response>()
            .await?;
        Ok(response)
    }
}
