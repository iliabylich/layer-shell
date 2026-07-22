use libc::{AF_UNIX, sa_family_t, sockaddr_un};

pub(crate) const fn new_sockaddr_un(path: &[u8]) -> sockaddr_un {
    let mut addr = sockaddr_un {
        sun_family: 0,
        sun_path: [0; _],
    };
    addr.sun_family = AF_UNIX as sa_family_t;

    assert!(path.len() < addr.sun_path.len(), "path is too long");

    let mut idx = 0;
    #[expect(clippy::indexing_slicing)]
    while idx < path.len() {
        addr.sun_path[idx] = path[idx].cast_signed();
        idx += 1;
    }

    addr
}
