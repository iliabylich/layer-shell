use crate::{
    modules::FallibleModule,
    sansio::{Satisfy, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
    utils::dbus::queue::DBusQueue,
};
use anyhow::{Context, Result, bail};
use dbus::{DBusConnection, DBusWants, IncomingMessage};
use libc::{sockaddr, sockaddr_un};

pub(crate) struct SessionDBus {
    fd: Option<i32>,
    conn: DBusConnection,
    sock_addr: Option<sockaddr_un>,
}

static mut READBUF: Vec<u8> = vec![];
fn readbuf() -> &'static mut Vec<u8> {
    unsafe { &mut READBUF }
}

static mut QUEUE: DBusQueue = DBusQueue::new();
const fn queue() -> &'static mut DBusQueue {
    unsafe { &mut QUEUE }
}

impl SessionDBus {
    pub(crate) fn init() -> Result<()> {
        queue().push_hello()?;
        readbuf().resize(400 * 1_024, 0);
        Ok(())
    }

    fn try_new() -> Result<Self> {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
        let (_, address) = address
            .split_once('=')
            .context("malformed $DBUS_SESSION_BUS_ADDRESS")?;

        Ok(Self {
            fd: None,
            conn: DBusConnection::new_session(address)?,
            sock_addr: None,
        })
    }

    pub(crate) fn new() -> Self {
        Self::try_new().unwrap_or_else(|err| {
            log::error!(target: Self::MODULE_ID.as_str(), "{err:?}");
            Self {
                fd: None,
                conn: DBusConnection::dummy(),
                sock_addr: None,
            }
        })
    }

    pub(crate) const fn queue() -> &'static mut DBusQueue {
        queue()
    }
}

impl FallibleModule for SessionDBus {
    const MODULE_ID: ModuleId = ModuleId::SessionDBus;
    type Output = IncomingMessage<'static>;

    fn wants(&mut self) -> Result<Option<Wants>> {
        let Some(wants) = self.conn.wants(queue(), readbuf())? else {
            return Ok(None);
        };

        let wants = match wants {
            DBusWants::Socket {
                domain,
                r#type,
                seq,
            } => Wants::Socket {
                domain: domain.as_raw().into(),
                r#type: i32::try_from(r#type.as_raw()).context("malformed socket type")?,
                seq,
            },
            DBusWants::Connect { addr, seq } => {
                self.sock_addr = Some(new_unix_socket(
                    addr.path_bytes().context("empty sockaddr")?,
                )?);
                let addr = self
                    .sock_addr
                    .as_ref()
                    .map(|addr| (&raw const *addr).cast::<sockaddr>())
                    .unwrap_or_else(|| unreachable!("it is set 1 line above"));
                let fd = self
                    .fd
                    .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));

                Wants::Connect {
                    fd,
                    addr,
                    addrlen: size_of::<sockaddr_un>() as u32,
                    seq,
                }
            }
            DBusWants::Read { buf, seq } => {
                let fd = self
                    .fd
                    .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));
                Wants::Read {
                    fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                    seq,
                }
            }
            DBusWants::Write { buf, seq } => {
                let fd = self
                    .fd
                    .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));
                Wants::Write {
                    fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                    seq,
                }
            }
            DBusWants::ReadWrite {
                readbuf,
                readseq,
                writebuf,
                writeseq,
            } => {
                let fd = self
                    .fd
                    .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));
                Wants::ReadWrite {
                    fd,
                    readbuf: readbuf.as_mut_ptr(),
                    readlen: readbuf.len(),
                    readseq,
                    writebuf: writebuf.as_ptr(),
                    writelen: writebuf.len(),
                    writeseq,
                }
            }
        };

        Ok(Some(wants))
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        match satisfy {
            Satisfy::Socket => {
                self.fd = Some(res);
                self.conn.satisfy_socket()?;
                Ok(None)
            }
            Satisfy::Connect => {
                self.conn.satisfy_connect()?;
                Ok(None)
            }
            Satisfy::Write => {
                if let Ok(len) = usize::try_from(res) {
                    self.conn.satisfy_write(len, queue())?;
                    Ok(None)
                } else {
                    self.conn.stop();
                    bail!("SessionDBus got error on Write: {res}");
                }
            }
            Satisfy::Read => {
                if let Ok(len) = usize::try_from(res) {
                    let message = self.conn.satisfy_read(len, readbuf())?;
                    Ok(message)
                } else {
                    self.conn.stop();
                    bail!("SessionDBus got error on Read: {res}");
                }
            }
            Satisfy::Close | Satisfy::OpenAt => unreachable!(),
        }
    }
}
