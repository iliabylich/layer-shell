use crate::{
    IoEvent,
    emitter::Emitter,
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
        log::trace!("Creating Control");

        let mut buf = [0; 200];
        let path = write_in_place!(&mut buf, "{xdg_runtime_dir}/layer-shell.sock");
        match socket_at(path) {
            Ok(fd) => Some(Self { fd, emitter }),
            Err(err) => {
                log::error!("{err:?}");
                None
            }
        }
    }

    pub(crate) fn wants(&self) -> Wants {
        let wants = Wants::Accept { fd: self.fd };
        log::trace!("{wants:?}");
        wants
    }

    pub(crate) fn satisfy(&self, satisfy: Satisfy) -> Result<(), ()> {
        let Satisfy::Accept(fd) = satisfy else {
            log::error!("wrong satisfy {satisfy:?} (expected Accept)");
            return Err(());
        };
        let fd = fd?;

        let mut buf = [0_u8; 1];
        let bytes_read = rustix::io::read(fd, &mut buf).map_err(|errno| {
            log::error!("failed to read: {errno:?}");
        })?;

        if bytes_read == 1 {
            let event = match buf[0] {
                b's' => IoEvent::ToggleSessionScreen,
                b'e' => IoEvent::Exit,
                _ => {
                    log::warn!("received unknown command over UNIX socket: {}", buf[0]);
                    return Ok(());
                }
            };
            self.emitter.emit(&event);
        } else {
            log::error!("failed to read from the UNIX socket client");
        }
        Ok(())
    }
}

pub fn socket_at(socket_path: &[u8]) -> Result<BorrowedFd<'static>, ()> {
    const SOMAXCONN: i32 = 4_096;

    let addr = SocketAddrUnix::new(socket_path).map_err(|errno| {
        log::error!("failed to create socket addr: {errno:?}");
    })?;
    ensure_addr_is_free(&addr)?;

    let fd =
        rustix::net::socket(AddressFamily::UNIX, SocketType::STREAM, None).map_err(|errno| {
            log::error!("failed to socket: {errno:?}");
        })?;

    match rustix::fs::unlink(socket_path) {
        Ok(()) => {}
        Err(errno) if errno == Errno::NOENT || errno == Errno::WOULDBLOCK => {}
        Err(errno) => {
            log::error!("failed to unlink: {errno:?}");
            return Err(());
        }
    }
    rustix::net::bind(&fd, &addr).map_err(|errno| {
        log::error!("failed to bind: {errno:?}");
    })?;
    rustix::fs::chmod(socket_path, Mode::from_raw_mode(0o666)).map_err(|errno| {
        log::error!("failed to chmod: {errno:?}");
    })?;
    rustix::net::listen(&fd, SOMAXCONN).map_err(|errno| {
        log::error!("failed to listen: {errno:?}");
    })?;

    log::trace!("Listening on {addr:?}");

    let fd = unsafe { BorrowedFd::borrow_raw(fd.into_raw_fd()) };
    Ok(fd)
}

fn ensure_addr_is_free(addr: &SocketAddrUnix) -> Result<(), ()> {
    let fd =
        rustix::net::socket(AddressFamily::UNIX, SocketType::STREAM, None).map_err(|errno| {
            log::error!("failed to connect: {errno:?}");
        })?;
    if rustix::net::connect(&fd, addr).is_ok() {
        log::error!("address in use");
        return Err(());
    }
    Ok(())
}
