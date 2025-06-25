use crate::response::Response;
use anyhow::Result;
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

    pub(crate) async fn get(&self) -> Result<Response> {
        let response = self
            .client
            .get("https://api.open-meteo.com/v1/forecast")
            .query(&[
                ("latitude", "52.2298"),
                ("longitude", "21.0118"),
                ("current", "temperature_2m,weather_code"),
                ("hourly", "temperature_2m,weather_code"),
                (
                    "daily",
                    "temperature_2m_min,temperature_2m_max,weather_code",
                ),
                ("timezone", "Europe/Warsaw"),
            ])
            .send()
            .await?
            .json::<Response>()
            .await?;
        Ok(response)
    }
}
