use anyhow::{Result, bail};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageType {
    Invalid = 0,
    MethodCall = 1,
    MethodReturn = 2,
    Error = 3,
    Signal = 4,
}

impl MessageType {
    pub(crate) fn from_u8(value: u8) -> Result<Self> {
        let ty = match value {
            0 => Self::Invalid,
            1 => Self::MethodCall,
            2 => Self::MethodReturn,
            3 => Self::Error,
            4 => Self::Signal,
            other => bail!("unknown message type: {other}"),
        };
        Ok(ty)
    }
}
