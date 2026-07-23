use crate::{
    IoEvent,
    emitter::Emitter,
    module_id::ModuleId,
    modules::Module,
    utils::{FixedSizeBuffer, unix_socket, unix_socket_addr},
};
use rustix::fd::{AsFd, BorrowedFd, OwnedFd};

pub struct KbMod {
    fd: OwnedFd,
    buf: FixedSizeBuffer<1>,
    events_left_to_drop: u8,
}

impl KbMod {
    pub(crate) fn new() -> Option<Self> {
        log::trace!("Creating KbMod");

        let addr = unix_socket_addr!("/run/kb-mod-monitor-systemd.sock").ok()?;
        let fd = unix_socket!().ok()?;

        if let Err(err) = rustix::net::connect(&fd, &addr) {
            log::error!("failed to connect(): {err:?}");
            return None;
        }

        Some(Self {
            fd,
            buf: FixedSizeBuffer::new(),
            events_left_to_drop: 2,
        })
    }
}

impl Module for KbMod {
    fn read(&mut self, emitter: Emitter) -> Result<(), ()> {
        let count = rustix::io::read(&self.fd, self.buf.remainder())
            .map_err(|err| log::error!("failed to read(): {err:?}"))?;
        let Some(buf) = self.buf.written(count) else {
            return Ok(());
        };

        let (kind, enabled) = match buf[0] {
            b'0' => (KbModKind::CapsLock, false),
            b'1' => (KbModKind::CapsLock, true),
            b'2' => (KbModKind::NumLock, false),
            b'3' => (KbModKind::NumLock, true),
            byte => {
                log::error!("Unknown byte: {byte:?}");
                return Err(());
            }
        };

        if self.events_left_to_drop == 0 {
            emitter.emit(&IoEvent::KbModToggled { kind, enabled });
        }
        self.events_left_to_drop = self.events_left_to_drop.saturating_sub(1);

        Ok(())
    }

    fn id(&self) -> ModuleId {
        ModuleId::KbMod
    }

    const MODULE_ID: ModuleId = ModuleId::KbMod;
}

impl AsFd for KbMod {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum KbModKind {
    CapsLock,
    NumLock,
}
