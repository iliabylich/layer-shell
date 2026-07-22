use crate::{
    IoEvent,
    emitter::Emitter,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{FixedSizeBuffer, SockaddrUn, write_in_place},
};
use libc::sockaddr_un;

#[derive(Clone, Copy)]
pub struct PW {
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
        SockaddrUn::from_bytes(path)
    }

    pub(crate) fn new(emitter: Emitter) -> Self {
        log::trace!("Creating PW");

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
        let wants = self.reader.wants(addr, buf.remainder())?;
        log::trace!("{wants:?}");
        Some(wants)
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        buf: &mut FixedSizeBuffer<{ Self::BUFFER_SIZE }>,
    ) -> Result<(), ()> {
        if let Some(written) = self.reader.satisfy(satisfy)?
            && let Some(buf) = buf.written(written)
        {
            let event = PWEvent::deserialize(buf)?;
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
