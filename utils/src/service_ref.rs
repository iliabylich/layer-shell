use anyhow::{Context as _, Result};
use tokio::{
    sync::{mpsc::UnboundedReceiver, oneshot::Sender},
    task::JoinHandle,
};

pub struct ServiceRef<E> {
    handle: JoinHandle<()>,
    rx: UnboundedReceiver<E>,
    exit: Sender<()>,
}

impl<E> ServiceRef<E> {
    pub(crate) fn new(handle: JoinHandle<()>, rx: UnboundedReceiver<E>, exit: Sender<()>) -> Self {
        Self { handle, rx, exit }
    }

    pub async fn recv(&mut self) -> Option<E> {
        self.rx.recv().await
    }

    pub async fn stop(self) -> Result<()> {
        if self.exit.send(()).is_ok() {
            self.handle
                .await
                .context("failed to wait for task completion")
        } else {
            self.handle.abort();
            Ok(())
        }
    }
}
