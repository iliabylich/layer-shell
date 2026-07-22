use crate::utils::log_err_and_exit;
use string_bath::StringPool;

const SLOTS_COUNT: usize = 100;
const STRING_LEN: usize = 256;

static mut STRING_POOL: StringPool<SLOTS_COUNT, STRING_LEN> = StringPool::new();

/// cbindgen:ignore
pub type StringRef = string_bath::StringRef<'static, STRING_LEN>;

pub trait StringRefExt {
    fn new(s: &str) -> Self;
    fn as_const_ptr(&self) -> *const i8;
    fn empty() -> Self;
}

impl StringRefExt for StringRef {
    fn new(s: &str) -> Self {
        unsafe {
            STRING_POOL
                .alloc(s)
                .unwrap_or_else(|err| log_err_and_exit!("{err}"))
        }
    }

    fn as_const_ptr(&self) -> *const i8 {
        self.as_str().as_ptr().cast::<i8>()
    }

    fn empty() -> Self {
        Self::new("")
    }
}
