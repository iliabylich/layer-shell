use crate::dbus::{MessageEncoder, OutgoingMessage, messages::org_freedesktop_dbus::Hello};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub(crate) struct DBusQueue {
    inner: Rc<RefCell<Inner>>,
}

impl DBusQueue {
    pub(crate) fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner::new())),
        }
    }

    pub(crate) fn push_back(&self, message: impl Into<OutgoingMessage>) -> u32 {
        let mut inner = self.inner.borrow_mut();
        inner.push_back(message)
    }

    pub(crate) fn pop_front(&self) -> Option<Vec<u8>> {
        let mut inner = self.inner.borrow_mut();
        inner.pop_front()
    }

    pub(crate) fn copy(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

struct Inner {
    serial: u32,
    q: VecDeque<Vec<u8>>,
}

impl Inner {
    fn new() -> Self {
        let mut this = Self {
            serial: 1,
            q: VecDeque::new(),
        };
        this.push_back(Hello);
        this
    }

    fn push_back(&mut self, message: impl Into<OutgoingMessage>) -> u32 {
        let mut message: OutgoingMessage = message.into();
        *message.serial_mut() = self.serial;
        self.serial += 1;
        let buf = MessageEncoder::encode(&message);
        self.q.push_back(buf);
        message.serial()
    }

    fn pop_front(&mut self) -> Option<Vec<u8>> {
        self.q.pop_front()
    }
}
