use crate::dbus::{Message, MessageEncoder};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

#[derive(Clone, Debug)]
pub(crate) struct DBusQueue {
    serial: Cell<u32>,
    queue: Rc<RefCell<VecDeque<Vec<u8>>>>,
}

impl DBusQueue {
    pub(crate) fn new() -> Self {
        Self {
            serial: Cell::new(1),
            queue: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub(crate) fn encode_in_place(&self, message: &mut Message) -> Vec<u8> {
        let serial = self.serial.get();
        *message.serial_mut() = serial;
        self.serial.set(serial + 1);
        let message = MessageEncoder::encode(&message);
        message
    }

    pub(crate) fn push_back(&self, message: &mut Message) {
        let message = self.encode_in_place(message);

        let mut q = self.queue.borrow_mut();
        q.push_back(message);
    }

    pub(crate) fn pop_front(&self) -> Option<Vec<u8>> {
        let mut q = self.queue.borrow_mut();
        q.pop_front()
    }
}
