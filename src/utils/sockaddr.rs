use crate::external::{AF_INET, AF_UNIX, in_addr, sa_family_t, sockaddr_in, sockaddr_un};

pub(crate) const fn new_sockaddr_in(ip: [u8; 4], port: u16) -> sockaddr_in {
    sockaddr_in {
        sin_family: AF_INET as sa_family_t,
        sin_port: port.to_be(),
        sin_addr: in_addr {
            s_addr: u32::from_be_bytes(ip).to_be(),
        },
        sin_zero: [0; 8],
    }
}

pub(crate) const fn new_sockaddr_un(
    path: &[u8],
) -> Result<sockaddr_un, PathIsTooLongForSockaddrUn> {
    let mut addr: sockaddr_un = unsafe { core::mem::zeroed() };
    addr.sun_family = AF_UNIX as sa_family_t;

    if path.len() >= addr.sun_path.len() {
        return Err(PathIsTooLongForSockaddrUn);
    }

    let mut idx = 0;
    #[expect(clippy::indexing_slicing)]
    while idx < path.len() {
        addr.sun_path[idx] = path[idx].cast_signed();
        idx += 1;
    }

    Ok(addr)
}

#[derive(Debug)]
pub(crate) struct PathIsTooLongForSockaddrUn;
impl core::fmt::Display for PathIsTooLongForSockaddrUn {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PathIsTooLongForSockaddrUn")
    }
}
impl core::error::Error for PathIsTooLongForSockaddrUn {}
