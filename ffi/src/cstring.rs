use std::sync::Arc;

#[repr(transparent)]
pub struct CString {
    pub ptr: *mut std::ffi::c_char,
}

impl CString {
    pub fn as_str(&self) -> &str {
        unsafe { std::ffi::CStr::from_ptr(self.ptr.cast()) }
            .to_str()
            .unwrap_or_else(|err| {
                log::error!("{:?}", err);
                std::process::exit(1)
            })
    }

    pub fn into_raw(self) -> *mut std::ffi::c_char {
        let ptr = self.ptr;
        std::mem::forget(self);
        ptr
    }

    pub fn none() -> Self {
        Self::from("none".to_string())
    }
}

impl From<String> for CString {
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

impl From<CString> for String {
    fn from(s: CString) -> Self {
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

impl From<Arc<str>> for CString {
    fn from(value: Arc<str>) -> Self {
        Self::from(value.as_ref().to_string())
    }
}

impl From<*const std::ffi::c_char> for CString {
    fn from(ptr: *const std::ffi::c_char) -> Self {
        fn ptr_to_self(ptr: *const std::ffi::c_char) -> CString {
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

unsafe impl Send for CString {}
unsafe impl Sync for CString {}

impl Drop for CString {
    fn drop(&mut self) {
        unsafe {
            drop(std::ffi::CString::from_raw(self.ptr));
        }
    }
}

impl std::fmt::Debug for CString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = unsafe { std::ffi::CString::from_raw(self.ptr) };
        write!(f, "{:?}", s)?;
        std::mem::forget(s);
        Ok(())
    }
}

impl Clone for CString {
    fn clone(&self) -> Self {
        let s = unsafe { std::ffi::CString::from_raw(self.ptr) };
        let out = Self {
            ptr: s.clone().into_raw(),
        };
        std::mem::forget(s);
        out
    }
}

impl Default for CString {
    fn default() -> Self {
        Self::from(String::default())
    }
}

impl PartialEq for CString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for CString {}
