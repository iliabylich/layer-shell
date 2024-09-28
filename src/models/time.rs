use crate::models::Event;
use anyhow::{Context, Result};
use chrono::Local;
use tokio::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        eprintln!("TIme model error:\n{}\n{}", err, err.backtrace());
        return;
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    loop {
        let now = Local::now();
        tx.send(Event::Time {
            time: now.format("%H:%M:%S").to_string(),
            date: now.format("%Y %B %e").to_string(),
        })
        .await
        .context("failed to send event")?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
