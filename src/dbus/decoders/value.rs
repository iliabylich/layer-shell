use crate::dbus::{
    decoders::{DecodingBuffer, SignatureDecoder},
    types::{CompleteType, Signature, Value},
};
use anyhow::{Context, Result};
use std::borrow::Cow;

pub(crate) struct ValueDecoder;

impl ValueDecoder {
    fn decode_u8(buffer: &mut DecodingBuffer) -> Result<u8> {
        buffer.next_u8()
    }

    fn decode_bool(buf: &mut DecodingBuffer) -> Result<bool> {
        Self::decode_u32(buf).map(|v| v != 0)
    }

    fn decode_i16(buf: &mut DecodingBuffer) -> Result<i16> {
        buf.align(2)?;
        buf.next_i16()
    }

    fn decode_u16(buf: &mut DecodingBuffer) -> Result<u16> {
        buf.align(2)?;
        buf.next_u16()
    }

    fn decode_i32(buf: &mut DecodingBuffer) -> Result<i32> {
        buf.align(4)?;
        buf.next_i32()
    }

    fn decode_u32(buf: &mut DecodingBuffer) -> Result<u32> {
        buf.align(4)?;
        buf.next_u32()
    }

    fn decode_i64(buf: &mut DecodingBuffer) -> Result<i64> {
        buf.align(8)?;
        buf.next_i64()
    }

    fn decode_u64(buf: &mut DecodingBuffer) -> Result<u64> {
        buf.align(8)?;
        buf.next_u64()
    }

    fn decode_f64(buf: &mut DecodingBuffer) -> Result<f64> {
        buf.align(8)?;
        buf.next_f64()
    }

    fn decode_string(buf: &mut DecodingBuffer) -> Result<String> {
        let len = Self::decode_u32(buf)? as usize;
        let s = String::from_utf8_lossy(buf.next_n(len)?).into_owned();
        buf.skip();
        Ok(s)
    }

