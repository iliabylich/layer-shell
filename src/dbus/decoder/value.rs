use crate::dbus::decoder::{
    ArrayValue, CompleteType, Cursor, DictEntryValue, StructValue, VariantValue,
};
use anyhow::{Result, ensure};

#[derive(Debug)]
pub(crate) enum Value<'a> {
    Byte(u8),
    Bool(bool),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Double(f64),
    UnixFD(u32),
    String(&'a str),
    ObjectPath(&'a str),
    Signature(&'a str),
    Struct(StructValue<'a>),
    Array(ArrayValue<'a>),
    DictEntry(DictEntryValue<'a>),
    Variant(VariantValue<'a>),
}
impl<'a> Value<'a> {
    pub(crate) fn cut(cur: &mut Cursor<'a>, type_: CompleteType<'a>) -> Result<Self> {
        cur.align(type_.alignment())?;
        let mut cur = {
            let start = cur.offset();
            let size = type_.bytesize(*cur)?;
            let buf = cur.take(size)?;
            Cursor::new(buf, start)
        };
        macro_rules! cut_primitive {
            ($f:ident, $ctor:ident) => {{
                let v = cur.$f()?;
                ensure!(cur.is_empty(), "expected no leftover");
                Self::$ctor(v)
            }};
        }

        let value = match type_ {
            CompleteType::Byte => cut_primitive!(cut_u8, Byte),
            CompleteType::Bool => cut_primitive!(cut_bool, Bool),
            CompleteType::Int16 => cut_primitive!(cut_i16, Int16),
            CompleteType::UInt16 => cut_primitive!(cut_u16, UInt16),
            CompleteType::Int32 => cut_primitive!(cut_i32, Int32),
            CompleteType::UInt32 => cut_primitive!(cut_u32, UInt32),
            CompleteType::Int64 => cut_primitive!(cut_i64, Int64),
            CompleteType::UInt64 => cut_primitive!(cut_u64, UInt64),
            CompleteType::Double => cut_primitive!(cut_f64, Double),
            CompleteType::UnixFD => cut_primitive!(cut_u32, UnixFD),
            CompleteType::String => cut_primitive!(cut_string, String),
            CompleteType::ObjectPath => cut_primitive!(cut_string, ObjectPath),
            CompleteType::Signature => cut_primitive!(cut_signature, Signature),
            CompleteType::Struct { .. } => Self::Struct(StructValue::new(type_, cur)),
            CompleteType::Array {
                item: item_type, ..
            } => Self::Array(ArrayValue::cut(&mut cur, item_type)?),
            CompleteType::DictEntry {
                key: key_type,
                value: value_type,
                ..
            } => Self::DictEntry(DictEntryValue::cut(&mut cur, key_type, value_type)?),
            CompleteType::Variant => Self::Variant(VariantValue::new(cur)),
        };

        Ok(value)
    }

    #[allow(dead_code)]
    pub(crate) fn log(&self, indent: usize) -> Result<()> {
        let offset = " ".repeat(indent);

        match self {
            Self::Byte(n) => eprintln!("{offset}u8: {n}"),
            Self::Bool(bool) => eprintln!("{offset}bool: {bool}"),
            Self::Int16(n) => eprintln!("{offset}i16: {n}"),
            Self::UInt16(n) => eprintln!("{offset}u16: {n}"),
            Self::Int32(n) => eprintln!("{offset}i32: {n}"),
            Self::UInt32(n) => eprintln!("{offset}u32: {n}"),
            Self::Int64(n) => eprintln!("{offset}i64: {n}"),
            Self::UInt64(n) => eprintln!("{offset}u64: {n}"),
            Self::Double(n) => eprintln!("{offset}double: {n}"),
            Self::UnixFD(n) => eprintln!("{offset}unixfd: {n}"),
            Self::String(s) => eprintln!("{offset}string: {s:?}"),
            Self::ObjectPath(path) => eprintln!("{offset}path: {path:?}"),
            Self::Signature(signature) => eprintln!("{offset}signature: {signature:?}"),
            Self::Struct(struct_) => {
                let mut iter = struct_.iter()?;
                eprintln!("{offset}struct:");
                while let Some(item) = iter.try_next()? {
                    item.log(indent + 4)?;
                }
            }
            Self::Array(array) => {
                let mut iter = array.iter();
                eprintln!("{offset}array:");
                while let Some(item) = iter.try_next()? {
                    item.log(indent + 4)?;
                }
            }
            Self::DictEntry(pair) => {
                eprintln!("{offset}dict:");
                let (key, value) = pair.key_value()?;
                eprintln!("{offset}    key:");
                key.log(indent + 8)?;
                eprintln!("{offset}    value:");
                value.log(indent + 8)?;
            }
            Self::Variant(variant) => {
                eprintln!("{offset}variant:");
                let value = variant.materialize()?;
                value.log(indent + 4)?;
            }
        }

        Ok(())
    }
}
