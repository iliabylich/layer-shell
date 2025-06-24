use anyhow::{Result, anyhow};
use tokio::sync::mpsc::UnboundedSender;

pub struct Emitter<E>
where
    E: std::fmt::Debug,
{
    tx: UnboundedSender<E>,
}

impl<E> Emitter<E>
where
    E: std::fmt::Debug,
{
    pub(crate) fn new(tx: UnboundedSender<E>) -> Self {
        Self { tx }
    }

    pub fn emit(&self, event: E) -> Result<()> {
        self.tx
            .send(event)
            .map_err(|err| anyhow!("failed to send event {:?}; channel is closed", err.0))
    }
}
