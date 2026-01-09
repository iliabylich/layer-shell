use crate::dbus::{
    decoders::DecodingBuffer,
    types::{CompleteType, Signature},
};
use anyhow::{Result, bail, ensure};

pub(crate) struct SignatureDecoder;

impl SignatureDecoder {
    pub(crate) fn decode_complete_type(buf: &mut DecodingBuffer) -> Result<CompleteType> {
        match buf.next_u8()? {
            b'y' => Ok(CompleteType::Byte),
            b'b' => Ok(CompleteType::Bool),
            b'n' => Ok(CompleteType::Int16),
            b'q' => Ok(CompleteType::UInt16),
            b'i' => Ok(CompleteType::Int32),
            b'u' => Ok(CompleteType::UInt32),
            b'x' => Ok(CompleteType::Int64),
            b't' => Ok(CompleteType::UInt64),
            b'd' => Ok(CompleteType::Double),
            b'h' => Ok(CompleteType::UnixFD),

            b's' => Ok(CompleteType::String),
            b'o' => Ok(CompleteType::ObjectPath),
            b'g' => Ok(CompleteType::Signature),

            b'(' => {
                let mut fields = vec![];
                while buf.peek().is_some_and(|b| b != b')') {
                    let field = Self::decode_complete_type(buf)?;
                    fields.push(field);
                }
                ensure!(buf.next_u8().is_ok_and(|b| b == b')'));
                Ok(CompleteType::Struct(fields))
            }

            b'{' => {
                let key = Self::decode_complete_type(buf)?;
                let value = Self::decode_complete_type(buf)?;
                ensure!(buf.next_u8().is_ok_and(|b| b == b'}'));
                Ok(CompleteType::DictEntry(Box::new(key), Box::new(value)))
            }

            b'a' => {
                let item = Self::decode_complete_type(buf)?;
                Ok(CompleteType::Array(Box::new(item)))
            }

            b'v' => Ok(CompleteType::Variant),

            other => bail!("unknown signature member: {}", other as char),
        }
    }

    pub(crate) fn decode_signature(buf: &mut DecodingBuffer) -> Result<Signature> {
        let mut sig = Signature { items: vec![] };
        while !buf.is_eof() {
            let complete_type = Self::decode_complete_type(buf)?;
            sig.items.push(complete_type);
        }
        Ok(sig)
    }
}

#[test]
fn test_signature_decode() {
    let mut buf = DecodingBuffer::new(b"(isad(gh))");

    assert_eq!(
        SignatureDecoder::decode_complete_type(&mut buf).unwrap(),
        CompleteType::Struct(vec![
            CompleteType::Int32,
            CompleteType::String,
            CompleteType::Array(Box::new(CompleteType::Double)),
            CompleteType::Struct(vec![CompleteType::Signature, CompleteType::UnixFD])
        ])
    );
}
