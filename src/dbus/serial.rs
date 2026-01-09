use std::cell::Cell;

#[derive(Debug, Default)]
pub(crate) struct Serial {
    counter: Cell<u32>,
}

impl Serial {
    pub(crate) fn zero() -> Self {
        Self {
            counter: Cell::new(0),
        }
    }

    pub(crate) fn increment(&self) {
        self.counter.update(|v| v + 1);
    }

    pub(crate) fn get(&self) -> u32 {
        self.counter.get()
    }

    pub(crate) fn increment_and_get(&self) -> u32 {
        self.increment();
        self.get()
    }
}
