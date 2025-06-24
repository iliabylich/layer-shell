use crate::{Emitter, ServiceRef, TaskCtx};
use anyhow::Result;

pub struct Service;

impl Service {
    pub fn start<E, Fut, F>(f: F) -> ServiceRef<E>
    where
        E: Send + std::fmt::Debug + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        F: Send + 'static + FnOnce(TaskCtx<E>) -> Fut,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<E>();
        let emitter = Emitter::new(tx);

        let (exit_tx, exit_rx) = tokio::sync::oneshot::channel::<()>();
        let task_ctx = TaskCtx {
            emitter,
            exit: exit_rx,
        };

        let handle = tokio::spawn(async move {
            if let Err(err) = (f)(task_ctx).await {
                log::error!("{err:?}");
            }
        });

        ServiceRef::new(handle, rx, exit_tx)
    }
}
