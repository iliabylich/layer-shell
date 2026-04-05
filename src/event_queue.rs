use crate::Event;
use std::collections::VecDeque;

pub(crate) struct EventQueue;

static mut EVENT_QUEUE: VecDeque<Event> = VecDeque::new();

impl EventQueue {
    pub(crate) fn push_back(event: Event) {
        unsafe { EVENT_QUEUE.push_back(event) }
    }

    pub(crate) fn pop_front() -> Option<Event> {
        unsafe { EVENT_QUEUE.pop_front() }
    }
}
