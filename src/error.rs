use crate::{
    config::ConfigError,
    modules::{CpuError, MemoryError, NMError, NiriError, PWError, TrayError},
};
use rustix::io::Errno;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy)]
pub enum IoError {
    #[error("failed to {op}: {errno:?}")]
    FailedTo { op: &'static str, errno: Errno },
    #[error("wrong satisfy for state: {satisfy:?} {state}")]
    WrongSatisfy {
        state: &'static str,
        satisfy: &'static str,
    },
    #[error("EOF")]
    EofError,

    #[error(transparent)]
    ConfigError(#[from] ConfigError),

    #[error(transparent)]
    CpuError(#[from] CpuError),
    #[error(transparent)]
    MemoryError(#[from] MemoryError),
    #[error(transparent)]
    NiriError(#[from] NiriError),
    #[error(transparent)]
    NMError(#[from] NMError),
    #[error(transparent)]
    PWError(#[from] PWError),
    #[error(transparent)]
    TrayError(#[from] TrayError),
}
