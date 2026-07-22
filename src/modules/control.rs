use crate::{
    IoEvent,
    emitter::Emitter,
    error::IoError,
    sansio::{Satisfy, Wants},
    utils::write_in_place,
};
use rustix::{
    fd::{BorrowedFd, IntoRawFd},
    fs::Mode,
    io::Errno,
    net::{AddressFamily, SocketAddrUnix, SocketType},
};

pub struct Control {
    fd: BorrowedFd<'static>,
    emitter: Emitter,
}

impl Control {
    pub(crate) fn new(xdg_runtime_dir: &str, emitter: Emitter) -> Option<Self> {
        let mut buf = [0; 200];
        let path = write_in_place!(&mut buf, "{xdg_runtime_dir}/layer-shell.sock");
        match socket_at(path) {
            Ok(fd) => Some(Self { fd, emitter }),
            Err(err) => {
                log::error!(target: "Control", "{err:?}");
                None
            }
        }
    }

    pub(crate) const fn wants(&self) -> Wants {
        Wants::Accept { fd: self.fd }
    }

    pub(crate) fn satisfy(&self, satisfy: Satisfy) -> Result<(), IoError> {
        let Satisfy::Accept(fd) = satisfy else {
            return Err(IoError::WrongSatisfy {
                satisfy: satisfy.as_str(),
                state: "any",
            });
        };
        let fd = fd?;

        let mut buf = [0_u8; 1];
        let bytes_read = rustix::io::read(fd, &mut buf)
            .map_err(|errno| IoError::FailedTo { op: "read", errno })?;

        if bytes_read == 1 {
            let event = match buf[0] {
                b's' => IoEvent::ToggleSessionScreen,
                b'e' => IoEvent::Exit,
                _ => {
                    log::warn!(
                        "Control received unknown command over control UNIX socket: {}",
                        buf[0]
                    );
                    return Ok(());
                }
            };
            self.emitter.emit(&event);
        } else {
            log::error!("failed to read from control UNIX socket client");
        }
        Ok(())
    }
}

pub fn socket_at(socket_path: &[u8]) -> Result<BorrowedFd<'static>, IoError> {
    const SOMAXCONN: i32 = 4_096;

    let addr = SocketAddrUnix::new(socket_path).map_err(|errno| IoError::FailedTo {
        op: "create socket addr",
        errno,
    })?;
    ensure_addr_is_free(&addr)?;

    let fd =
        rustix::net::socket(AddressFamily::UNIX, SocketType::STREAM, None).map_err(|errno| {
            IoError::FailedTo {
                op: "socket",
                errno,
            }
        })?;

    match rustix::fs::unlink(socket_path) {
        Ok(()) => {}
        Err(errno) if errno == Errno::NOENT || errno == Errno::WOULDBLOCK => {}
        Err(errno) => {
            return Err(IoError::FailedTo {
                op: "unlink",
                errno,
            });
        }
    }
    rustix::net::bind(&fd, &addr).map_err(|errno| IoError::FailedTo { op: "bind", errno })?;
    rustix::fs::chmod(socket_path, Mode::from_raw_mode(0o666))
        .map_err(|errno| IoError::FailedTo { op: "chmod", errno })?;
    rustix::net::listen(&fd, SOMAXCONN).map_err(|errno| IoError::FailedTo {
        op: "listen",
        errno,
    })?;

    log::trace!("Listening on {addr:?}");

    let fd = unsafe { BorrowedFd::borrow_raw(fd.into_raw_fd()) };
    Ok(fd)
}

fn ensure_addr_is_free(addr: &SocketAddrUnix) -> Result<(), IoError> {
    let fd =
        rustix::net::socket(AddressFamily::UNIX, SocketType::STREAM, None).map_err(|errno| {
            IoError::FailedTo {
                op: "connect",
                errno,
            }
        })?;
    if rustix::net::connect(&fd, addr).is_ok() {
        return Err(IoError::FailedTo {
            op: "connect",
            errno: Errno::ADDRINUSE,
        });
    }
    Ok(())
}
