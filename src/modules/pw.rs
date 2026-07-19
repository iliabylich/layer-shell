use crate::{
    Event,
    emitter::Emitter,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{getenv, new_sockaddr_un},
};
use anyhow::{Context, Result, bail};
use libc::sockaddr_un;

pub(crate) struct PW {
    reader: Box<UnixSocketReader>,
    buf: Buffer,
    volume: Option<u8>,
    muted: Option<bool>,
    events_left_to_drop: u8,
    emitter: Emitter,
}

impl PW {
    pub(crate) fn address() -> Result<sockaddr_un> {
        let xdg_runtime_dir =
            core::str::from_utf8(getenv(c"XDG_RUNTIME_DIR").context("no $XDG_RUNTIME_DIR")?)?;
        let path = format!("{xdg_runtime_dir}/pipewire-mon.sock");
        let addr = new_sockaddr_un(path.as_bytes())?;
        Ok(addr)
    }

    pub(crate) fn new(emitter: Emitter) -> Self {
        Self {
            reader: Box::new(UnixSocketReader::new()),
            buf: Buffer::new(),
            volume: None,
            muted: None,
            events_left_to_drop: 1,
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

        for event in self.buf.push(bytes) {
            match event {
                PWEvent::Volume(volume) => self.volume = Some(volume),
                PWEvent::Mute(muted) => self.muted = Some(muted),
            }
        }

        if let Some(volume) = self.volume
            && let Some(muted) = self.muted
        {
            if self.events_left_to_drop == 0 {
                self.emitter.emit(&Event::Sound { volume, muted });
            }
            self.events_left_to_drop = self.events_left_to_drop.saturating_sub(1);
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

struct Buffer(Vec<u8>);
impl Buffer {
    const fn new() -> Self {
        Self(vec![])
    }

    fn push(&mut self, bytes: &[u8]) -> Vec<PWEvent> {
        self.0.extend_from_slice(bytes);
        let mut events = vec![];

        while let Some((first, rest)) = self.0.split_first_chunk::<{ PWEvent::SERIALIZED_LENGTH }>()
        {
            match PWEvent::deserialize(*first) {
                Ok(event) => events.push(event),
                Err(err) => {
                    log::error!(target: "PW", "{err:?}");
                }
            }
            self.0 = rest.to_vec();
        }

        events
    }
}
