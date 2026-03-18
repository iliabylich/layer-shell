use crate::ffi::ShortString;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct Service {
    raw_address: String,
    name: String,
}

impl Service {
    pub(crate) fn new(raw_address: String, name: String) -> Self {
        Self { raw_address, name }
    }

    pub(crate) fn raw_address(&self) -> ShortString {
        ShortString::from(self.raw_address.as_str())
    }

    pub(crate) fn name(&self) -> ShortString {
        ShortString::from(self.name.as_str())
    }
}
