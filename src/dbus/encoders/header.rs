use crate::dbus::encoders::EncodingBuffer;

pub(crate) struct HeaderEncoder;

impl HeaderEncoder {
    const LITTLE_ENDIAN: u8 = b'l';
    const PROTOCOL_VERSION: u8 = 1;

    pub(crate) fn encode(buf: &mut EncodingBuffer, message_type: u8, flags: u8, serial: u32) {
        buf.encode_u8(Self::LITTLE_ENDIAN);
        buf.encode_u8(message_type);
        buf.encode_u8(flags);
        buf.encode_u8(Self::PROTOCOL_VERSION);
        buf.encode_u32(0); // body len
        buf.encode_u32(serial);
    }
}
