use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{StringRef, StringRefExt, new_sockaddr_un},
};
use anyhow::{Context, Result, bail};
use libc::sockaddr_un;

pub(crate) struct NM {
    reader: Box<UnixSocketReader>,
    buf: Buffer,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ProxyUntilRead,
    WaitingForWrite(Wants),
    WriteFinished(Wants),
    Proxy,
}

impl NM {
    const SPEED_THRESHOLD: u64 = 5_000;

    pub(crate) fn address() -> Result<sockaddr_un> {
        let addr = new_sockaddr_un(b"/run/nm-mon-systemd.sock")?;
        Ok(addr)
    }

    pub(crate) fn new() -> Self {
        Self {
            reader: Box::new(UnixSocketReader::new()),
            buf: Buffer::new(),
            state: State::ProxyUntilRead,
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
                    let event = match event {
                        NMEvent::UploadSpeed { mut bytes_per_sec } => {
                            if bytes_per_sec < Self::SPEED_THRESHOLD {
                                bytes_per_sec = 0;
                            }
                            Event::UploadSpeed { bytes_per_sec }
                        }
                        NMEvent::DownloadSpeed { mut bytes_per_sec } => {
                            if bytes_per_sec < Self::SPEED_THRESHOLD {
                                bytes_per_sec = 0;
                            }
                            Event::DownloadSpeed { bytes_per_sec }
                        }
                        NMEvent::SsidAndStrength { ssid, strength } => {
                            Event::NetworkSsidAndStrength {
                                ssid: ssid.clone(),
                                strength,
                            }
                        }
                    };

                    events.push_back(event);
                }

                Ok(())
            }

            _ => bail!("KbMod only accepts Socket, Connect and Read, got: {satisfy:?}"),
        }
    }
}

enum NMEvent {
    UploadSpeed { bytes_per_sec: u64 },
    DownloadSpeed { bytes_per_sec: u64 },
    SsidAndStrength { ssid: StringRef, strength: u8 },
}

impl NMEvent {
    const SERIALIZED_LENGTH: usize = 32;

    fn deserialize(buf: [u8; Self::SERIALIZED_LENGTH]) -> Result<Self> {
        match buf[0] {
            1 => Ok(Self::UploadSpeed {
                bytes_per_sec: u64::from_be_bytes([
                    buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
                ]),
            }),

            2 => Ok(Self::DownloadSpeed {
                bytes_per_sec: u64::from_be_bytes([
                    buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
                ]),
            }),

            3 => Ok(Self::SsidAndStrength {
                ssid: {
                    let len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
                    let bytes = &buf[8..].get(..len).context("message length is too big")?;
                    let ssid = core::str::from_utf8(bytes).context("non-utf SSID")?;
                    StringRef::new(ssid)
                },
                strength: buf[1],
            }),

            _ => {
                bail!("malformed byte sequence: {buf:?}")
            }
        }
    }
}

struct Buffer(Vec<u8>);
impl Buffer {
    const fn new() -> Self {
        Self(vec![])
    }

    fn push(&mut self, bytes: &[u8]) -> Vec<NMEvent> {
        self.0.extend_from_slice(bytes);
        let mut events = vec![];

        while let Some((first, rest)) = self.0.split_first_chunk::<{ NMEvent::SERIALIZED_LENGTH }>()
        {
            match NMEvent::deserialize(*first) {
                Ok(event) => events.push(event),
                Err(err) => {
                    log::error!(target: "NM", "{err:?}");
                }
            }
            self.0 = rest.to_vec();
        }

        events
    }
}
