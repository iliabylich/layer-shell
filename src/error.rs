#[derive(Debug, Clone, Copy)]
pub struct IoError;

macro_rules! impl_from {
    ($t:tt) => {
        impl From<$t> for IoError {
            fn from(_: $t) -> Self {
                Self
            }
        }
    };
}

impl_from!(());
