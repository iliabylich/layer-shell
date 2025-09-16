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
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self { client })
    }

    async fn get_lat_lng(&self) -> Result<(String, String)> {
        #[derive(Debug, Deserialize)]
        struct Response {
            location: Vec<Location>,
        }
        #[derive(Debug, Deserialize)]
        struct Location {
            lat: f64,
            lng: f64,
            source: Source,
        }
        #[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
        enum Source {
            #[serde(rename = "freegeoip")]
            FreeGeoIP,
            #[serde(rename = "ipapi")]
            IpAPI,
            #[serde(rename = "ipwhois")]
            IpWhoIs,
        }
        let response = self
            .client
            .get("https://myip.ibylich.dev")
            .send()
            .await?
            .json::<Response>()
            .await?;

        let get = |source: Source| -> Option<(f64, f64)> {
            response
                .location
                .iter()
                .find(|loc| loc.source == source)
                .map(|loc| (loc.lat, loc.lng))
        };
        let (lat, lng) = get(Source::FreeGeoIP)
            .or_else(|| get(Source::IpAPI))
            .or_else(|| get(Source::IpWhoIs))
            .context("failed to get at least one location")?;

        Ok((lat.to_string(), lng.to_string()))
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
