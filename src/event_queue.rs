use crate::Event;
use alloc::collections::VecDeque;

pub(crate) struct EventQueue(VecDeque<Event>);

impl EventQueue {
    pub(crate) const fn new() -> Self {
        Self(VecDeque::new())
    }

    pub(crate) fn push_back(&mut self, event: Event) {
        self.0.push_back(event);
    }

    pub(crate) fn pop_front(&mut self) -> Option<Event> {
        self.0.pop_front()
    }
}
