mod auth;
mod connector;
mod decoders;
mod encoders;
mod introspectible_object_at;
mod reader;
mod requests;
mod serial;
mod writer;

use crate::dbus::{encoders::MessageEncoder, serial::Serial};
use auth::Auth;
use connector::Connector;
use reader::Reader;
use std::collections::VecDeque;
use writer::Writer;

pub(crate) mod messages;
pub(crate) mod types;
pub(crate) use introspectible_object_at::{IntrospectibleObjectAt, IntrospectibleObjectAtRequest};
pub(crate) use requests::{Oneshot, OneshotResource, Subscription, SubscriptionResource};
pub(crate) use types::Message;

#[derive(Clone, Copy, Debug)]
pub(crate) enum ConnectionKind {
    Session,
    System,
}

pub(crate) struct DBus {
    fd: i32,
    connector: Connector,
    auth: Auth,
    reader: Reader,
    writer: Writer,
    writer_is_ready: bool,
    queue: VecDeque<Vec<u8>>,
    serial: Serial,
}

impl DBus {
    pub(crate) fn new_session() -> Box<Self> {
        Box::new(Self::new(ConnectionKind::Session))
    }

    pub(crate) fn new_system() -> Box<Self> {
        Box::new(Self::new(ConnectionKind::System))
    }

    fn new(kind: ConnectionKind) -> Self {
        Self {
            fd: -1,
            connector: Connector::new(kind),
            auth: Auth::new(kind),
            reader: Reader::new(kind),
            writer: Writer::new(kind),
            writer_is_ready: false,
            queue: VecDeque::new(),
            serial: Serial::zero(),
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) {
        *message.serial_mut() = self.serial.increment_and_get();
        let bytes = MessageEncoder::encode(message);

        if self.writer_is_ready {
            self.writer.init(self.fd, bytes);
            self.writer_is_ready = false;
        } else {
            self.queue.push_back(bytes);
        }
    }

    pub(crate) fn init(&mut self) {
        self.connector.init();
    }

    pub(crate) fn process_connector(&mut self, op: u8, res: i32) {
        if let Some(fd) = self.connector.process(op, res) {
            self.fd = fd;
            self.auth.init(fd);
        }
    }

    pub(crate) fn process_auth(&mut self, op: u8, res: i32) {
        let finished = self.auth.process(op, res);
        if finished {
            self.reader.init(self.fd);

            if let Some(bytes) = self.queue.pop_front() {
                self.writer.init(self.fd, bytes);
                self.writer_is_ready = false;
            } else {
                self.writer_is_ready = true;
            }
        }
    }

    pub(crate) fn process_read(&mut self, op: u8, res: i32) -> Option<Message<'static>> {
        self.reader.process(op, res)
    }

    pub(crate) fn process_write(&mut self, op: u8, res: i32) {
        self.writer.process(op, res);
        if let Some(bytes) = self.queue.pop_front() {
            self.writer.init(self.fd, bytes);
        } else {
            self.writer_is_ready = true;
        }
    }
}
