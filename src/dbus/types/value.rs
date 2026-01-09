use crate::dbus::types::signature::CompleteType;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Value {
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

    String(String),
    ObjectPath(Cow<'static, str>),
    Signature(Vec<u8>),
    Struct(Vec<Value>),
    Array(CompleteType, Vec<Value>),
    DictEntry(Box<Value>, Box<Value>),
    Variant(Box<Value>),
}

impl Value {
    #[allow(dead_code)]
    pub(crate) fn new_non_empty_auto_array(items: Vec<Value>) -> Self {
        let Some(first_item) = items.first() else {
            panic!("an array must be non-empty");
        };
        let item_type = first_item.complete_type();
        Self::Array(item_type, items)
    }

    pub(crate) fn complete_type(&self) -> CompleteType {
        match self {
            Self::Byte(_) => CompleteType::Byte,
            Self::Bool(_) => CompleteType::Bool,
            Self::Int16(_) => CompleteType::Int16,
            Self::UInt16(_) => CompleteType::UInt16,
            Self::Int32(_) => CompleteType::Int32,
            Self::UInt32(_) => CompleteType::UInt32,
            Self::Int64(_) => CompleteType::Int64,
            Self::UInt64(_) => CompleteType::UInt64,
            Self::Double(_) => CompleteType::Double,
            Self::UnixFD(_) => CompleteType::UnixFD,
            Self::String(_) => CompleteType::String,
            Self::ObjectPath(_) => CompleteType::ObjectPath,
            Self::Signature(_) => CompleteType::Signature,
            Self::Struct(values) => {
                let mut types = vec![];
                for value in values {
                    types.push(value.complete_type());
                }
                CompleteType::Struct(types)
            }
            Self::Array(item_type, items) => {
                for item in items {
                    if item.complete_type() != *item_type {
                        panic!("heterogenous array")
                    }
                }
                CompleteType::Array(Box::new(item_type.clone()))
            }
            Self::DictEntry(key, value) => CompleteType::DictEntry(
                Box::new(key.complete_type()),
                Box::new(value.complete_type()),
            ),
            Self::Variant(_value) => CompleteType::Variant,
        }
    }
}
