use crate::{
    IoEvent,
    emitter::Emitter,
    module_id::ModuleId,
    modules::Module,
    utils::{unix_socket, unix_socket_addr, write_in_place},
};
use rustix::{
    fd::{AsFd, BorrowedFd, OwnedFd},
    fs::Mode,
    io::Errno,
};

pub struct Control {
    fd: OwnedFd,
}

impl Control {
    pub(crate) fn new(xdg_runtime_dir: &str) -> Option<Self> {
        log::trace!("Creating Control");

        let mut buf = [0; 200];
        let path = write_in_place!(&mut buf, "{xdg_runtime_dir}/layer-shell.sock");

        ensure_addr_is_free(path).ok()?;
        let fd = socket_at(path).ok()?;

        Some(Self { fd })
    }
}

impl Module for Control {
    fn read(&mut self, emitter: Emitter) -> Result<(), ()> {
        let fd = rustix::net::accept(&self.fd)
            .map_err(|err| log::error!("failed to accept(): {err:?}"))?;

        let mut buf = [0_u8; 1];
        let len = rustix::io::read(fd, &mut buf).map_err(|err| {
            log::error!("failed to read: {err:?}");
        })?;

        if len != 1 {
            log::error!("failed to read from the UNIX socket client");
            return Err(());
        }

        let event = match buf[0] {
            b's' => IoEvent::ToggleSessionScreen,
            b'e' => IoEvent::Exit,
            _ => {
                log::warn!("received unknown command over UNIX socket: {}", buf[0]);
                return Ok(());
            }
        };
        emitter.emit(&event);
        Ok(())
    }

    fn id(&self) -> ModuleId {
        ModuleId::Control
    }

    const MODULE_ID: ModuleId = ModuleId::Control;
}

impl AsFd for Control {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

pub fn socket_at(path: &str) -> Result<OwnedFd, ()> {
    const SOMAXCONN: i32 = 4_096;

    let addr = unix_socket_addr!("{path}")?;
    let fd = unix_socket!()?;

    match rustix::fs::unlink(path) {
        Ok(()) => {}
        Err(errno) if errno == Errno::NOENT || errno == Errno::WOULDBLOCK => {}
        Err(errno) => {
            log::error!("failed to unlink: {errno:?}");
            return Err(());
        }
    }
    rustix::net::bind(&fd, &addr).map_err(|err| {
        log::error!("failed to bind: {err:?}");
    })?;
    rustix::fs::chmod(path, Mode::from_raw_mode(0o666)).map_err(|err| {
        log::error!("failed to chmod: {err:?}");
    })?;
    rustix::net::listen(&fd, SOMAXCONN).map_err(|err| {
        log::error!("failed to listen: {err:?}");
    })?;

    log::trace!("Listening on {addr:?}");

    Ok(fd)
}

fn ensure_addr_is_free(path: &str) -> Result<(), ()> {
    let addr = unix_socket_addr!("{path}")?;
    let fd = unix_socket!()?;
    if rustix::net::connect(&fd, &addr).is_ok() {
        log::error!("address in use");
        return Err(());
    }
    Ok(())
}
