use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{Satisfy, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
};
use anyhow::Result;

pub(crate) struct CapsLock {
    reader: UnixSocketReader,
    consumed_initial_state: bool,
}

impl CapsLock {
    pub(crate) fn new() -> Self {
        let addr = new_unix_socket(b"/run/caps-lock-daemon.sock");
        Self {
            reader: UnixSocketReader::new(addr),
            consumed_initial_state: false,
        }
    }

    pub(crate) fn module_id(&self) -> ModuleId {
        ModuleId::CapsLock
    }

    pub(crate) fn wants(&mut self) -> Result<Option<Wants>> {
        Ok(self.reader.wants())
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) {
        let Some((buf, len)) = self.reader.satisfy(satisfy, res) else {
            return;
        };

        let bytes = &buf[..len];

        if bytes.len() == 1 && !self.consumed_initial_state {
            self.consumed_initial_state = true;
            return;
        }

        let Some(last) = bytes.last().copied() else {
            return;
        };

        let enabled = match last {
            b'0' => false,
            b'1' => true,
            _ => return,
        };

        EventQueue::push_back(Event::CapsLockToggled { enabled });
    }
}
