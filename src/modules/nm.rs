use crate::{
    IoEvent,
    emitter::Emitter,
    error::IoError,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{FixedSizeBuffer, SockaddrUn, StringRef, StringRefExt},
};
use libc::sockaddr_un;
use thiserror::Error;

#[derive(Clone, Copy)]
pub struct NM {
    reader: UnixSocketReader,
    emitter: Emitter,
}

impl NM {
    pub(crate) const BUFFER_SIZE: usize = NMEvent::SERIALIZED_LENGTH;
    const SPEED_THRESHOLD: u64 = 5_000;
    pub(crate) const ADDRESS: sockaddr_un = SockaddrUn::from_bytes(b"/run/nm-mon-systemd.sock");

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
    ) -> Result<(), IoError> {
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

    fn deserialize(buf: [u8; Self::SERIALIZED_LENGTH]) -> Result<Self, NMError> {
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
                    let bytes = buf[8..].get(..len).ok_or(NMError::SsidIsTooLong { len })?;
                    let ssid = core::str::from_utf8(bytes).map_err(NMError::NonUtf8Ssid)?;
                    StringRef::new(ssid)
                },
                strength: buf[1],
            }),

            kind => Err(NMError::MalformedByteSequence { kind }),
        }
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum NMError {
    #[error("message length too big: {len}")]
    SsidIsTooLong { len: usize },
    #[error("non-utf8 SSID")]
    NonUtf8Ssid(core::str::Utf8Error),
    #[error("malformed network-manager byte sequence: {kind}")]
    MalformedByteSequence { kind: u8 },
}
