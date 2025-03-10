use anyhow::{Result, bail};
use libc::{
    EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLLIN, c_int, close, epoll_create, epoll_ctl, epoll_event,
    epoll_wait,
};

pub(crate) trait Reader {
    type Output;

    const NAME: &str;

    fn read(&mut self) -> Result<Self::Output>;
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn fd(&self) -> i32;
    fn fd_id(&self) -> FdId;
}

pub(crate) struct Epoll {
    epfd: i32,
}

impl Epoll {
    pub(crate) fn new() -> Result<Self> {
        let fd = unsafe { epoll_create(1) };
        if fd == -1 {
            Err(anyhow::Error::from(std::io::Error::last_os_error()).context("epoll_create failed"))
        } else {
            Ok(Self { epfd: fd })
        }
    }

    pub(crate) fn add_reader<T>(&mut self, reader: &T) -> Result<()>
    where
        T: Reader,
    {
        let id = reader.fd_id();
        if id == FdId::Disconnected {
            return Ok(());
        }
        let fd = reader.fd();
        self.add_reader_fd(fd, id)
    }

    fn add_reader_fd(&mut self, fd: i32, id: FdId) -> Result<()> {
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

    pub(crate) fn remove_reader<T>(&mut self, reader: &T)
    where
        T: Reader,
    {
        let id = reader.fd_id();
        let fd = reader.fd();

        let res = unsafe {
            epoll_ctl(
                self.epfd,
                EPOLL_CTL_DEL,
                fd,
                &mut epoll_event {
                    events: EPOLLIN as u32,
                    u64: id as u64,
                },
            )
        };
        println!("removed: {:?}", res);
    }

    pub(crate) fn poll(&self, buf: &mut Vec<epoll_event>) -> Result<()> {
        buf.clear();
        let len = unsafe { epoll_wait(self.epfd, buf.as_mut_ptr(), 1024, -1 as c_int) };
        if len == -1 {
            return Err(
                anyhow::Error::from(std::io::Error::last_os_error()).context("epoll_wait failed")
            );
        }

        unsafe { buf.set_len(len as usize) };
        Ok(())
    }

    pub(crate) fn read_from_or_disable<T>(&mut self, reader: &mut T) -> Option<T::Output>
    where
        T: Reader,
    {
        match reader.read() {
            Ok(output) => Some(output),
            Err(err) => {
                log::error!(
                    "module {} has crashes, disabling it: {err:?}",
                    reader.name()
                );
                self.remove_reader(reader);
                None
            }
        }
    }
}
impl Drop for Epoll {
    fn drop(&mut self) {
        unsafe { close(self.epfd) };
    }
}

#[derive(Debug, PartialEq)]
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

    Disconnected,
}
impl TryFrom<u64> for FdId {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value >= Self::Disconnected as u64 {
            bail!("invalid fd id {value}");
        }
        Ok(unsafe { std::mem::transmute::<u64, Self>(value) })
    }
}
