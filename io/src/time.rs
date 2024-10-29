use crate::Event;
use anyhow::{Context, Result};
use chrono::Local;
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("Time model error: {}\n{}", err, err.backtrace());
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    loop {
        let now = Local::now();
        tx.send(Event::Time {
            time: now.format("%H:%M:%S").to_string(),
            date: now.format("%Y %B %e").to_string(),
        })
        .context("failed to send event")?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
