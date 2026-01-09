#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompleteType {
    Byte,
    Bool,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Double,
    UnixFD,

    String,
    ObjectPath,
    Signature,
    Struct(Vec<CompleteType>),
    Array(Box<CompleteType>),
    DictEntry(Box<CompleteType>, Box<CompleteType>),
    Variant,
}

impl CompleteType {
    pub(crate) fn alignment(&self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Bool => 4,
            Self::Int16 => 2,
            Self::UInt16 => 2,
            Self::Int32 => 4,
            Self::UInt32 => 4,
            Self::Int64 => 8,
            Self::UInt64 => 8,
            Self::Double => 8,
            Self::UnixFD => 4,
            Self::String => 4,
            Self::ObjectPath => 4,
            Self::Signature => 1,
            Self::Struct(_) => 8,
            Self::Array(_) => 4,
            Self::DictEntry(_, _) => 8,
            Self::Variant => 1,
        }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) struct Signature {
    pub(crate) items: Vec<CompleteType>,
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signature(")?;
        let mut started = false;
        for item in &self.items {
            write!(f, "{}{:?}", if started { " -> " } else { "" }, item)?;
            started = true;
        }
        write!(f, ")")
    }
}
