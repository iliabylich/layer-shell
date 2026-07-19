use crate::{
    Event,
    emitter::Emitter,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::new_sockaddr_un,
};
use alloc::boxed::Box;
use anyhow::{Context, Result};
use libc::sockaddr_un;

pub(crate) struct KbMod {
    reader: Box<UnixSocketReader>,
    events_left_to_drop: u8,
    emitter: Emitter,
}

impl KbMod {
    pub(crate) fn address() -> Result<sockaddr_un> {
        let addr = new_sockaddr_un(b"/run/kb-mod-monitor-systemd.sock")?;
        Ok(addr)
    }

    pub(crate) fn new(emitter: Emitter) -> Self {
        Self {
            reader: Box::new(UnixSocketReader::new()),
            events_left_to_drop: 2,
            emitter,
        }
    }

    pub(crate) fn wants(&mut self, addr: &sockaddr_un) -> Option<Wants> {
        self.reader.wants(addr)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) -> Result<()> {
        let Some((buf, len)) = self.reader.satisfy(satisfy)? else {
            return Ok(());
        };
        let bytes = buf.get(..len).context("buf is too short")?;

        for byte in bytes {
            let (kind, enabled) = match *byte {
                b'0' => (KbModKind::CapsLock, false),
                b'1' => (KbModKind::CapsLock, true),
                b'2' => (KbModKind::NumLock, false),
                b'3' => (KbModKind::NumLock, true),
                _ => return Ok(()),
            };

            if self.events_left_to_drop == 0 {
                self.emitter.emit(&Event::KbModToggled { kind, enabled });
            }
            self.events_left_to_drop = self.events_left_to_drop.saturating_sub(1);
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum KbModKind {
    CapsLock,
    NumLock,
}
