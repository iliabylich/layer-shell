use libc::sockaddr;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Wants {
    Socket {
        domain: i32,
        type_: i32,
    },
    Connect {
        fd: i32,
        addr: *const sockaddr,
        addrlen: u32,
    },
    Read {
        fd: i32,
        buf: *mut u8,
        len: usize,
    },
    Write {
        fd: i32,
        buf: *const u8,
        len: usize,
    },
    Accept {
        fd: i32,
    },
}
