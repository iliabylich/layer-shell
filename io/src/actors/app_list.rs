use crate::Event;
use anyhow::Result;
use futures::{pin_mut, StreamExt};
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("{:?}", err);
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let app_list_stream = layer_shell_app_list::connect().await;
    pin_mut!(app_list_stream);

    while let Some(event) = app_list_stream.next().await {
        if let Err(err) = tx.send(Event::from(event)) {
            log::error!("Failed to send event: {:?}", err);
        }
    }

    Ok(())
}

impl From<layer_shell_app_list::Event> for Event {
    fn from(e: layer_shell_app_list::Event) -> Self {
        match e {
            layer_shell_app_list::Event::AppList(e) => Self::AppList(e),
        }
    }
}
