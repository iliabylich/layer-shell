use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail};
use connector::DBusConnector;
use libc::sockaddr_un;
use reader::DBusReader;
use writer::DBusWriter;

pub(crate) use queue::DBusQueue;

mod connector;
mod queue;
mod reader;
mod writer;

pub(crate) struct DBusConnection {
    state: State,
    queue: DBusQueue,
    buf: *mut u8,
}

const BUF_SIZE: usize = 500_000;

enum State {
    Connecting(DBusConnector),
    Ready {
        reader: DBusReader,
        writer: DBusWriter,
    },
}

impl DBusConnection {
    pub(crate) fn new(addr: sockaddr_un, queue: DBusQueue) -> Self {
        Self {
            state: State::Connecting(DBusConnector::new(addr)),
            queue,
            buf: Box::leak(Box::new([0; BUF_SIZE])).as_mut_ptr(),
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match &mut self.state {
            State::Connecting(connector) => connector.wants(),
            State::Ready { reader, writer } => {
                let buf: &'static mut [u8] =
                    unsafe { std::slice::from_raw_parts_mut(self.buf, BUF_SIZE) };

                match (reader.wants(buf), writer.wants()) {
                    (
                        Wants::Read {
                            fd,
                            buf: readbuf,
                            len: readlen,
                        },
                        Wants::Write {
                            buf: writebuf,
                            len: writelen,
                            ..
                        },
                    ) => Wants::ReadWrite {
                        fd,
                        readbuf,
                        readlen,
                        writebuf,
                        writelen,
                    },

                    (read, Wants::Nothing) => read,
                    (Wants::Nothing, write) => write,
                    other => unreachable!("DBus reader/write never want {other:?}"),
                }
            }
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&'static [u8]>> {
        let buf: &'static mut [u8] = unsafe { std::slice::from_raw_parts_mut(self.buf, BUF_SIZE) };

        match (&mut self.state, satisfy) {
            (State::Connecting(connector), _) => {
                if let Some(fd) = connector.satisfy(satisfy, res)? {
                    self.state = State::Ready {
                        reader: DBusReader::new(fd),
                        writer: DBusWriter::new(fd, self.queue.clone()),
                    };
                }
                Ok(None)
            }

            (State::Ready { reader, .. }, Satisfy::Read) => {
                if let Some(len) = reader.satisfy(satisfy, res, buf)? {
                    Ok(Some(&buf[..len]))
                } else {
                    Ok(None)
                }
            }

            (State::Ready { writer, .. }, Satisfy::Write) => {
                writer.satisfy(satisfy, res)?;
                Ok(None)
            }

            (_, satisfy) => {
                bail!("malformed DBusReader state: reader/writer can't handle {satisfy:?}")
            }
        }
    }
}
