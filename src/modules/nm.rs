use crate::{
    IoEvent,
    emitter::Emitter,
    module_id::ModuleId,
    modules::Module,
    utils::{FixedSizeBuffer, StringRef, StringRefExt, unix_socket, unix_socket_addr},
};
use rustix::fd::{AsFd, BorrowedFd, OwnedFd};

pub struct NM {
    fd: OwnedFd,
    buf: FixedSizeBuffer<{ NMEvent::SERIALIZED_LENGTH }>,
}

impl NM {
    const SPEED_THRESHOLD: u64 = 5_000;

    pub(crate) fn new() -> Option<Self> {
        log::trace!("Creating NM");

        let addr = unix_socket_addr!("/run/nm-mon-systemd.sock")
            .unwrap_or_else(|err| panic!("failed to create UNIX socket: {err:?}"));

        let fd = unix_socket!().ok()?;

        if let Err(err) = rustix::net::connect(&fd, &addr) {
            log::error!("failed to connect(): {err:?}");
            return None;
        }

        Some(Self {
            fd,
            buf: FixedSizeBuffer::new(),
        })
    }
}

impl Module for NM {
    fn read(&mut self, emitter: Emitter) -> Result<(), ()> {
        let count = rustix::io::read(&self.fd, self.buf.remainder())
            .map_err(|err| log::error!("failed to read(): {err:?}"))?;
        let Some(buf) = self.buf.written(count) else {
            return Ok(());
        };

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

        emitter.emit(&event);

        Ok(())
    }

    fn id(&self) -> ModuleId {
        ModuleId::NM
    }

    const MODULE_ID: ModuleId = ModuleId::NM;
}

impl AsFd for NM {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

enum NMEvent {
    UploadSpeed { bytes_per_sec: u64 },
    DownloadSpeed { bytes_per_sec: u64 },
    SsidAndStrength { ssid: StringRef, strength: u8 },
}

impl NMEvent {
    const SERIALIZED_LENGTH: usize = 32;

    fn deserialize(buf: [u8; Self::SERIALIZED_LENGTH]) -> Result<Self, ()> {
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
                    let bytes = buf[8..].get(..len).ok_or_else(|| {
                        log::error!("SSID is too long: {len}");
                    })?;
                    let ssid = core::str::from_utf8(bytes).map_err(|err| {
                        log::error!("non-utf8 SSID: {err:?}");
                    })?;
                    StringRef::new(ssid)
                },
                strength: buf[1],
            }),

            kind => {
                log::error!("malformed byte sequence, starts with {kind}");
                Err(())
            }
        }
    }
}
