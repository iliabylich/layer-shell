use anyhow::{Result, anyhow};
use tokio::sync::mpsc::Sender;

pub struct Emitter<E>
where
    E: std::fmt::Debug,
{
    tx: Sender<E>,
}

impl<E> Emitter<E>
where
    E: std::fmt::Debug,
{
    pub(crate) fn new(tx: Sender<E>) -> Self {
        Self { tx }
    }

    pub async fn emit(&self, event: E) -> Result<()> {
        self.tx
            .send(event)
            .await
            .map_err(|err| anyhow!("failed to send event {:?}; channel is closed", err.0))
    }
}
