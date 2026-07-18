use libc::{AF_UNIX, sa_family_t, sockaddr_un};

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
