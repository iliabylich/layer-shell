use libc::sockaddr_un;

pub struct SockaddrUn;

impl SockaddrUn {
    pub(crate) const fn from_bytes(path: &[u8]) -> sockaddr_un {
        let mut addr = sockaddr_un {
            sun_family: 0,
            sun_path: [0; _],
        };
        addr.sun_family = rustix::net::AddressFamily::UNIX.as_raw();

        assert!(path.len() < addr.sun_path.len(), "path is too long");

        let mut idx = 0;
        #[expect(clippy::indexing_slicing)]
        while idx < path.len() {
            addr.sun_path[idx] = path[idx].cast_signed();
            idx += 1;
        }

        addr
    }
}
