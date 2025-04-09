use crate::{fd_id::FdId, modules::Module};
use anyhow::{Context as _, Result};
use mio::{Events, Interest, unix::SourceFd};
use std::os::fd::RawFd;

pub(crate) struct Poll {
    poll: mio::Poll,
}

impl Poll {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            poll: mio::Poll::new().context("failed to create poll")?,
        })
    }

    pub(crate) fn add_fd(&self, fd: RawFd, fd_id: FdId) -> Result<()> {
        let token = fd_id.token();
        self.poll
            .registry()
            .register(&mut SourceFd(&fd), token, Interest::READABLE)
            .with_context(|| {
                format!("failed to register {fd} with {token:?} ({fd_id:?}) in epoll")
            })?;
        log::info!("[epoll] registered fd {fd} with token {token:?} ({fd_id:?})");
        Ok(())
    }

    pub(crate) fn add_reader<T>(&self, module: &T) -> Result<()>
    where
        T: Module,
    {
        self.add_fd(module.as_raw_fd(), T::FD_ID)
    }

    pub(crate) fn remove_fd(&self, fd: RawFd) {
        if let Err(err) = self.poll.registry().deregister(&mut SourceFd(&fd)) {
            log::error!("[epoll] failed to un-register {fd} from epoll: {err:?}");
        }
    }

    pub(crate) fn remove_reader<T>(&self, module: &T)
    where
        T: Module,
    {
        self.remove_fd(module.as_raw_fd());
    }

    pub(crate) fn poll(&mut self, events: &mut Events) -> Result<()> {
        self.poll.poll(events, None).context("failed to poll epoll")
    }
}
