use crate::{
    Event,
    actor::{CanStop, TryWantsTrySatisfy},
    event_queue::EventQueue,
    sansio::{Satisfy, UnixSocketReader, Wants},
    user_data::ModuleId,
};
use anyhow::{Context, Result, bail};
use rustix::net::SocketAddrUnix;

pub(crate) enum KbMod {
    Running {
        reader: UnixSocketReader,
        bytes_to_drop: usize,
    },
    Stopped,
}

impl KbMod {
    pub(crate) fn new() -> Self {
        Self::try_new().unwrap_or_else(|err| {
            log::error!("{err:?}");
            Self::stopped()
        })
    }

    fn try_new() -> Result<Self> {
        Ok(Self::Running {
            reader: UnixSocketReader::new(SocketAddrUnix::new("/run/kb-mod-monitor.sock")?),
            bytes_to_drop: 2,
        })
    }

    const fn stopped() -> Self {
        Self::Stopped
    }
}

impl TryWantsTrySatisfy for KbMod {
    const ID: ModuleId = ModuleId::KbMod;
    type Output = ();

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        match self {
            Self::Running { reader, .. } => Ok(reader.wants()),
            Self::Stopped => Ok(None),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<Self::Output> {
        let Self::Running {
            reader,
            bytes_to_drop,
        } = self
        else {
            return Ok(());
        };

        match satisfy {
            Satisfy::Socket(res) => {
                let fd = res?;
                reader.satisfy_socket(fd)?;
                Ok(())
            }

            Satisfy::Connect(res) => {
                res?;
                reader.satisfy_connect()?;
                Ok(())
            }

            Satisfy::Read(res) => {
                let bytes_read = res?;
                let (buf, len) = reader.satisfy_read(bytes_read)?;
                let mut bytes = buf.get(..len).context("buf is too short")?;

                if *bytes_to_drop > 0 && !bytes.is_empty() {
                    let to_drop = std::cmp::min(bytes.len(), *bytes_to_drop);
                    *bytes_to_drop = bytes_to_drop
                        .checked_sub(to_drop)
                        .unwrap_or_else(|| unreachable!());
                    bytes = bytes.get(to_drop..).unwrap_or_else(|| unreachable!());
                }

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

                Ok(())
            }

            _ => bail!("KbMod only accepts Socket, Connect and Read, got: {satisfy:?}"),
        }
    }
}

impl CanStop for KbMod {
    fn stopped(&mut self) -> Self {
        Self::Stopped
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum KbModKind {
    CapsLock,
    NumLock,
}
