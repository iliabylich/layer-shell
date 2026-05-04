use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
    sansio::{Satisfy, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
};
use anyhow::{Context, Result};

pub(crate) struct CapsLock {
    reader: UnixSocketReader,
    consumed_initial_state: bool,
}

impl CapsLock {
    pub(crate) fn new() -> Result<Self> {
        let addr = new_unix_socket(b"/run/caps-lock-daemon.sock")?;
        Ok(Self {
            reader: UnixSocketReader::new(addr),
            consumed_initial_state: false,
        })
    }
}

impl FallibleModule for CapsLock {
    const MODULE_ID: ModuleId = ModuleId::CapsLock;
    type Output = ();

    fn wants(&mut self) -> Option<Wants> {
        self.reader.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        let Some((buf, len)) = self.reader.try_satisfy(satisfy, res)? else {
            return Ok(None);
        };

        let bytes = buf.get(..len).context("buf is too short")?;

        if bytes.len() == 1 && !self.consumed_initial_state {
            self.consumed_initial_state = true;
            return Ok(None);
        }

        let Some(last) = bytes.last().copied() else {
            return Ok(None);
        };

        let enabled = match last {
            b'0' => false,
            b'1' => true,
            _ => return Ok(None),
        };

        EventQueue::push_back(Event::CapsLockToggled { enabled });
        Ok(None)
    }
}
