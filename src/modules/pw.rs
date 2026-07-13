use crate::{
    Event,
    event_queue::EventQueue,
    external::sockaddr_un,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{getenv, new_sockaddr_un},
};
use anyhow::{Context, Result, bail};

pub(crate) struct PW {
    reader: Box<UnixSocketReader>,
    buf: Buffer,
    state: State,
    volume: Option<u8>,
    muted: Option<bool>,
    ignored_first_event: bool,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ProxyUntilRead,
    WaitingForWrite(Wants),
    WriteFinished(Wants),
    Proxy,
}

impl PW {
    pub(crate) fn address() -> Result<sockaddr_un> {
        let xdg_runtime_dir =
            core::str::from_utf8(getenv(c"XDG_RUNTIME_DIR").context("no $XDG_RUNTIME_DIR")?)?;
        let path = format!("{xdg_runtime_dir}/pipewire-mon.sock");
        let addr = new_sockaddr_un(path.as_bytes())?;
        Ok(addr)
    }

    pub(crate) fn new() -> Self {
        Self {
            reader: Box::new(UnixSocketReader::new()),
            buf: Buffer::new(),
            state: State::ProxyUntilRead,
            volume: None,
            muted: None,
            ignored_first_event: false,
        }
    }

    pub(crate) fn wants(&mut self, addr: &sockaddr_un) -> Option<Wants> {
        match self.state {
            State::ProxyUntilRead => {
                let wants = self.reader.wants(addr);
                if let Some(wants @ Wants::Read { fd, .. }) = wants {
                    self.state = State::WaitingForWrite(wants);
                    Some(Wants::Write {
                        fd,
                        buf: b"1".as_ptr(),
                        len: 1,
                    })
                } else {
                    wants
                }
            }
            State::WaitingForWrite(_) => None,
            State::WriteFinished(wants) => {
                self.state = State::Proxy;
                Some(wants)
            }
            State::Proxy => self.reader.wants(addr),
        }
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

            Satisfy::Write(res) => {
                res?;
                if let State::WaitingForWrite(wants) = self.state {
                    self.state = State::WriteFinished(wants);
                } else {
                    bail!("malformed state");
                }
                Ok(())
            }

            Satisfy::Read(res) => {
                let bytes_read = res?;
                let (buf, len) = self.reader.satisfy_read(bytes_read)?;
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
                    if self.ignored_first_event {
                        events.push_back(Event::Sound { volume, muted });
                    } else {
                        self.ignored_first_event = true;
                    }
                }

                Ok(())
            }

            _ => bail!("KbMod only accepts Socket, Connect and Read, got: {satisfy:?}"),
        }
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
