use crate::{
    Event,
    event_queue::EventQueue,
    modules::Module,
    sansio::{Satisfy, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
};
use anyhow::{Context, Result};

pub(crate) struct CapsLock {
    reader: UnixSocketReader,
    consumed_initial_state: bool,
    dead: bool,
}

impl CapsLock {
    pub(crate) fn new() -> Result<Self> {
        let addr = new_unix_socket(b"/run/caps-lock-daemon.sock")?;
        Ok(Self {
            reader: UnixSocketReader::new(addr),
            consumed_initial_state: false,
            dead: false,
        })
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        if self.dead {
            return Ok(());
        }

        let Some((buf, len)) = self.reader.satisfy(satisfy, res) else {
            return Ok(());
        };

        let bytes = buf.get(..len).context("buf is too short")?;

        if bytes.len() == 1 && !self.consumed_initial_state {
            self.consumed_initial_state = true;
            return Ok(());
        }

        let Some(last) = bytes.last().copied() else {
            return Ok(());
        };

        let enabled = match last {
            b'0' => false,
            b'1' => true,
            _ => return Ok(()),
        };

        EventQueue::push_back(Event::CapsLockToggled { enabled });
        Ok(())
    }
}

impl Module for CapsLock {
    type Output = ();

    fn wants(&mut self) -> Result<Option<Wants>> {
        if self.dead {
            return Ok(None);
        }

        Ok(self.reader.wants())
    }

    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Self::Output {
        if let Err(err) = self.try_satisfy(satisfy, res) {
            log::error!(target: "CapsLock", "{err:?}");
            self.dead = true;
        }
    }
}
