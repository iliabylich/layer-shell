#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct Service {
    raw_address: String,
    name: String,
}

impl Service {
    pub(crate) fn new(raw_address: String, name: String) -> Self {
        Self { raw_address, name }
    }

    pub(crate) fn raw_address(&self) -> &str {
        &self.raw_address
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}
