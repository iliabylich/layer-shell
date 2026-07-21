use crate::{
    IoEvent,
    emitter::Emitter,
    error::IoError,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{FixedSizeBuffer, new_sockaddr_un, write_in_place},
};
use libc::sockaddr_un;
use thiserror::Error;

#[derive(Clone, Copy)]
pub(crate) struct PW {
    reader: UnixSocketReader,
    volume: Option<u8>,
    muted: Option<bool>,
    events_left_to_drop: u8,
    emitter: Emitter,
}

impl PW {
    pub(crate) const BUFFER_SIZE: usize = PWEvent::SERIALIZED_LENGTH;

    pub(crate) fn address(xdg_runtime_dir: &str) -> sockaddr_un {
        let mut buf = [0; 200];
        let path = write_in_place!(&mut buf, "{xdg_runtime_dir}/pipewire-mon.sock");
        new_sockaddr_un(path)
    }

    pub(crate) const fn new(emitter: Emitter) -> Self {
        Self {
            reader: UnixSocketReader::new(),
            volume: None,
            muted: None,
            events_left_to_drop: 1,
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
            let event = PWEvent::deserialize(buf).ok_or(PWError::FailedToDeserialize)?;
            match event {
                PWEvent::Volume(volume) => self.volume = Some(volume),
                PWEvent::Mute(muted) => self.muted = Some(muted),
            }
            if let Some(volume) = self.volume
                && let Some(muted) = self.muted
            {
                if self.events_left_to_drop == 0 {
                    self.emitter.emit(&IoEvent::Sound { volume, muted });
                }
                self.events_left_to_drop = self.events_left_to_drop.saturating_sub(1);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PWEvent {
    Volume(u8),
    Mute(bool),
}

impl PWEvent {
    const SERIALIZED_LENGTH: usize = 2;

    fn deserialize(buf: [u8; Self::SERIALIZED_LENGTH]) -> Option<Self> {
        match buf[0] {
            1 => {
                let volume = buf[1];
                Some(Self::Volume(volume))
            }
            2 => {
                let muted = bool::try_from(buf[1]).ok()?;
                Some(Self::Mute(muted))
            }

            _kind => None,
        }
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub(crate) enum PWError {
    #[error("failed to deserialize PW event")]
    FailedToDeserialize,
}
