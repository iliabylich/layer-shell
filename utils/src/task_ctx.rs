use crate::Emitter;

pub struct TaskCtx<E>
where
    E: std::fmt::Debug,
{
    pub emitter: Emitter<E>,
    pub exit: tokio::sync::oneshot::Receiver<()>,
}
