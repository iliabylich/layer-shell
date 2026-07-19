use crate::{
    Event,
    emitter::Emitter,
    sansio::{Satisfy, Wants},
    utils::getenv,
};
use anyhow::{Context, Result, bail};
use rustix::{
    fd::{AsRawFd, OwnedFd},
    fs::Mode,
    io::Errno,
    net::{AddressFamily, SocketAddrUnix, SocketType},
};

pub(crate) struct Control {
    fd: OwnedFd,
    emitter: Emitter,
}

impl Control {
    pub(crate) fn new(emitter: Emitter) -> Result<Self> {
        let xdg_runtime_dir =
            core::str::from_utf8(getenv(c"XDG_RUNTIME_DIR").context("no $XDG_RUNTIME_DIR")?)?;
        let path = format!("{xdg_runtime_dir}/layer-shell.sock");
        let fd = socket_at(&path)?;
        Ok(Self { fd, emitter })
    }

    pub(crate) fn wants(&self) -> Wants {
        Wants::Accept {
            fd: self.fd.as_raw_fd(),
        }
    }

    pub(crate) fn satisfy(&self, satisfy: Satisfy) -> Result<()> {
        let Satisfy::Accept(fd) = satisfy else {
            bail!("Control may only process prep_accept, received: {satisfy:?}");
        };
        let fd = fd?;

        let mut buf = [0_u8; 1];
        let bytes_read = unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) };
        unsafe { libc::close(fd) };

        if bytes_read == 1 {
            let event = match buf[0] {
                b's' => Event::ToggleSessionScreen,
                b'e' => Event::Exit,
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

pub fn socket_at(socket_path: &str) -> Result<OwnedFd> {
    const SOMAXCONN: i32 = 4_096;

    let addr = SocketAddrUnix::new(socket_path).map_err(|err| anyhow::anyhow!(err))?;
    ensure_addr_is_free(&addr)?;

    let fd = rustix::net::socket(AddressFamily::UNIX, SocketType::STREAM, None)
        .map_err(|err| anyhow::anyhow!(err))?;

    match rustix::fs::unlink(socket_path) {
        Ok(()) => {}
        Err(err) if err == Errno::NOENT => {}
        Err(err) if err == Errno::WOULDBLOCK => {}
        Err(err) => return Err(anyhow::anyhow!(err)),
    }
    rustix::net::bind(&fd, &addr).map_err(|err| anyhow::anyhow!(err))?;
    rustix::fs::chmod(socket_path, Mode::from_raw_mode(0o666))
        .map_err(|err| anyhow::anyhow!(err))?;
    rustix::net::listen(&fd, SOMAXCONN).map_err(|err| anyhow::anyhow!(err))?;

    log::trace!("Listening on {socket_path}");
    Ok(fd)
}

fn ensure_addr_is_free(addr: &SocketAddrUnix) -> Result<()> {
    let fd = rustix::net::socket(AddressFamily::UNIX, SocketType::STREAM, None)
        .map_err(|err| anyhow::anyhow!(err))?;
    if rustix::net::connect(&fd, addr).is_ok() {
        bail!("there's already a server listenning at {addr:?}");
    }
    Ok(())
}
