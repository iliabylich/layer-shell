mod array_writer;
mod fixed_size_array;
mod fixed_size_buffer;
mod getenv;
mod nl_separated_buffer;
mod sockaddr;
mod spawn;
mod string_pool;

pub use array_writer::ArrayWriter;
pub use fixed_size_array::FixedSizeArrray;
pub use fixed_size_buffer::FixedSizeBuffer;
pub use getenv::EnvHelper;
pub use nl_separated_buffer::NlSeparatedBuffer;
pub use sockaddr::SockaddrUn;
pub use spawn::SpawnHelper;
pub use string_pool::{StringRef, StringRefExt};

pub(crate) use array_writer::write_in_place;

macro_rules! log_err_and_exit {
    ($($arg:tt)*) => {{
        log::error!("Fatal error at {}:{}", file!(), line!());
        log::error!($($arg)+);
        #[allow(unused_unsafe)]
        unsafe { libc::exit(1) }
    }};
}
pub(crate) use log_err_and_exit;
