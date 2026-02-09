use libc::{AF_UNIX, sockaddr_un};

pub(crate) fn zero_unix_socket() -> sockaddr_un {
    sockaddr_un {
        sun_family: 0,
        sun_path: [0; _],
    }
}

pub(crate) fn new_unix_socket(path: &[u8]) -> sockaddr_un {
    sockaddr_un {
        sun_family: AF_UNIX as u16,
        sun_path: {
            let path = unsafe { std::mem::transmute::<&[u8], &[i8]>(path) };
            let mut out = [0; 108];
            out[..path.len()].copy_from_slice(path);
            out
        },
    }
}
