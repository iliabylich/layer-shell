use crate::fatal::fatal;

#[repr(transparent)]
pub struct CString {
    pub ptr: *mut std::ffi::c_char,
}

impl From<String> for CString {
    fn from(s: String) -> Self {
        let cstring = std::ffi::CString::new(s).unwrap_or_else(|err| fatal!("{:?}", err));
        Self {
            ptr: cstring.into_raw(),
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

impl<T> From<CArray<T>> for Vec<T> {
    fn from(array: CArray<T>) -> Self {
        let vec = unsafe { Vec::from_raw_parts(array.ptr, array.len, array.len) };
        std::mem::forget(array);
        vec
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
        f.debug_list().entries(&vec).finish()?;
        std::mem::forget(vec);
        Ok(())
    }
}

impl<T> Clone for CArray<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let vec = unsafe { Vec::from_raw_parts(self.ptr, self.len, self.len) };
        let clone = vec.clone();
        std::mem::forget(vec);
        Self::from(clone)
    }
}

unsafe impl<T> Send for CArray<T> {}

#[repr(C)]
pub enum COption<T> {
    None,
    Some(T),
}

impl<T> std::fmt::Debug for COption<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Some(value) => f.debug_tuple("Some").field(value).finish(),
        }
    }
}
