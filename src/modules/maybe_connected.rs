use crate::epoll::{FdId, Reader};
use anyhow::Result;
use std::os::fd::RawFd;

pub(crate) struct MaybeConnected<T> {
    module: Option<T>,
}

impl<T> MaybeConnected<T>
where
    T: Reader,
{
    pub(crate) fn new(module: Result<T>) -> Self {
        match module {
            Ok(module) => Self {
                module: Some(module),
            },
            Err(err) => {
                log::error!("failed to instantiate module {}: {err:?}", T::NAME);
                Self { module: None }
            }
        }
    }

    pub(crate) fn with(&mut self, f: impl FnOnce(&mut T) -> Result<()>) {
        if let Some(module) = self.module.as_mut() {
            if let Err(err) = f(module) {
                log::error!(
                    "Got error while processiong command of {}: {err:?}",
                    module.name()
                );
            }
        } else {
            log::error!("module {} is not connected", T::NAME)
        }
    }
}

impl<T> Reader for MaybeConnected<T>
where
    T: Reader,
{
    type Output = T::Output;

    const NAME: &str = "Disconnected";

    fn read(&mut self) -> Result<Self::Output> {
        if let Some(module) = self.module.as_mut() {
            module.read()
        } else {
            unreachable!("can't read from disconnected {}", T::NAME)
        }
    }

    fn name(&self) -> &'static str {
        if let Some(module) = self.module.as_ref() {
            module.name()
        } else {
            Self::NAME
        }
    }

    fn fd(&self) -> RawFd {
        if let Some(module) = self.module.as_ref() {
            module.fd()
        } else {
            unreachable!("can't get fd of disconnected {}", T::NAME)
        }
    }

    fn fd_id(&self) -> FdId {
        if let Some(module) = self.module.as_ref() {
            module.fd_id()
        } else {
            FdId::Disconnected
        }
    }
}
