use std::time::Duration;

use crate::{Event, response::Response};
use anyhow::Result;
use tokio::time::Interval;
use utils::{TaskCtx, service};

struct Task {
    ctx: TaskCtx<Event>,
    timer: Interval,
    client: reqwest::Client,
}

impl Task {
    async fn start(ctx: TaskCtx<Event>) -> Result<()> {
        Self {
            ctx,
            timer: tokio::time::interval(Duration::from_secs(120)),
            client: reqwest::ClientBuilder::new()
                .connect_timeout(Duration::from_secs(2))
                .read_timeout(Duration::from_secs(2))
                .timeout(Duration::from_secs(5))
                .build()?,
        }
        .r#loop()
        .await
    }

    async fn r#loop(mut self) -> Result<()> {
        loop {
            tokio::select! {
                _ = self.timer.tick() => self.tick().await?,

                _ = &mut self.ctx.exit => {
                    log::info!(target: "Weather", "exiting...");
                    return Ok(())
                }
            }
        }
    }

    async fn tick(&mut self) -> Result<()> {
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
        let events = response.into_events()?;

        for event in events {
            self.ctx.emitter.emit(event)?;
        }

        Ok(())
    }
}

service!(Weather, Event, Task::start);
