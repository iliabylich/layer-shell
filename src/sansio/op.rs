use anyhow::{Result, bail};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub(crate) enum Op {
    Socket,
    Connect,
    Write,
    Read,
    Close,
    OpenAt,
    Accept,
}

impl From<Op> for u8 {
    fn from(value: Op) -> Self {
        value as Self
    }
}

impl TryFrom<u8> for Op {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        if value == Self::Socket as u8 {
            Ok(Self::Socket)
        } else if value == Self::Connect as u8 {
            Ok(Self::Connect)
        } else if value == Self::Write as u8 {
            Ok(Self::Write)
        } else if value == Self::Read as u8 {
            Ok(Self::Read)
        } else if value == Self::Close as u8 {
            Ok(Self::Close)
        } else if value == Self::OpenAt as u8 {
            Ok(Self::OpenAt)
        } else if value == Self::Accept as u8 {
            Ok(Self::Accept)
        } else {
            bail!("can't convert {value} to SatisfyKind")
        }
    }
}
