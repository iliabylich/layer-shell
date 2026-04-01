use crate::{
    dbus::{MessageEncoder, OutgoingMessage, messages::org_freedesktop_dbus::Hello},
    sansio::DBusConnectionKind,
};
use std::collections::VecDeque;

pub(crate) struct SystemDBusQueue;

static mut SYSTEM_DBUS_QUEUE: Queue = Queue::empty();

impl SystemDBusQueue {
    pub(crate) fn init() -> u32 {
        Self::push_back(Hello)
    }

    pub(crate) fn push_back(message: impl Into<OutgoingMessage>) -> u32 {
        #[expect(static_mut_refs)]
        unsafe {
            SYSTEM_DBUS_QUEUE.push_back(message)
        }
    }

    pub(crate) fn pop_front() -> Option<Vec<u8>> {
        #[expect(static_mut_refs)]
        unsafe {
            SYSTEM_DBUS_QUEUE.pop_front()
        }
    }
}

pub(crate) struct SessionDBusQueue;

static mut SESSION_DBUS_QUEUE: Queue = Queue::empty();

impl SessionDBusQueue {
    pub(crate) fn init() -> u32 {
        Self::push_back(Hello)
    }

    pub(crate) fn push_back(message: impl Into<OutgoingMessage>) -> u32 {
        #[expect(static_mut_refs)]
        unsafe {
            SESSION_DBUS_QUEUE.push_back(message)
        }
    }

    pub(crate) fn pop_front() -> Option<Vec<u8>> {
        #[expect(static_mut_refs)]
        unsafe {
            SESSION_DBUS_QUEUE.pop_front()
        }
    }
}

pub(crate) struct DBusQueue;

impl DBusQueue {
    pub(crate) fn push_back(kind: DBusConnectionKind, message: impl Into<OutgoingMessage>) -> u32 {
        match kind {
            DBusConnectionKind::System => SystemDBusQueue::push_back(message),
            DBusConnectionKind::Session => SessionDBusQueue::push_back(message),
        }
    }

    pub(crate) fn pop_front(kind: DBusConnectionKind) -> Option<Vec<u8>> {
        match kind {
            DBusConnectionKind::System => SystemDBusQueue::pop_front(),
            DBusConnectionKind::Session => SessionDBusQueue::pop_front(),
        }
    }
}

struct Queue {
    serial: u32,
    q: VecDeque<Vec<u8>>,
}

impl Queue {
    const fn empty() -> Self {
        Self {
            serial: 1,
            q: VecDeque::new(),
        }
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
