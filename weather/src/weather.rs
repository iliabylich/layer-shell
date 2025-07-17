use crate::{WeatherEvent, client::Client};
use anyhow::Result;
use module::Module;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

pub struct Weather {
    etx: UnboundedSender<WeatherEvent>,
    token: CancellationToken,
}

#[async_trait::async_trait]
impl Module for Weather {
    const NAME: &str = "Weather";

    type Event = WeatherEvent;
    type Command = ();
    type Ctl = ();

    fn new(
        etx: UnboundedSender<Self::Event>,
        _: UnboundedReceiver<Self::Command>,
        token: CancellationToken,
    ) -> Self {
        Self { etx, token }
    }

    async fn start(&mut self) -> Result<()> {
        let mut timer = tokio::time::interval(Duration::from_secs(120));
        let client = Client::new()?;

        loop {
            tokio::select! {
                _ = timer.tick() => {
                    let events = get_weather(&client).await?;
                    for event in events {
                        self.etx.send(event)?;
                    }
                }

                _ = self.token.cancelled() => {
                    log::info!(target: "Weather", "exiting...");
                    return Ok(())
                }
            }
        }
    }
}

async fn get_weather(client: &Client) -> Result<Vec<WeatherEvent>> {
    let response = client.get().await?;
    response.into_events()
}
