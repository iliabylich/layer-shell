use anyhow::{Result, ensure};

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
        value as u8
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

impl From<Satisfy> for mini_sansio_dbus::Satisfy {
    fn from(satisfy: Satisfy) -> Self {
        match satisfy {
            Satisfy::Socket => mini_sansio_dbus::Satisfy::Socket,
            Satisfy::Connect => mini_sansio_dbus::Satisfy::Connect,
            Satisfy::Write => mini_sansio_dbus::Satisfy::Write,
            Satisfy::Read => mini_sansio_dbus::Satisfy::Read,
            _ => unreachable!(),
        }
    }
}
