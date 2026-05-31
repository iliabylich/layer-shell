use crate::utils::StringRef;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct Service {
    raw_address: StringRef,
    name: StringRef,
}

impl Service {
    pub(crate) const fn new(raw_address: StringRef, name: StringRef) -> Self {
        Self { raw_address, name }
    }

    pub(crate) fn raw_address(&self) -> StringRef {
        self.raw_address.clone()
    }
    pub(crate) fn raw_address_str(&self) -> &str {
        self.raw_address.as_str()
    }

    pub(crate) fn name(&self) -> StringRef {
        self.name.clone()
    }
    pub(crate) fn name_str(&self) -> &str {
        self.name.as_str()
    }
}
