use core::{ffi::CStr, ptr::NonNull};

pub struct EnvHelper;

impl EnvHelper {
    pub(crate) fn home() -> &'static str {
        let Some(home) = getenv(c"HOME") else {
            panic!("no $HOME")
        };
        let Ok(home) = core::str::from_utf8(home) else {
            panic!("non-utf8 $HOME")
        };
        home
    }

    pub(crate) fn xdg_runtime_dir() -> &'static str {
        let Some(xdg_runtime_dir) = getenv(c"XDG_RUNTIME_DIR") else {
            panic!("no $XDG_RUNTIME_DIR")
        };
        let Ok(xdg_runtime_dir) = core::str::from_utf8(xdg_runtime_dir) else {
            panic!("non-utf8 $XDG_RUNTIME_DIR")
        };
        xdg_runtime_dir
    }

    pub(crate) fn xdg_config_dir() -> Option<&'static str> {
        let xdg_config_dir = getenv(c"XDG_CONFIG_DIR")?;
        let Ok(xdg_config_dir) = core::str::from_utf8(xdg_config_dir) else {
            panic!("non-utf8 $XDG_CONFIG_DIR")
        };
        Some(xdg_config_dir)
    }

    pub(crate) fn niri_socket() -> Option<&'static str> {
        let niri_socket = getenv(c"NIRI_SOCKET")?;
        let Ok(niri_socket) = core::str::from_utf8(niri_socket) else {
            panic!("non-utf8 $NIRI_SOCKET");
        };
        Some(niri_socket)
    }

    pub(crate) fn rust_log() -> Option<&'static [u8]> {
        getenv(c"RUST_LOG")
    }
}

fn getenv(var: &'static CStr) -> Option<&'static [u8]> {
    let ptr = NonNull::new(unsafe { libc::getenv(var.as_ptr()) })?;
    Some(unsafe { CStr::from_ptr(ptr.as_ptr()) }.to_bytes())
}
