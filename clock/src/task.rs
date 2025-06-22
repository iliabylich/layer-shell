use crate::Event;
use anyhow::{Context as _, Result};
use std::time::Duration;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub(crate) struct Task {
    tx: Sender<Event>,
}

impl Task {
    pub(crate) fn spawn() -> (Receiver<Event>, JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::channel(256);

        let handle = tokio::spawn(async move {
            let task = Self { tx };
            if let Err(err) = task.start().await {
                log::error!("{err:?}");
            }
        });

        (rx, handle)
    }

    async fn start(self) -> Result<()> {
        loop {
            let now = chrono::Local::now();
            self.tx
                .send(Event {
                    time: now.format("%H:%M:%S | %b %e | %a").to_string(),
                })
                .await
                .context("failed to send event, channel is closed")?;

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
