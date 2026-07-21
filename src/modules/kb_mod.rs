use crate::{
    IoEvent,
    emitter::Emitter,
    error::IoError,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{FixedSizeBuffer, new_sockaddr_un},
};
use libc::sockaddr_un;

#[derive(Clone, Copy)]
pub(crate) struct KbMod {
    reader: UnixSocketReader,
    events_left_to_drop: u8,
    emitter: Emitter,
}

impl KbMod {
    pub(crate) const BUFFER_SIZE: usize = 1;
    pub(crate) const ADDRESS: sockaddr_un = new_sockaddr_un(b"/run/kb-mod-monitor-systemd.sock");

    pub(crate) const fn new(emitter: Emitter) -> Self {
        Self {
            reader: UnixSocketReader::new(),
            events_left_to_drop: 2,
            emitter,
        }
    }

    pub(crate) fn wants(
        &mut self,
        addr: &sockaddr_un,
        buf: &mut FixedSizeBuffer<{ Self::BUFFER_SIZE }>,
    ) -> Option<Wants> {
        self.reader.wants(addr, buf.remainder())
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        buf: &mut FixedSizeBuffer<{ Self::BUFFER_SIZE }>,
    ) -> Result<(), IoError> {
        if let Some(written) = self.reader.satisfy(satisfy)?
            && let Some(buf) = buf.written(written)
        {
            let (kind, enabled) = match buf[0] {
                b'0' => (KbModKind::CapsLock, false),
                b'1' => (KbModKind::CapsLock, true),
                b'2' => (KbModKind::NumLock, false),
                b'3' => (KbModKind::NumLock, true),
                _ => return Ok(()),
            };

            if self.events_left_to_drop == 0 {
                self.emitter.emit(&IoEvent::KbModToggled { kind, enabled });
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
