use anyhow::{Result, bail};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Flags {
    pub(crate) byte: u8,
}

impl TryFrom<u8> for Flags {
    type Error = anyhow::Error;

    fn try_from(byte: u8) -> Result<Self> {
        match byte {
            0..=7 => Ok(Self { byte }),
            _ => bail!("flags must be in 0..=7 range"),
        }
    }
}

impl From<Flags> for u8 {
    fn from(flags: Flags) -> Self {
        flags.byte
    }
}

impl Flags {
    pub(crate) const NO_REPLY_EXPECTED: u8 = 0x1;
    pub(crate) const NO_AUTO_START: u8 = 0x2;
    pub(crate) const ALLOW_INTERACTIVE_AUTHORIZATION: u8 = 0x4;
}

impl std::fmt::Debug for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut start = false;
        if self.byte & Self::NO_REPLY_EXPECTED != 0 {
            write!(f, "NO_REPLY_EXPECTED")?;
            start = false;
        }
        if self.byte & Self::NO_AUTO_START != 0 {
            write!(f, "{}NO_AUTO_START", if start { "|" } else { "" })?;
            start = false;
        }
        if self.byte & Self::ALLOW_INTERACTIVE_AUTHORIZATION != 0 {
            write!(
                f,
                "{}ALLOW_INTERACTIVE_AUTHORIZATION",
                if start { "|" } else { "" }
            )?;
        }
        Ok(())
    }
}
