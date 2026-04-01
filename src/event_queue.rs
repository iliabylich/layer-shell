use crate::Event;
use core::cell::RefCell;
use std::{collections::VecDeque, rc::Rc};

pub(crate) struct EventQueue {
    queue: Rc<RefCell<VecDeque<Event>>>,
}

impl EventQueue {
    pub(crate) fn new() -> Self {
        Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub(crate) fn push_back(&self, event: Event) {
        let mut q = self.queue.borrow_mut();
        q.push_back(event);
    }

    pub(crate) fn pop_front(&self) -> Option<Event> {
        let mut q = self.queue.borrow_mut();
        q.pop_front()
    }

    pub(crate) fn copy(&self) -> Self {
        Self {
            queue: Rc::clone(&self.queue),
        }
    }
}
