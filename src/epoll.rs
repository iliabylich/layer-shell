use anyhow::{bail, Result};
use libc::{
    c_int, close, epoll_create, epoll_ctl, epoll_event, epoll_wait, EPOLLIN, EPOLL_CTL_ADD,
};
use std::os::fd::AsRawFd;

pub(crate) struct Epoll {
    epfd: i32,
    buffer: Vec<epoll_event>,
}

impl Epoll {
    pub(crate) fn new() -> Result<Self> {
        let fd = unsafe { epoll_create(1) };
        if fd == -1 {
            Err(anyhow::Error::from(std::io::Error::last_os_error()).context("epoll_create failed"))
        } else {
            Ok(Self {
                epfd: fd,
                buffer: Vec::with_capacity(1024),
            })
        }
    }

    pub(crate) fn add_reader(&mut self, reader: &impl AsRawFd, id: FdId) -> Result<()> {
        self.add_reader_fd(reader.as_raw_fd(), id)
    }

    pub(crate) fn add_reader_fd(&mut self, fd: i32, id: FdId) -> Result<()> {
        let res = unsafe {
            epoll_ctl(
                self.epfd,
                EPOLL_CTL_ADD,
                fd,
                &mut epoll_event {
                    events: EPOLLIN as u32,
                    u64: id as u64,
                },
            )
        };
        if res == -1 {
            Err(anyhow::Error::from(std::io::Error::last_os_error()).context("epoll_ctl failed"))
        } else {
            Ok(())
        }
    }

    pub(crate) fn poll(&mut self) -> Result<&[epoll_event]> {
        self.buffer.clear();
        let len = unsafe { epoll_wait(self.epfd, self.buffer.as_mut_ptr(), 1024, -1 as c_int) };
        if len == -1 {
            return Err(
                anyhow::Error::from(std::io::Error::last_os_error()).context("epoll_wait failed")
            );
        }

        unsafe { self.buffer.set_len(len as usize) };
        Ok(&self.buffer)
    }
}
impl Drop for Epoll {
    fn drop(&mut self) {
        unsafe { close(self.epfd) };
    }
}

#[repr(u64)]
pub(crate) enum FdId {
    Timer,
    HyprlandSocket,
    Command,
    ControlDBus,
    PipewireDBus,
    LauncherGlobalDirInotify,
    LauncherUserDirInotify,
    NetworkDBus,
    TrayDBus,

    Last,
}
impl TryFrom<u64> for FdId {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value >= Self::Last as u64 {
            bail!("invalid fd id {value}");
        }
        Ok(unsafe { std::mem::transmute::<u64, Self>(value) })
    }
}
