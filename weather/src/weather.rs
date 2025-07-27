use crate::{WeatherEvent, client::Client};
use anyhow::Result;
use module::{Module, TimerSubscriber};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

pub struct Weather {
    etx: UnboundedSender<WeatherEvent>,
    token: CancellationToken,
    timer: TimerSubscriber,
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
        timer: TimerSubscriber,
    ) -> Self {
        Self {
            etx,
            token,
            timer: timer.with_cycle(120),
        }
    }

    async fn start(&mut self) -> Result<()> {
        let client = Client::new()?;

        loop {
            tokio::select! {
                _ = self.timer.recv() => {
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
