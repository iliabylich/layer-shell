use std::sync::Arc;

#[repr(transparent)]
pub struct FFIString {
    pub ptr: *mut std::ffi::c_char,
}

impl FFIString {
    pub(crate) fn as_str(&self) -> &str {
        unsafe { std::ffi::CStr::from_ptr(self.ptr.cast()) }
            .to_str()
            .unwrap_or_else(|err| {
                log::error!("{:?}", err);
                std::process::exit(1)
            })
    }

    pub(crate) fn into_raw(self) -> *mut std::ffi::c_char {
        let ptr = self.ptr;
        std::mem::forget(self);
        ptr
    }

    #[expect(dead_code)]
    pub(crate) fn none() -> Self {
        Self::from("none".to_string())
    }
}

impl From<String> for FFIString {
    fn from(s: String) -> Self {
        let cstring = std::ffi::CString::new(s).unwrap_or_else(|err| {
            log::error!("{:?}", err);
            std::process::exit(1)
        });
        Self {
            ptr: cstring.into_raw(),
        }
    }
}

impl From<FFIString> for String {
    fn from(s: FFIString) -> Self {
        let out = unsafe { std::ffi::CString::from_raw(s.ptr) }
            .to_str()
            .unwrap_or_else(|err| {
                log::error!("{:?}", err);
                std::process::exit(1)
            })
            .to_string();
        std::mem::forget(s);
        out
    }
}

impl From<Arc<str>> for FFIString {
    fn from(value: Arc<str>) -> Self {
        Self::from(value.as_ref().to_string())
    }
}

impl From<*const std::ffi::c_char> for FFIString {
    fn from(ptr: *const std::ffi::c_char) -> Self {
        fn ptr_to_self(ptr: *const std::ffi::c_char) -> FFIString {
            unsafe { std::ffi::CStr::from_ptr(ptr) }
                .to_str()
                .unwrap_or_else(|err| {
                    log::error!("{:?}", err);
                    std::process::exit(1)
                })
                .to_string()
                .into()
        }
        ptr_to_self(ptr)
    }
}

unsafe impl Send for FFIString {}
unsafe impl Sync for FFIString {}

impl Drop for FFIString {
    fn drop(&mut self) {
        unsafe {
            drop(std::ffi::CString::from_raw(self.ptr));
        }
    }
}

impl std::fmt::Debug for FFIString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = unsafe { std::ffi::CString::from_raw(self.ptr) };
        write!(f, "{:?}", s)?;
        std::mem::forget(s);
        Ok(())
    }
}

impl Clone for FFIString {
    fn clone(&self) -> Self {
        let s = unsafe { std::ffi::CString::from_raw(self.ptr) };
        let out = Self {
            ptr: s.clone().into_raw(),
        };
        std::mem::forget(s);
        out
    }
}

impl Default for FFIString {
    fn default() -> Self {
        Self::from(String::default())
    }
}

impl PartialEq for FFIString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for FFIString {}
