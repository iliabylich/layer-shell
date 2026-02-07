mod auth;
mod decoders;
mod encoders;
mod introspectible_object_at;
mod reader;
mod requests;
mod serial;
mod writer;

use crate::{
    dbus::{encoders::MessageEncoder, serial::Serial},
    user_data::ModuleId,
};
use anyhow::{Context, Result};
use auth::Auth;
use reader::Reader;
use std::{
    collections::VecDeque,
    os::{fd::IntoRawFd, unix::net::UnixStream},
};
use writer::Writer;

pub(crate) mod messages;
pub(crate) mod types;
pub(crate) use introspectible_object_at::{IntrospectibleObjectAt, IntrospectibleObjectAtRequest};
pub(crate) use requests::{Oneshot, OneshotResource, Subscription, SubscriptionResource};
pub(crate) use types::Message;

pub(crate) struct DBus {
    auth: Auth,
    reader: Reader,
    writer: Writer,
    writer_is_ready: bool,
    queue: VecDeque<Vec<u8>>,
    serial: Serial,
}

impl DBus {
    pub(crate) fn new_session() -> Result<Box<Self>> {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
        let (_, path) = address
            .split_once("=")
            .context("malformed DBUS_SESSION_BUS_ADDRESS")?;
        let fd = UnixStream::connect(path)?.into_raw_fd();

        Ok(Box::new(Self::new(
            fd,
            ModuleId::SessionDBusAuth,
            ModuleId::SessionDBusReader,
            ModuleId::SessionDBusWriter,
        )))
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

        Ok(Box::new(Self::new(
            fd,
            ModuleId::SystemDBusAuth,
            ModuleId::SystemDBusReader,
            ModuleId::SystemDBusWriter,
        )))
    }

    fn new(
        fd: i32,
        auth_module_id: ModuleId,
        reader_module_id: ModuleId,
        writer_module_id: ModuleId,
    ) -> Self {
        Self {
            auth: Auth::new(fd, auth_module_id),
            reader: Reader::new(fd, reader_module_id),
            writer: Writer::new(fd, writer_module_id),
            writer_is_ready: false,
            queue: VecDeque::new(),
            serial: Serial::zero(),
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) {
        *message.serial_mut() = self.serial.increment_and_get();
        let bytes = MessageEncoder::encode(message);

        if self.writer_is_ready {
            self.writer.init(bytes);
            self.writer_is_ready = false;
        } else {
            self.queue.push_back(bytes);
        }
    }

    pub(crate) fn init(&mut self) {
        self.auth.init()
    }

    pub(crate) fn process_auth(&mut self, op: u8, res: i32) -> Result<()> {
        let finished = self.auth.process(op, res)?;
        if finished {
            self.reader.init();

            if let Some(bytes) = self.queue.pop_front() {
                self.writer.init(bytes);
                self.writer_is_ready = false;
            } else {
                self.writer_is_ready = true;
            }
        }
        Ok(())
    }

    pub(crate) fn process_read(&mut self, op: u8, res: i32) -> Result<Option<Message<'static>>> {
        self.reader.process(op, res)
    }

    pub(crate) fn process_write(&mut self, op: u8, res: i32) -> Result<()> {
        self.writer.process(op, res)?;
        if let Some(bytes) = self.queue.pop_front() {
            self.writer.init(bytes);
        } else {
            self.writer_is_ready = true;
        }
        Ok(())
    }
}
