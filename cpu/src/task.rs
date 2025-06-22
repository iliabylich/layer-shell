use std::time::Duration;

use crate::{Event, store::Store};
use anyhow::{Context as _, Result};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub(crate) struct Task {
    tx: Sender<Event>,
    store: Store,
}

impl Task {
    pub(crate) fn spawn() -> (Receiver<Event>, JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        let handle = tokio::spawn(async move {
            let mut task = Self {
                tx,
                store: Store::new(),
            };
            if let Err(err) = task.start().await {
                log::error!("{err:?}");
            }
        });
        (rx, handle)
    }

    async fn start(&mut self) -> Result<()> {
        loop {
            let usage_per_core = self.store.update()?;

            self.tx
                .send(Event { usage_per_core })
                .await
                .context("can't send event, channel is closed")?;

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
