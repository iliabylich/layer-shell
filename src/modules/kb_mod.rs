use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{Satisfy, UnixSocketReader, Wants},
};
use alloc::boxed::Box;
use anyhow::{Context, Result, bail};
use rustix::net::{SocketAddrAny, SocketAddrUnix};

pub(crate) struct KbMod {
    reader: Box<UnixSocketReader>,
}

impl KbMod {
    pub(crate) fn address() -> Result<SocketAddrAny> {
        let addr = SocketAddrUnix::new("/run/kb-mod-monitor-systemd.sock")
            .map_err(|errno| anyhow::anyhow!(errno))?;
        Ok(addr.into())
    }

    pub(crate) fn new() -> Self {
        Self {
            reader: Box::new(UnixSocketReader::new()),
        }
    }

    pub(crate) fn wants(&mut self, addr: &SocketAddrAny) -> Option<Wants> {
        self.reader.wants(addr)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<()> {
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
                let bytes = buf.get(..len).context("buf is too short")?;

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

#[repr(C)]
#[derive(Debug)]
pub enum KbModKind {
    CapsLock,
    NumLock,
}
