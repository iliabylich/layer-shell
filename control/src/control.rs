use crate::{event::Event, task::Task};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

pub struct Control {
    rx: Receiver<Event>,
    handle: JoinHandle<()>,
}

impl Control {
    pub fn new() -> Self {
        let (rx, handle) = Task::spawn();
        Self { rx, handle }
    }

    pub async fn recv(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub async fn abort(&self) {
        self.handle.abort();
    }
}
