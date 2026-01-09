use crate::dbus::{
    decoders::DecodingBuffer,
    types::{Flags, Header, MessageType},
};
use anyhow::Result;

pub(crate) struct HeaderDecoder;

impl HeaderDecoder {
    pub(crate) const LENGTH: usize = 12;

    pub(crate) fn decode(buffer: &mut DecodingBuffer<'_>) -> Result<Header> {
        let _endian = buffer.next_u8()?;
        let message_type = MessageType::from(buffer.next_u8()?);
        let flags = Flags::try_from(buffer.next_u8()?)?;
        let _protocol_version = buffer.next_u8();
        let body_len = buffer.next_u32()? as usize;
        let serial = buffer.next_u32()?;

        Ok(Header {
            message_type,
            flags,
            body_len,
            serial,
        })
    }
}
