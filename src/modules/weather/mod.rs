use crate::{channel::EventSender, fd_id::FdId, modules::Module};
use anyhow::Result;
use std::{
    net::TcpStream,
    os::fd::{AsRawFd, RawFd},
};

mod client;
mod code;
mod mapper;
pub(crate) use code::WeatherCode;

pub(crate) struct Weather {
    tx: EventSender,
    stream: TcpStream,
}

impl Module for Weather {
    const FD_ID: FdId = FdId::Weather;
    const NAME: &str = "Weather";

    type ReadOutput = ();

    fn new(tx: &EventSender) -> Result<Self> {
        let fd = client::send_request()?;
        Ok(Self {
            tx: tx.clone(),
            stream: fd,
        })
    }

    fn read_events(&mut self) -> Result<()> {
        let res = client::read_response(&mut self.stream)?;
        let (current, forecast) = mapper::map(res)?;
        self.tx.send(current);
        self.tx.send(forecast);
        Ok(())
    }
}

impl Weather {
    pub(crate) const INTERVAL: u64 = 60 * 30;
}

impl AsRawFd for Weather {
    fn as_raw_fd(&self) -> RawFd {
        self.stream.as_raw_fd()
    }
}
