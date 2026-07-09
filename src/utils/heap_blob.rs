use core::ptr::NonNull;

pub(crate) struct HeapBlob {
    ptr: NonNull<u8>,
    len: usize,
}

impl HeapBlob {
    pub(crate) fn new(len: usize) -> Result<Self, FailedToMallocError> {
        Ok(Self {
            ptr: unsafe {
                NonNull::new(libc::calloc(len, 1))
                    .ok_or(FailedToMallocError)?
                    .cast::<u8>()
            },
            len,
        })
    }

    pub(crate) const fn as_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for HeapBlob {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr.as_ptr().cast());
        }
    }
}

#[derive(Debug)]
pub(crate) struct FailedToMallocError;
impl core::fmt::Display for FailedToMallocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FailedToMalloc")
    }
}
impl core::error::Error for FailedToMallocError {}
