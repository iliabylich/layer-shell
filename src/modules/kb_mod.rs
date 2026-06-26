use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{Satisfy, UnixSocketReader, Wants},
};
use anyhow::{Context, Result, bail};
use rustix::net::SocketAddrUnix;

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
        Ok(Self {
            reader: UnixSocketReader::new(SocketAddrUnix::new("/run/kb-mod-monitor.sock")?),
            state: State::WaitingForInitialBytes(2),
        })
    }

    const fn dummy() -> Self {
        Self {
            reader: UnixSocketReader::dummy(),
            state: State::Dummy,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        if self.state == State::Dummy {
            return None;
        }

        self.reader.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<()> {
        if self.state == State::Dummy {
            return Ok(());
        }

        match satisfy {
            Satisfy::Socket(res) => {
                let fd = res?;
                self.reader.satisfy_socket(fd)?;
                Ok(())
            }

            Satisfy::Connect(res) => {
                res?;
                self.reader.satisfy_connect()?;
                Ok(())
            }

            Satisfy::Read(res) => {
                let bytes_read = res?;
                let (buf, len) = self.reader.satisfy_read(bytes_read)?;
                let mut bytes = buf.get(..len).context("buf is too short")?;

                if let State::WaitingForInitialBytes(pending) = self.state {
                    if bytes.is_empty() {
                        return Ok(());
                    }
                    let min = std::cmp::min(pending, bytes.len());
                    let pending = pending.checked_sub(min).context("malformed state")?;
                    bytes = bytes.get(min..).context("malformed state")?;

                    if pending == 0 {
                        self.state = State::ReadingUpdates;
                    } else {
                        self.state = State::WaitingForInitialBytes(pending);
                        return Ok(());
                    }
                }

                if self.state == State::ReadingUpdates {
                    for byte in bytes {
                        let (kind, enabled) = match *byte {
                            b'0' => (KbModKind::CapsLock, false),
                            b'1' => (KbModKind::CapsLock, true),
                            b'2' => (KbModKind::NumLock, false),
                            b'3' => (KbModKind::NumLock, true),
                            _ => return Ok(()),
                        };

                        events.push_back(Event::KbModToggled { kind, enabled });
                    }
                }

                Ok(())
            }

            _ => bail!("KbMod only accepts Socket, Connect and Read, got: {satisfy:?}"),
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) {
        if let Err(err) = self.try_satisfy(satisfy, events) {
            log::error!("{err:?}");
            self.reader.stop();
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum KbModKind {
    CapsLock,
    NumLock,
}
