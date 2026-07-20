use crate::{
    IoEvent,
    emitter::Emitter,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{FixedSizeBuffer, StringRef, StringRefExt, new_sockaddr_un},
};
use anyhow::{Context, Result, bail};
use libc::sockaddr_un;

#[derive(Clone, Copy)]
pub(crate) struct NM {
    reader: UnixSocketReader,
    emitter: Emitter,
}

impl NM {
    pub(crate) const BUFFER_SIZE: usize = NMEvent::SERIALIZED_LENGTH;
    const SPEED_THRESHOLD: u64 = 5_000;

    pub(crate) fn address() -> Result<sockaddr_un> {
        let addr = new_sockaddr_un(b"/run/nm-mon-systemd.sock")?;
        Ok(addr)
    }

    pub(crate) const fn new(emitter: Emitter) -> Self {
        Self {
            reader: UnixSocketReader::new(),
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
            let event = NMEvent::deserialize(buf)?;
            let event = match event {
                NMEvent::UploadSpeed { mut bytes_per_sec } => {
                    if bytes_per_sec < Self::SPEED_THRESHOLD {
                        bytes_per_sec = 0;
                    }
                    IoEvent::UploadSpeed { bytes_per_sec }
                }
                NMEvent::DownloadSpeed { mut bytes_per_sec } => {
                    if bytes_per_sec < Self::SPEED_THRESHOLD {
                        bytes_per_sec = 0;
                    }
                    IoEvent::DownloadSpeed { bytes_per_sec }
                }
                NMEvent::SsidAndStrength { ssid, strength } => IoEvent::NetworkSsidAndStrength {
                    ssid: ssid.clone(),
                    strength,
                },
            };

            self.emitter.emit(&event);
        }
        Ok(())
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
