use mini_sansio_dbus::DBusWants;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Wants {
    Socket {
        domain: i32,
        r#type: i32,
        seq: u64,
    },
    Connect {
        fd: i32,
        addr: *const libc::sockaddr,
        addrlen: u32,
        seq: u64,
    },
    Read {
        fd: i32,
        buf: *mut u8,
        len: usize,
        seq: u64,
    },
    Write {
        fd: i32,
        buf: *const u8,
        len: usize,
        seq: u64,
    },
    ReadWrite {
        fd: i32,
        readbuf: *mut u8,
        readlen: usize,
        readseq: u64,
        writebuf: *const u8,
        writelen: usize,
        writeseq: u64,
    },
    OpenAt {
        dfd: i32,
        path: *const core::ffi::c_char,
        flags: i32,
        mode: u32,
        seq: u64,
    },
    Close {
        fd: i32,
        seq: u64,
    },
}

impl From<DBusWants> for Wants {
    fn from(wants: DBusWants) -> Self {
        match wants {
            DBusWants::Socket {
                domain,
                r#type,
                seq,
            } => Self::Socket {
                domain,
                r#type,
                seq,
            },
            DBusWants::Connect {
                fd,
                addr,
                addrlen,
                seq,
            } => Self::Connect {
                fd,
                addr,
                addrlen,
                seq,
            },
            DBusWants::Read { fd, buf, len, seq } => Self::Read { fd, buf, len, seq },
            DBusWants::Write { fd, buf, len, seq } => Self::Write { fd, buf, len, seq },
            DBusWants::ReadWrite {
                fd,
                readbuf,
                readlen,
                readseq,
                writebuf,
                writelen,
                writeseq,
            } => Self::ReadWrite {
                fd,
                readbuf,
                readlen,
                readseq,
                writebuf,
                writelen,
                writeseq,
            },
        }
    }
}
