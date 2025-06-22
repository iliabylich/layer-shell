use crate::{Event, task::Task};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

pub struct CPU {
    rx: Receiver<Event>,
    handle: JoinHandle<()>,
}

impl CPU {
    pub fn new() -> Self {
        let (rx, handle) = Task::spawn();
        Self { rx, handle }
    }

    pub async fn recv(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn abort(&self) {
        self.handle.abort();
    }
}
