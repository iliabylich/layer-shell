mod array_writer;
mod epoll;
mod fixed_size_array;
mod fixed_size_buffer;
mod getenv;
mod nl_separated_buffer;
mod spawn;
mod string_pool;

pub use array_writer::ArrayWriter;
pub use epoll::{Epoll, ModulePoll};
pub use fixed_size_array::FixedSizeArrray;
pub use fixed_size_buffer::FixedSizeBuffer;
pub use getenv::EnvHelper;
pub use nl_separated_buffer::NlSeparatedBuffer;
pub use spawn::SpawnHelper;
pub use string_pool::{StringRef, StringRefExt};

pub(crate) use array_writer::write_in_place;

macro_rules! unix_socket {
    () => {
        rustix::net::socket(
            rustix::net::AddressFamily::UNIX,
            rustix::net::SocketType::STREAM,
            None,
        )
        .map_err(|err| {
            log::error!("failed to socket(): {err:?}");
        })
    };
}
pub(crate) use unix_socket;

macro_rules! unix_socket_addr {
    ($($arg:tt)*) => {{
        let mut buf = [0; 200];
        let path = $crate::utils::write_in_place!(&mut buf, $($arg)+);
        rustix::net::SocketAddrUnix::new(path)
            .map_err(|err| log::error!("failed to create UNIX socket: {err:?}"))
    }};
}
pub(crate) use unix_socket_addr;
