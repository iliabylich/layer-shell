use alloc::vec::Vec;

#[repr(C)]
pub struct FFIArray<T> {
    pub ptr: *const T,
    pub len: usize,
}

impl<T> FFIArray<T> {
    pub(crate) const fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<T, U> From<Vec<T>> for FFIArray<U>
where
    U: From<T>,
{
    fn from(value: Vec<T>) -> Self {
        let value = value.into_iter().map(|e| U::from(e)).collect::<Vec<_>>();
        let mut boxed_slice = value.into_boxed_slice();
        let len = boxed_slice.len();
        let ptr = boxed_slice.as_mut_ptr();
        core::mem::forget(boxed_slice);
        Self { ptr, len }
    }
}

impl<T> From<FFIArray<T>> for Vec<T> {
    fn from(array: FFIArray<T>) -> Self {
        let vec = unsafe { Self::from_raw_parts(array.ptr.cast_mut(), array.len, array.len) };
        core::mem::forget(array);
        vec
    }
}

impl<T> Drop for FFIArray<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Vec::from_raw_parts(self.ptr.cast_mut(), self.len, self.len));
        }
    }
}

impl<T> core::fmt::Debug for FFIArray<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let vec = unsafe { Vec::from_raw_parts(self.ptr.cast_mut(), self.len, self.len) };
        f.debug_list().entries(&vec).finish()?;
        core::mem::forget(vec);
        Ok(())
    }
}

impl<T> Clone for FFIArray<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let vec = unsafe { Vec::from_raw_parts(self.ptr.cast_mut(), self.len, self.len) };
        let clone = vec.clone();
        core::mem::forget(vec);
        Self::from(clone)
    }
}

unsafe impl<T> Send for FFIArray<T> {}
unsafe impl<T> Sync for FFIArray<T> {}

impl<T> Default for FFIArray<T> {
    fn default() -> Self {
        Self::from(Vec::default())
    }
}

impl<T> PartialEq for FFIArray<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T> Eq for FFIArray<T> where T: Eq {}