    fn decode_object_path(buf: &mut DecodingBuffer) -> Result<Cow<'static, str>> {
        let len = Self::decode_u32(buf)? as usize;
        let path = buf.next_n(len)?.to_vec();
        buf.skip();
        Ok(String::from_utf8(path).context("non-utf8 path")?.into())
    }

    fn decode_complete_type(buf: &mut DecodingBuffer) -> Result<CompleteType> {
        let len = Self::decode_u8(buf)? as usize;
        let bytes = buf.next_n(len)?.to_vec();
        buf.skip();
        let mut buf = DecodingBuffer::new(&bytes);
        SignatureDecoder::decode_complete_type(&mut buf)
    }

    fn decode_signature(buf: &mut DecodingBuffer) -> Result<Vec<u8>> {
        let len = Self::decode_u8(buf)? as usize;
        let s = buf.next_n(len)?.to_vec();
        buf.skip();
        Ok(s)
    }

    fn decode_array(
        buf: &mut DecodingBuffer,
        item_type: &CompleteType,
    ) -> Result<Vec<Value<'static>>> {
        let byte_len = Self::decode_u32(buf)? as usize;

        buf.align(item_type.alignment())?;

        let start_pos = buf.pos();
        let end_pos = start_pos + byte_len;

        let mut items = vec![];
        while buf.pos() < end_pos {
            let item = Self::decode_value_by_complete_type(buf, item_type)?;
            items.push(item);
        }

        Ok(items)
    }

    fn decode_struct(
        buf: &mut DecodingBuffer,
        field_types: &[CompleteType],
    ) -> Result<Vec<Value<'static>>> {
        buf.align(8)?;
        let mut fields = vec![];
        for field_type in field_types {
            let value = Self::decode_value_by_complete_type(buf, field_type)?;
            fields.push(value);
        }
        Ok(fields)
    }

    fn decode_dict_entry(
        buf: &mut DecodingBuffer,
        key_type: &CompleteType,
        value_type: &CompleteType,
    ) -> Result<(Value<'static>, Value<'static>)> {
        buf.align(8)?;
        let key = Self::decode_value_by_complete_type(buf, key_type)?;
        let value = Self::decode_value_by_complete_type(buf, value_type)?;
        Ok((key, value))
    }

    pub(crate) fn decode_value_by_complete_type(
        buf: &mut DecodingBuffer,
        complete_type: &CompleteType,
    ) -> Result<Value<'static>> {
        match complete_type {
            CompleteType::Byte => {
                let value = Self::decode_u8(buf)?;
                Ok(Value::Byte(value))
            }
            CompleteType::Bool => {
                let value = Self::decode_bool(buf)?;
                Ok(Value::Bool(value))
            }
            CompleteType::Int16 => {
                let value = Self::decode_i16(buf)?;
                Ok(Value::Int16(value))
            }
            CompleteType::UInt16 => {
                let value = Self::decode_u16(buf)?;
                Ok(Value::UInt16(value))
            }
            CompleteType::Int32 => {
                let value = Self::decode_i32(buf)?;
                Ok(Value::Int32(value))
            }
            CompleteType::UInt32 => {
                let value = Self::decode_u32(buf)?;
                Ok(Value::UInt32(value))
            }
            CompleteType::Int64 => {
                let value = Self::decode_i64(buf)?;
                Ok(Value::Int64(value))
            }
            CompleteType::UInt64 => {
                let value = Self::decode_u64(buf)?;
                Ok(Value::UInt64(value))
            }
            CompleteType::Double => {
                let value = Self::decode_f64(buf)?;
                Ok(Value::Double(value))
            }
            CompleteType::UnixFD => {
                let value = Self::decode_u32(buf)?;
                Ok(Value::UnixFD(value))
            }
            CompleteType::String => {
                let value = Self::decode_string(buf)?;
                Ok(Value::String(value))
            }
            CompleteType::ObjectPath => {
                let value = Self::decode_object_path(buf)?;
                Ok(Value::ObjectPath(value))
            }
            CompleteType::Signature => {
                let value = Self::decode_signature(buf)?;
                Ok(Value::Signature(value))
            }
            CompleteType::Struct(signatures) => {
                let fields = Self::decode_struct(buf, signatures)?;
                Ok(Value::Struct(fields))
            }
            CompleteType::Array(item_signature) => {
                let items = Self::decode_array(buf, item_signature)?;
                Ok(Value::Array(*item_signature.clone(), items))
            }
            CompleteType::DictEntry(key_type, value_type) => {
                let (key, value) = Self::decode_dict_entry(buf, key_type, value_type)?;
                Ok(Value::DictEntry(Box::new(key), Box::new(value)))
            }
            CompleteType::Variant => {
                let complete_type = Self::decode_complete_type(buf)?;
                let inner = Self::decode_value_by_complete_type(buf, &complete_type)?;
                Ok(Value::Variant(Box::new(inner)))
            }
        }
    }

    pub(crate) fn decode_values_by_signature(
        buf: &mut DecodingBuffer,
        signature: &Signature,
    ) -> Result<Vec<Value<'static>>> {
        let mut out = vec![];
        for complete_type in &signature.items {
            let value = Self::decode_value_by_complete_type(buf, complete_type)?;
            out.push(value);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::{DecodingBuffer, ValueDecoder};

    #[test]
    fn test_read_byte() {
        let mut buf = DecodingBuffer::new(b"\xFF");
        buf.set_pos(0);
        assert_eq!(ValueDecoder::decode_u8(&mut buf).unwrap(), 255);
        assert!(buf.is_eof());
    }

    #[test]
    fn test_read_bool() {
        let mut buf = DecodingBuffer::new(b"\0\0\0\0\x01\x00\x00\x00");
        buf.set_pos(1);
        assert_eq!(ValueDecoder::decode_bool(&mut buf).unwrap(), true);
        assert!(buf.is_eof());

        let mut buf = DecodingBuffer::new(b"\0\0\0\0\x00\x00\x00\x00");
        buf.set_pos(1);
        assert_eq!(ValueDecoder::decode_bool(&mut buf).unwrap(), false);
        assert!(buf.is_eof());
    }

    #[test]
    fn test_read_int16() {
        let mut buf = DecodingBuffer::new(b"\0\0\xAA\xBB");
        buf.set_pos(1);
        assert_eq!(
            ValueDecoder::decode_i16(&mut buf).unwrap(),
            0xBB << 8 | 0xAA
        );
        assert!(buf.is_eof());
    }

    #[test]
    fn test_read_uint16() {
        let mut buf = DecodingBuffer::new(b"\0\0\xAA\xBB");
        buf.set_pos(1);
        assert_eq!(
            ValueDecoder::decode_u16(&mut buf).unwrap(),
            0xBB << 8 | 0xAA
        );
        assert!(buf.is_eof());
    }

    #[test]
    fn test_read_int32() {
        let mut buf = DecodingBuffer::new(b"\0\0\0\0\xAA\xBB\xCC\xDD");
        buf.set_pos(1);
        assert_eq!(
            ValueDecoder::decode_i32(&mut buf).unwrap(),
            0xDD << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA
        );
        assert!(buf.is_eof());
    }

    #[test]
    fn test_read_uint32() {
        let mut buf = DecodingBuffer::new(b"\0\0\0\0\xAA\xBB\xCC\xDD");
        buf.set_pos(1);
        assert_eq!(
            ValueDecoder::decode_u32(&mut buf).unwrap(),
            0xDD << 24 | 0xCC << 16 | 0xBB << 8 | 0xAA
        );
        assert!(buf.is_eof());
    }

    #[test]
    fn test_read_int64() {
        let mut buf = DecodingBuffer::new(b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08");
        buf.set_pos(1);
        assert_eq!(
            ValueDecoder::decode_i64(&mut buf).unwrap(),
            0x08 << 56
                | 0x07 << 48
                | 0x06 << 40
                | 0x05 << 32
                | 0x04 << 24
                | 0x03 << 16
                | 0x02 << 8
                | 0x01,
        );
        assert!(buf.is_eof());
    }

    #[test]
    fn test_read_uint64() {
        let mut buf = DecodingBuffer::new(b"\0\0\0\0\0\0\0\0\x01\x02\x03\x04\x05\x06\x07\x08");
        buf.set_pos(1);
        assert_eq!(
            ValueDecoder::decode_u64(&mut buf).unwrap(),
            0x08 << 56
                | 0x07 << 48
                | 0x06 << 40
                | 0x05 << 32
                | 0x04 << 24
                | 0x03 << 16
                | 0x02 << 8
                | 0x01,
        );
        assert!(buf.is_eof());
    }

    #[test]
    fn test_read_f64() {
        let mut buf = DecodingBuffer::new(b"\0\0\0\0\0\0\0\0\xB0\x72\x68\x91\xED\x7C\xBF\x3F");
        buf.set_pos(1);
        assert_eq!(ValueDecoder::decode_f64(&mut buf).unwrap(), 0.123);
        assert!(buf.is_eof())
    }

    #[test]
    fn test_read_object_path() {
        let mut buf = DecodingBuffer::new(b"\0\0\0\0\x04\0\0\0abcd\0");
        buf.set_pos(1);
        assert_eq!(ValueDecoder::decode_object_path(&mut buf).unwrap(), "abcd");
        assert!(buf.is_eof())
    }

    #[test]
    fn test_read_string() {
        let mut buf = DecodingBuffer::new(b"\0\0\0\0\x04\0\0\0abcd\0");
        buf.set_pos(1);
        assert_eq!(ValueDecoder::decode_string(&mut buf).unwrap(), "abcd");
        assert!(buf.is_eof())
    }

    #[test]
    fn test_read_signature() {
        let mut buf = DecodingBuffer::new(b"\0\x04abcd\0");
        buf.set_pos(1);
        assert_eq!(ValueDecoder::decode_signature(&mut buf).unwrap(), b"abcd");
        assert!(buf.is_eof())
    }
}
