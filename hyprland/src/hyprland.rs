use crate::{Event, task::Task, writer::Writer};
use anyhow::Result;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

pub struct Hyprland {
    rx: Receiver<Event>,
    handle: JoinHandle<()>,
}

impl Hyprland {
    pub fn new() -> Self {
        let (rx, handle) = Task::spawn();
        Self { rx, handle }
    }

    pub fn abort(&self) {
        self.handle.abort();
    }

    pub async fn recv(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub async fn hyprctl_dispatch(&mut self, cmd: impl AsRef<str>) -> Result<()> {
        Writer::dispatch(cmd).await
    }
}
