mod auth;
mod decoders;
mod encoders;
mod read_write;
mod serial;

use crate::{liburing::IoUring, user_data::UserData};
use anyhow::{Context, Result};
use auth::Auth;
use read_write::ReadWrite;
use std::os::{
    fd::{AsRawFd, IntoRawFd},
    unix::net::UnixStream,
};

pub(crate) mod messages;
pub(crate) mod types;
pub(crate) use messages::BuiltinDBusMessage;
pub(crate) use types::Message;

#[expect(clippy::large_enum_variant)]
pub(crate) enum DBus {
    Auth(Auth),
    ReadWrite(ReadWrite),
}

impl DBus {
    pub(crate) fn new() -> Result<Box<Self>> {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
        let (_, path) = address
            .split_once("=")
            .context("malformed DBUS_SESSION_BUS_ADDRESS")?;
        let fd = UnixStream::connect(path)?.into_raw_fd();

        Ok(Box::new(Self::Auth(Auth::new(fd))))
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        match self {
            Self::Auth(auth) => auth.enqueue(message),
            Self::ReadWrite(rw) => rw.enqueue(message),
        }
    }

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<bool> {
        match self {
            DBus::Auth(auth) => auth.drain(ring),
            DBus::ReadWrite(rw) => rw.drain(ring),
        }
    }

    pub(crate) fn feed(&mut self, user_data: UserData, res: i32) -> Result<Option<Message>> {
        match self {
            DBus::Auth(auth) => {
                if auth.feed(user_data, res)? {
                    let fd = auth.as_raw_fd();
                    let queue = auth.take_queue();
                    let serial = auth.take_serial();
                    *self = Self::ReadWrite(ReadWrite::new(fd, queue, serial));
                }
                Ok(None)
            }
            DBus::ReadWrite(rw) => rw.feed(user_data, res),
        }
    }
}
