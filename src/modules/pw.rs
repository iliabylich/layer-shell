use crate::{
    IoEvent,
    emitter::Emitter,
    module_id::ModuleId,
    modules::Module,
    utils::{FixedSizeBuffer, unix_socket, unix_socket_addr},
};
use rustix::fd::{AsFd, BorrowedFd, OwnedFd};

pub struct PW {
    fd: OwnedFd,
    buf: FixedSizeBuffer<{ PWEvent::SERIALIZED_LENGTH }>,

    volume: Option<u8>,
    muted: Option<bool>,
    events_left_to_drop: u8,
}

impl PW {
    pub(crate) fn new(xdg_runtime_dir: &str) -> Option<Self> {
        log::trace!("Creating PW");

        let addr = unix_socket_addr!("{xdg_runtime_dir}/pipewire-mon.sock").ok()?;
        let fd = unix_socket!().ok()?;

        if let Err(err) = rustix::net::connect(&fd, &addr) {
            log::error!("failed to connect(): {err:?}");
            return None;
        }

        Some(Self {
            fd,
            buf: FixedSizeBuffer::new(),

            volume: None,
            muted: None,
            events_left_to_drop: 1,
        })
    }
}

impl Module for PW {
    fn read(&mut self, emitter: Emitter) -> Result<(), ()> {
        let count = rustix::io::read(&self.fd, self.buf.remainder())
            .map_err(|err| log::error!("failed to read(): {err:?}"))?;
        let Some(buf) = self.buf.written(count) else {
            return Ok(());
        };

        let event = PWEvent::deserialize(buf)?;
        match event {
            PWEvent::Volume(volume) => self.volume = Some(volume),
            PWEvent::Mute(muted) => self.muted = Some(muted),
        }
        if let Some(volume) = self.volume
            && let Some(muted) = self.muted
        {
            if self.events_left_to_drop == 0 {
                emitter.emit(&IoEvent::Sound { volume, muted });
            }
            self.events_left_to_drop = self.events_left_to_drop.saturating_sub(1);
        }

        Ok(())
    }

    fn id(&self) -> ModuleId {
        ModuleId::PW
    }

    const MODULE_ID: ModuleId = ModuleId::PW;
}

impl AsFd for PW {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PWEvent {
    Volume(u8),
    Mute(bool),
}

impl PWEvent {
    const SERIALIZED_LENGTH: usize = 2;

    fn deserialize(buf: [u8; Self::SERIALIZED_LENGTH]) -> Result<Self, ()> {
        match buf[0] {
            1 => {
                let volume = buf[1];
                Ok(Self::Volume(volume))
            }
            2 => {
                let muted = bool::try_from(buf[1]).map_err(|err| {
                    log::error!("can't convert {} into bool: {err:?}", buf[1]);
                })?;
                Ok(Self::Mute(muted))
            }

            kind => {
                log::error!("malformed byte sequence, starts with {kind}");
                Err(())
            }
        }
    }
}
