use crate::dbus::{
    encoders::EncodingBuffer,
    types::{CompleteType, Signature},
};

pub(crate) struct SignatureEncoder;

impl SignatureEncoder {
    pub(crate) fn encode_complete_type(buf: &mut EncodingBuffer, complete_type: &CompleteType) {
        match complete_type {
            CompleteType::Byte => buf.encode_u8(b'y'),
            CompleteType::Bool => buf.encode_u8(b'b'),
            CompleteType::Int16 => buf.encode_u8(b'n'),
            CompleteType::UInt16 => buf.encode_u8(b'q'),
            CompleteType::Int32 => buf.encode_u8(b'i'),
            CompleteType::UInt32 => buf.encode_u8(b'u'),
            CompleteType::Int64 => buf.encode_u8(b'x'),
            CompleteType::UInt64 => buf.encode_u8(b't'),
            CompleteType::Double => buf.encode_u8(b'd'),
            CompleteType::UnixFD => buf.encode_u8(b'h'),

            CompleteType::String => buf.encode_u8(b's'),
            CompleteType::ObjectPath => buf.encode_u8(b'o'),
            CompleteType::Signature => buf.encode_u8(b'g'),

            CompleteType::Struct(fields) => {
                buf.encode_u8(b'(');
                for field in fields {
                    Self::encode_complete_type(buf, field);
                }
                buf.encode_u8(b')');
            }
            CompleteType::Array(item) => {
                buf.encode_u8(b'a');
                Self::encode_complete_type(buf, item);
            }
            CompleteType::DictEntry(key, value) => {
                buf.encode_u8(b'{');
                Self::encode_complete_type(buf, key);
                Self::encode_complete_type(buf, value);
                buf.encode_u8(b'}');
            }
            CompleteType::Variant => {
                buf.encode_u8(b'v');
            }
        }
    }

    pub(crate) fn encode_signature(buf: &mut EncodingBuffer, signature: &Signature) {
        for complete_type in &signature.items {
            Self::encode_complete_type(buf, complete_type);
        }
    }
}
