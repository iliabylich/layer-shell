use std::collections::HashSet;

pub(crate) struct Pending {
    inner: HashSet<u64>,
}

impl Pending {
    pub(crate) fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    pub(crate) fn is(&mut self, op: u64) -> bool {
        self.inner.contains(&op)
    }

    pub(crate) fn set(&mut self, op: u64) {
        self.inner.insert(op);
    }

    pub(crate) fn unset(&mut self, op: u64) {
        self.inner.remove(&op);
    }
}
