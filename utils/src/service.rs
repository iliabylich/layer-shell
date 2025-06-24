use crate::{Emitter, ServiceRef};
use anyhow::Result;

pub struct Service;

impl Service {
    pub fn start<E, Fut, F>(f: F) -> ServiceRef<E>
    where
        E: Send + std::fmt::Debug + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        F: Send + 'static + FnOnce(Emitter<E>, tokio::sync::oneshot::Receiver<()>) -> Fut,
    {
        let (tx, rx) = tokio::sync::mpsc::channel::<E>(256);
        let emitter = Emitter::new(tx);

        let (exit_tx, exit_rx) = tokio::sync::oneshot::channel::<()>();

        let handle = tokio::spawn(async move {
            if let Err(err) = (f)(emitter, exit_rx).await {
                log::error!("{err:?}");
            }
        });

        ServiceRef::new(handle, rx, exit_tx)
    }
}
