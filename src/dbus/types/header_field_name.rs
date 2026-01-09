#[repr(u8)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum HeaderFieldName {
    #[default]
    Invalid = 0,
    Path = 1,
    Interface = 2,
    Member = 3,
    ErrorName = 4,
    ReplySerial = 5,
    Destination = 6,
    Sender = 7,
    Signature = 8,
    UnixFds = 9,
}

impl From<u8> for HeaderFieldName {
    fn from(byte: u8) -> Self {
        match byte {
            1 => HeaderFieldName::Path,
            2 => HeaderFieldName::Interface,
            3 => HeaderFieldName::Member,
            4 => HeaderFieldName::ErrorName,
            5 => HeaderFieldName::ReplySerial,
            6 => HeaderFieldName::Destination,
            7 => HeaderFieldName::Sender,
            8 => HeaderFieldName::Signature,
            9 => HeaderFieldName::UnixFds,
            _ => HeaderFieldName::Invalid,
        }
    }
}

impl From<HeaderFieldName> for u8 {
    fn from(header_field: HeaderFieldName) -> Self {
        header_field as u8
    }
}
