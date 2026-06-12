use anyhow::Result;
use dbus::{
    DBusConnection, DBusConnector, DBusConnectorWants, DBusWantsRead, DBusWantsWrite,
    IncomingMessage,
};
use rustix::net::{AddressFamily, SocketAddrUnix, SocketType};
use std::os::fd::BorrowedFd;

use crate::{
    sansio::{Satisfy, Wants},
    utils::dbus::queue::DBusQueue,
};

pub(crate) enum DBusState {
    WantsSocket,
    WantsConnect {
        fd: BorrowedFd<'static>,
    },
    Connecting {
        fd: BorrowedFd<'static>,
        connector: DBusConnector,
    },
    Ready {
        fd: BorrowedFd<'static>,
        connection: DBusConnection,
    },
    Disconnected,
}

impl DBusState {
    pub(crate) fn wants(
        &mut self,
        addr: &SocketAddrUnix,
        readbuf: &mut [u8],
        queue: &DBusQueue,
    ) -> Result<Option<Wants>> {
        match self {
            Self::WantsSocket => Ok(Some(Wants::Socket {
                domain: AddressFamily::UNIX,
                r#type: SocketType::STREAM,
                seq: 0,
            })),
            Self::WantsConnect { fd } => Ok(Some(Wants::Connect {
                fd: *fd,
                addr: addr.clone().into(),
                seq: 1,
            })),
            Self::Connecting { fd, connector } => match connector.wants(readbuf)? {
                DBusConnectorWants::Read { buf, seq } => Ok(Some(Wants::Read {
                    fd: *fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                    seq,
                })),
                DBusConnectorWants::Write { buf, seq } => Ok(Some(Wants::Write {
                    fd: *fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                    seq,
                })),
            },
            Self::Ready { fd, connection } => match connection.wants(queue, readbuf)? {
                (
                    DBusWantsRead {
                        buf: readbuf,
                        seq: readseq,
                    },
                    Some(DBusWantsWrite {
                        buf: writebuf,
                        seq: writeseq,
                    }),
                ) => Ok(Some(Wants::ReadWrite {
                    fd: *fd,
                    readbuf: readbuf.as_mut_ptr(),
                    readlen: readbuf.len(),
                    readseq,
                    writebuf: writebuf.as_ptr(),
                    writelen: writebuf.len(),
                    writeseq,
                })),
                (DBusWantsRead { buf, seq }, None) => Ok(Some(Wants::Read {
                    fd: *fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                    seq,
                })),
            },

            Self::Disconnected => Ok(None),
        }
    }

    pub(crate) fn satisfy<'buf>(
        self,
        satisfy: Satisfy,
        readbuf: &'buf [u8],
        queue: &mut DBusQueue,
    ) -> Result<(Self, Option<IncomingMessage<'buf>>)> {
        match (self, satisfy) {
            (Self::WantsSocket, Satisfy::Socket(res)) => {
                let fd = res?;
                Ok((Self::WantsConnect { fd }, None))
            }

            (Self::WantsConnect { fd }, Satisfy::Connect(res)) => {
                res?;
                Ok((
                    Self::Connecting {
                        fd,
                        connector: DBusConnector::new(),
                    },
                    None,
                ))
            }

            (Self::Connecting { fd, mut connector }, Satisfy::Read(res)) => {
                let len = res?;
                connector.satisfy_read(len, readbuf)?;
                Ok((Self::Connecting { fd, connector }, None))
            }
            (Self::Connecting { fd, mut connector }, Satisfy::Write(res)) => {
                let len = res?;
                if let Some(seq) = connector.satisfy_write(len)? {
                    Ok((
                        Self::Ready {
                            fd,
                            connection: DBusConnection::new(seq),
                        },
                        None,
                    ))
                } else {
                    Ok((Self::Connecting { fd, connector }, None))
                }
            }

            (Self::Ready { fd, mut connection }, Satisfy::Read(res)) => {
                let len = res?;
                let message = connection.satisfy_read(len, readbuf)?;
                Ok((Self::Ready { fd, connection }, message))
            }
            (Self::Ready { fd, mut connection }, Satisfy::Write(res)) => {
                let len = res?;
                connection.satisfy_write(len, queue)?;
                Ok((Self::Ready { fd, connection }, None))
            }

            _ => unreachable!(),
        }
    }
}
