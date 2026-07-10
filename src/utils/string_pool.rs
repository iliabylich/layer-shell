use crate::external::exit;
use string_bath::StringPool;

const SLOTS_COUNT: usize = 100;
const STRING_LEN: usize = 256;

static mut STRING_POOL: StringPool<SLOTS_COUNT, STRING_LEN> = StringPool::new();

/// cbindgen:ignore
pub(crate) type StringRef = string_bath::StringRef<'static, STRING_LEN>;

pub(crate) trait StringRefExt {
    fn new(s: &str) -> Self;
    fn as_const_ptr(&self) -> *const i8;
}

impl StringRefExt for StringRef {
    fn new(s: &str) -> Self {
        unsafe {
            STRING_POOL.alloc(s).unwrap_or_else(|err| {
                log::error!("{err}");
                exit(1);
            })
        }
    }

    fn as_const_ptr(&self) -> *const i8 {
        self.as_str().as_ptr().cast::<i8>()
    }
}
