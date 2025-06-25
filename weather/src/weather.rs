use crate::{WeatherEvent, client::Client};
use anyhow::Result;
use futures::Stream;
use pin_project_lite::pin_project;
use std::time::Duration;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

pin_project! {
    pub struct Weather {
        #[pin]
        rx: UnboundedReceiver<WeatherEvent>,
        #[pin]
        handle: JoinHandle<()>
    }
}

impl Weather {
    pub fn new(token: CancellationToken) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<WeatherEvent>();
        let handle = tokio::task::spawn(async move {
            if let Err(err) = Self::r#loop(tx, token).await {
                log::error!("Network crashed: {err:?}");
            }
        });
        Self { rx, handle }
    }

    async fn r#loop(tx: UnboundedSender<WeatherEvent>, token: CancellationToken) -> Result<()> {
        let mut timer = tokio::time::interval(Duration::from_secs(120));
        let client = Client::new()?;

        loop {
            tokio::select! {
                _ = timer.tick() => {
                    let events = get_weather(&client).await;
                    for event in events {
                        tx.send(event)?;
                    }
                }

                _ = token.cancelled() => {
                    log::info!(target: "Weather", "exiting...");
                    return Ok(())
                }
            }
        }
    }
}

impl Stream for Weather {
    type Item = WeatherEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().rx.poll_recv(cx)
    }
}

impl Future for Weather {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Err(err) = futures::ready!(self.project().handle.poll(cx)) {
            log::error!("failed to await Weather task: {err:?}")
        }
        std::task::Poll::Ready(())
    }
}

async fn get_weather(client: &Client) -> Vec<WeatherEvent> {
    if let Ok(response) = client.get().await {
        if let Ok(events) = response.into_events() {
            return events;
        }
    }
    vec![]
}
