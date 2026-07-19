use crate::{
    Event,
    emitter::Emitter,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{ArrayWriter, FixedSizeBuffer, getenv, new_sockaddr_un},
};
use anyhow::{Context, Result, bail};
use core::fmt::Write;
use libc::sockaddr_un;

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

    pub(crate) fn address() -> Result<sockaddr_un> {
        let xdg_runtime_dir =
            core::str::from_utf8(getenv(c"XDG_RUNTIME_DIR").context("no $XDG_RUNTIME_DIR")?)?;
        let mut buf = [0; 200];
        let mut writer = ArrayWriter::new(&mut buf);
        write!(&mut writer, "{xdg_runtime_dir}/pipewire-mon.sock")?;
        let path = writer.as_bytes()?;
        let addr = new_sockaddr_un(path)?;
        Ok(addr)
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
    ) -> Result<()> {
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
                    self.emitter.emit(&Event::Sound { volume, muted });
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

    fn deserialize(buf: [u8; Self::SERIALIZED_LENGTH]) -> Result<Self> {
        match buf[0] {
            1 => {
                let volume = buf[1];
                Ok(Self::Volume(volume))
            }
            2 => {
                let muted = bool::try_from(buf[1]).context("malformed input")?;
                Ok(Self::Mute(muted))
            }

            _ => bail!("malformed input"),
        }
    }
}
