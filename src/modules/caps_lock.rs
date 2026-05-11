use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
    sansio::{Satisfy, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
};
use anyhow::{Context, Result, bail};

pub(crate) struct CapsLock {
    reader: UnixSocketReader,
    consumed_initial_state: bool,
    dummy: bool,
}

impl CapsLock {
    pub(crate) fn new() -> Self {
        Self::try_new().unwrap_or_else(|err| {
            log::error!("{err:?}");
            Self::dummy()
        })
    }

    fn try_new() -> Result<Self> {
        let addr = new_unix_socket(b"/run/caps-lock-daemon.sock")?;
        Ok(Self {
            reader: UnixSocketReader::new(addr),
            consumed_initial_state: false,
            dummy: false,
        })
    }

    const fn dummy() -> Self {
        Self {
            reader: UnixSocketReader::dummy(),
            consumed_initial_state: false,
            dummy: true,
        }
    }
}

impl FallibleModule for CapsLock {
    const MODULE_ID: ModuleId = ModuleId::CapsLock;
    type Output = ();

    fn wants(&mut self) -> Option<Wants> {
        if self.dummy {
            return None;
        }

        Some(self.reader.wants())
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        if self.dummy {
            return Ok(None);
        }

        match satisfy {
            Satisfy::Socket => {
                self.reader.satisfy_socket(res)?;
                Ok(None)
            }

            Satisfy::Connect => {
                self.reader.satisfy_connect(res)?;
                Ok(None)
            }

            Satisfy::Read => {
                let (buf, len) = self.reader.satisfy_read(res)?;

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

            _ => bail!("CapsLock only accepts Socket, Connect and Read, got: {satisfy:?}"),
        }
    }
}
