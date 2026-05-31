use anyhow::{Result, ensure};
use dbus::DBusSatisfy;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub(crate) enum Satisfy {
    Socket,
    Connect,
    Write,
    Read,
    Close,
    OpenAt,
}
const MAX: Satisfy = Satisfy::OpenAt;

impl From<Satisfy> for u8 {
    fn from(value: Satisfy) -> Self {
        value as Self
    }
}

impl TryFrom<u8> for Satisfy {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        ensure!(
            value <= MAX as u8,
            "received malformed Satisfy from io_uring: {value}"
        );
        unsafe { Ok(core::mem::transmute::<u8, Self>(value)) }
    }
}

impl From<Satisfy> for DBusSatisfy {
    fn from(satisfy: Satisfy) -> Self {
        match satisfy {
            Satisfy::Socket => Self::Socket,
            Satisfy::Connect => Self::Connect,
            Satisfy::Write => Self::Write,
            Satisfy::Read => Self::Read,
            _ => unreachable!(),
        }
    }
}
