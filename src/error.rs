use crate::{
    config::ConfigError,
    modules::{CpuError, MemoryError, NMError, NiriError, PWError, TrayError},
};
use libc::__errno_location;
use rustix::io::Errno;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy)]
pub(crate) enum IoError {
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

impl IoError {
    pub(crate) fn new_failed_to(op: &'static str) -> Self {
        Self::FailedTo {
            op,
            errno: Errno::from_raw_os_error(unsafe { *__errno_location() }),
        }
    }
}
