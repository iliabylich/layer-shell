mod auth;
mod decoders;
mod encoders;
mod introspectible_object_at;
mod read_write;
mod requests;
mod serial;

use crate::{liburing::IoUring, user_data::ModuleId};
use anyhow::{Context, Result};
use auth::Auth;
use read_write::ReadWrite;
use std::os::{
    fd::{AsRawFd, IntoRawFd},
    unix::net::UnixStream,
};

pub(crate) mod messages;
pub(crate) mod types;
pub(crate) use introspectible_object_at::{IntrospectibleObjectAt, IntrospectibleObjectAtRequest};
pub(crate) use requests::{Oneshot, OneshotResource, Subscription, SubscriptionResource};
pub(crate) use types::Message;

#[expect(clippy::large_enum_variant)]
pub(crate) enum DBus {
    Auth(Auth),
    ReadWrite(ReadWrite),
}

impl DBus {
    pub(crate) fn new_session() -> Result<Box<Self>> {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
        let (_, path) = address
            .split_once("=")
            .context("malformed DBUS_SESSION_BUS_ADDRESS")?;
        let fd = UnixStream::connect(path)?.into_raw_fd();

        Ok(Box::new(Self::Auth(Auth::new(fd, ModuleId::SessionDBus))))
    }

    pub(crate) fn new_system() -> Result<Box<Self>> {
        let path = std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
            .context("no DBUS_SYSTEM_BUS_ADDRESS")
            .and_then(|address| {
                address
                    .split_once("=")
                    .map(|(_, path)| path.to_string())
                    .context("malformed DBUS_SESSION_BUS_ADDRESS")
            })
            .unwrap_or_else(|_| String::from("/var/run/dbus/system_bus_socket"));

        let fd = UnixStream::connect(path)?.into_raw_fd();

        Ok(Box::new(Self::Auth(Auth::new(fd, ModuleId::SystemDBus))))
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) {
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

    pub(crate) fn feed(&mut self, op_id: u8, res: i32) -> Result<Option<Message<'static>>> {
        match self {
            DBus::Auth(auth) => {
                if auth.feed(op_id, res)? {
                    let fd = auth.as_raw_fd();
                    let queue = auth.take_queue();
                    let serial = auth.take_serial();
                    *self = Self::ReadWrite(ReadWrite::new(fd, queue, serial, auth.module_id));
                }
                Ok(None)
            }
            DBus::ReadWrite(rw) => rw.feed(op_id, res),
        }
    }
}
