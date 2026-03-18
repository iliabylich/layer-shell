use crate::ffi::ShortString;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) struct Service {
    raw_address: ShortString,
    name: ShortString,
}

impl Service {
    pub(crate) fn new(raw_address: ShortString, name: ShortString) -> Self {
        Self { raw_address, name }
    }

    pub(crate) fn raw_address(&self) -> ShortString {
        self.raw_address
    }

    pub(crate) fn name(&self) -> ShortString {
        self.name
    }
}
