use crate::dbus::decoder::Cursor;
use anyhow::{Context as _, Result};

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub(crate) struct Header {
    pub(crate) _endian: u8,
    pub(crate) message_type: u8,
    pub(crate) _flags: u8,
    pub(crate) _protocol_version: u8,
    pub(crate) body_len: u32,
    pub(crate) serial: u32,
    pub(crate) header_fields_len: u32,
}

impl Header {
    pub(crate) fn cut(cur: &mut Cursor<'_>) -> Result<Self> {
        let header = cur
            .take(core::mem::size_of::<Self>())
            .context("no Header")?;
        let header = unsafe { *header.as_ptr().cast::<Self>() };
        Ok(header)
    }
}
