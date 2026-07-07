use core::{ffi::CStr, ptr::NonNull};

pub(crate) fn getenv(var: &'static CStr) -> Option<&'static [u8]> {
    let ptr = NonNull::new(unsafe { libc::getenv(var.as_ptr()) })?;
    Some(unsafe { CStr::from_ptr(ptr.as_ptr()) }.to_bytes())
}
