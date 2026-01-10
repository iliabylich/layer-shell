#[repr(C)]
pub struct CArray<T> {
    pub ptr: *mut T,
    pub len: usize,
}

impl<T> CArray<T> {
    pub(crate) fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<T, U> From<Vec<T>> for CArray<U>
where
    U: From<T>,
{
    fn from(value: Vec<T>) -> Self {
        let value = value.into_iter().map(|e| U::from(e)).collect::<Vec<_>>();
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
unsafe impl<T> Sync for CArray<T> {}

impl<T> Default for CArray<T> {
    fn default() -> Self {
        Self::from(Vec::default())
    }
}

impl<T> PartialEq for CArray<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T> Eq for CArray<T> where T: Eq {}
