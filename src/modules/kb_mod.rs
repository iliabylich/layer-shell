use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
    sansio::{Satisfy, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
};
use anyhow::{Context, Result, bail};

pub(crate) struct KbMod {
    reader: UnixSocketReader,
    state: State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    WaitingForInitialBytes(usize),
    ReadingUpdates,
    Dummy,
}

impl KbMod {
    pub(crate) fn new() -> Self {
        Self::try_new().unwrap_or_else(|err| {
            log::error!("{err:?}");
            Self::dummy()
        })
    }

    fn try_new() -> Result<Self> {
        let addr = new_unix_socket(b"/run/kb-mod-monitor.sock")?;
        Ok(Self {
            reader: UnixSocketReader::new(addr),
            state: State::WaitingForInitialBytes(2),
        })
    }

    const fn dummy() -> Self {
        Self {
            reader: UnixSocketReader::dummy(),
            state: State::Dummy,
        }
    }
}

impl FallibleModule for KbMod {
    const MODULE_ID: ModuleId = ModuleId::KbMod;
    type Output = ();

    fn wants(&mut self) -> Option<Wants> {
        if self.state == State::Dummy {
            return None;
        }

        Some(self.reader.wants())
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        if self.state == State::Dummy {
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
                let mut bytes = buf.get(..len).context("buf is too short")?;

                if let State::WaitingForInitialBytes(pending) = self.state {
                    if bytes.is_empty() {
                        return Ok(None);
                    }
                    let min = std::cmp::min(pending, bytes.len());
                    // SAFETY: min <= pending
                    let pending = unsafe { pending.unchecked_sub(min) };
                    // SAFETY: min <= bytes.len()
                    bytes = unsafe { bytes.get_unchecked(min..) };

                    if pending == 0 {
                        self.state = State::ReadingUpdates;
                    } else {
                        self.state = State::WaitingForInitialBytes(pending);
                        return Ok(None);
                    }
                }

                if self.state == State::ReadingUpdates {
                    for byte in bytes {
                        let (kind, enabled) = match *byte {
                            b'0' => (KbModKind::CapsLock, false),
                            b'1' => (KbModKind::CapsLock, true),
                            b'2' => (KbModKind::NumLock, false),
                            b'3' => (KbModKind::NumLock, true),
                            _ => return Ok(None),
                        };

                        EventQueue::push_back(Event::KbModToggled { kind, enabled });
                    }
                }

                Ok(None)
            }

            _ => bail!("KbMod only accepts Socket, Connect and Read, got: {satisfy:?}"),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum KbModKind {
    CapsLock,
    NumLock,
}
