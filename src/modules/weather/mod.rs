use crate::{
    Event, VerboseSender,
    epoll::{FdId, Reader},
};
use anyhow::{Context as _, Result};
use std::os::fd::RawFd;

mod client;
mod code;
mod mapper;

pub use code::WeatherCode;

pub(crate) struct Weather {
    tx: VerboseSender<Event>,
    fd: Option<RawFd>,
}

impl Weather {
    pub(crate) const INTERVAL: u64 = 60 * 30;

    pub(crate) fn new(tx: VerboseSender<Event>) -> Self {
        Self { tx, fd: None }
    }

    pub(crate) fn refresh(&mut self) -> bool {
        match client::send_request() {
            Ok(fd) => {
                self.fd = Some(fd);
                true
            }
            Err(err) => {
                log::error!("failed to get weather: {:?}", err);
                self.fd = None;
                false
            }
        }
    }
}

impl Reader for Weather {
    type Output = ();

    const NAME: &str = "Weather";

    fn read(&mut self) -> Result<Self::Output> {
        let fd = self.fd.context("can't read from no fd")?;
        let res = client::read_response(fd)?;
        let (current, forecast) = mapper::map(res)?;
        self.tx.send(current);
        self.tx.send(forecast);
        Ok(())
    }

    fn fd(&self) -> RawFd {
        self.fd.unwrap_or(-1)
    }

    fn fd_id(&self) -> FdId {
        FdId::Weather
    }
}
