#[repr(C)]
pub struct CString {
    pub ptr: *mut std::ffi::c_char,
}

impl From<String> for CString {
    fn from(s: String) -> Self {
        let cstring = std::ffi::CString::new(s).unwrap();
        Self {
            ptr: cstring.into_raw().cast(),
        }
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
        write!(f, "{:?}", s).unwrap();
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

#[repr(C)]
pub struct CArray<T> {
    pub ptr: *mut T,
    pub len: usize,
}

impl<T> From<Vec<T>> for CArray<T> {
    fn from(value: Vec<T>) -> Self {
        let mut boxed_slice = value.into_boxed_slice();
        let len = boxed_slice.len();
        let ptr = boxed_slice.as_mut_ptr();
        std::mem::forget(boxed_slice);
        Self { ptr, len }
    }
}

impl<T> Drop for CArray<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Vec::from_raw_parts(self.ptr, self.len, self.len));
        }
    }
}

impl<T> std::fmt::Debug for CArray<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vec = unsafe { Vec::from_raw_parts(self.ptr, self.len, self.len) };
        f.debug_list().entries(&vec).finish().unwrap();
        std::mem::forget(vec);
        Ok(())
    }
}

unsafe impl<T> Send for CArray<T> {}

#[repr(C)]
pub struct CBytes {
    pub content: *const u8,
    pub len: usize,
}

impl CBytes {
    pub const fn new(slice: &'static [u8]) -> Self {
        Self {
            content: slice.as_ptr(),
            len: slice.len(),
        }
    }
}

unsafe impl Send for CBytes {}
