use crate::{
    external::{__socket_type_SOCK_STREAM as SOCK_STREAM, AF_UNIX, sockaddr_un, socklen_t},
    sansio::{Satisfy, Wants},
};
use anyhow::{Result, bail};
use core::mem::size_of;
use dbus::{
    DBusConnection, DBusConnector, DBusConnectorWants, DBusWantsRead, DBusWantsWrite,
    IncomingMessage, OutgoingQueue,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum DBusState {
    CanSocket,
    WaitingForSocket,

    CanConnect { fd: i32 },
    WaitingForConnect { fd: i32 },

    Connecting { fd: i32, connector: DBusConnector },
    ConnectingWaiting { fd: i32, connector: DBusConnector },

    Ready { fd: i32, connection: DBusConnection },
    ReadyWaitingRead { fd: i32, connection: DBusConnection },
    ReadyWaitingWrite { fd: i32, connection: DBusConnection },
    ReadyWaitingReadWrite { fd: i32, connection: DBusConnection },

    Disconnected,
}

impl DBusState {
    pub(crate) fn try_wants(
        &mut self,
        addr: &sockaddr_un,
        readbuf: &mut [u8],
        queue: &impl OutgoingQueue,
    ) -> Result<Option<Wants>> {
        match *self {
            Self::CanSocket => {
                *self = Self::WaitingForSocket;
                Ok(Some(Wants::Socket {
                    domain: AF_UNIX,
                    type_: SOCK_STREAM,
                }))
            }

            Self::CanConnect { fd } => {
                *self = Self::WaitingForConnect { fd };
                Ok(Some(Wants::Connect {
                    fd,
                    addr: core::ptr::from_ref(addr).cast(),
                    addrlen: size_of::<sockaddr_un>() as socklen_t,
                }))
            }

            Self::Connecting { fd, connector } => {
                *self = Self::ConnectingWaiting { fd, connector };

                match connector.wants(readbuf)? {
                    DBusConnectorWants::Read { buf, .. } => Ok(Some(Wants::Read {
                        fd,
                        buf: buf.as_mut_ptr(),
                        len: buf.len(),
                    })),
                    DBusConnectorWants::Write { buf, .. } => Ok(Some(Wants::Write {
                        fd,
                        buf: buf.as_ptr(),
                        len: buf.len(),
                    })),
                }
            }

            Self::Ready { fd, connection } => match connection.wants(queue, readbuf)? {
                (
                    DBusWantsRead { buf: readbuf, .. },
                    Some(DBusWantsWrite { buf: writebuf, .. }),
                ) => {
                    *self = Self::ReadyWaitingReadWrite { fd, connection };
                    Ok(Some(Wants::ReadWrite {
                        fd,
                        readbuf: readbuf.as_mut_ptr(),
                        readlen: readbuf.len(),
                        writebuf: writebuf.as_ptr(),
                        writelen: writebuf.len(),
                    }))
                }
                (DBusWantsRead { buf, .. }, None) => {
                    *self = Self::ReadyWaitingRead { fd, connection };
                    Ok(Some(Wants::Read {
                        fd,
                        buf: buf.as_mut_ptr(),
                        len: buf.len(),
                    }))
                }
            },

            Self::ReadyWaitingRead { fd, connection } => {
                let (_, Some(DBusWantsWrite { buf, .. })) = connection.wants(queue, readbuf)?
                else {
                    return Ok(None);
                };
                *self = Self::ReadyWaitingReadWrite { fd, connection };
                Ok(Some(Wants::Write {
                    fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }))
            }

            Self::ReadyWaitingWrite { fd, connection } => {
                let (DBusWantsRead { buf, .. }, _) = connection.wants(queue, readbuf)?;
                *self = Self::ReadyWaitingReadWrite { fd, connection };
                Ok(Some(Wants::Read {
                    fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                }))
            }

            Self::Disconnected
            | Self::WaitingForSocket
            | Self::WaitingForConnect { .. }
            | Self::ConnectingWaiting { .. }
            | Self::ReadyWaitingReadWrite { .. } => Ok(None),
        }
    }

    pub(crate) fn wants(
        &mut self,
        addr: &sockaddr_un,
        readbuf: &mut [u8],
        queue: &impl OutgoingQueue,
    ) -> Option<Wants> {
        match self.try_wants(addr, readbuf, queue) {
            Ok(wants) => wants,
            Err(err) => {
                log::error!("{err:?}");
                *self = Self::Disconnected;
                None
            }
        }
    }

    fn try_satisfy<'buf>(
        self,
        satisfy: Satisfy,
        readbuf: &'buf [u8],
        queue: &mut impl OutgoingQueue,
    ) -> Result<(Self, Option<IncomingMessage<'buf>>)> {
        match (self, satisfy) {
            (Self::WaitingForSocket, Satisfy::Socket(res)) => {
                let fd = res?;
                Ok((Self::CanConnect { fd }, None))
            }

            (Self::WaitingForConnect { fd }, Satisfy::Connect(res)) => {
                res?;
                Ok((
                    Self::Connecting {
                        fd,
                        connector: DBusConnector::new(),
                    },
                    None,
                ))
            }

            (Self::ConnectingWaiting { fd, mut connector }, Satisfy::Read(res)) => {
                let len = res?;
                connector.satisfy_read(len, readbuf)?;
                Ok((Self::Connecting { fd, connector }, None))
            }
            (Self::ConnectingWaiting { fd, mut connector }, Satisfy::Write(res)) => {
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

            (Self::ReadyWaitingRead { fd, mut connection }, Satisfy::Read(res)) => {
                let len = res?;
                let message = connection.satisfy_read(len, readbuf)?;
                Ok((Self::Ready { fd, connection }, message))
            }
            (Self::ReadyWaitingWrite { fd, mut connection }, Satisfy::Write(res)) => {
                let len = res?;
                connection.satisfy_write(len, queue)?;
                Ok((Self::Ready { fd, connection }, None))
            }
            (Self::ReadyWaitingReadWrite { fd, mut connection }, Satisfy::Read(res)) => {
                let len = res?;
                let message = connection.satisfy_read(len, readbuf)?;
                Ok((Self::ReadyWaitingWrite { fd, connection }, message))
            }
            (Self::ReadyWaitingReadWrite { fd, mut connection }, Satisfy::Write(res)) => {
                let len = res?;
                connection.satisfy_write(len, queue)?;
                Ok((Self::ReadyWaitingRead { fd, connection }, None))
            }

            (state, satisfy) => {
                bail!("malformed DBusState: {state:?} vs {satisfy:?}")
            }
        }
    }

    pub(crate) fn satisfy<'buf>(
        self,
        satisfy: Satisfy,
        readbuf: &'buf [u8],
        queue: &mut impl OutgoingQueue,
    ) -> (Self, Option<IncomingMessage<'buf>>) {
        match self.try_satisfy(satisfy, readbuf, queue) {
            Ok(out) => out,
            Err(err) => {
                log::error!("{err:?}");
                (Self::Disconnected, None)
            }
        }
    }
}
