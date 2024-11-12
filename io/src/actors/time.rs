use crate::Event;
use anyhow::{Context, Result};
use chrono::Local;
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    loop {
        if let Err(err) = tick(&tx) {
            log::error!("{:?}", err);
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

fn tick(tx: &Sender<Event>) -> Result<()> {
    let now = Local::now();
    tx.send(Event::Time {
        time: now.format("%H:%M:%S").to_string(),
        date: now.format("%Y %B %e").to_string(),
    })
    .context("failed to send event")?;
    Ok(())
}
