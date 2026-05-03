use anyhow::{Context as _, Result};
use libc::{AF_UNIX, sockaddr_un};

#[expect(clippy::cast_possible_truncation)]
pub(crate) fn new_unix_socket(path: &[u8]) -> Result<sockaddr_un> {
    Ok(sockaddr_un {
        sun_family: { AF_UNIX as u16 },
        sun_path: {
            let path = unsafe {
                &*({
                    let ptr: *const _ = path;
                    ptr as *const [i8]
                })
            };
            let mut out = [0; 108];
            out.get_mut(..path.len())
                .context("UNIX socket path is too long (max 108)")?
                .copy_from_slice(path);
            out
        },
    })
}
