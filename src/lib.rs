#![expect(static_mut_refs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(deprecated_in_future)]
#![warn(unused_lifetimes)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![expect(clippy::redundant_pub_crate)]
#![expect(clippy::cast_possible_truncation)]
#![expect(clippy::arithmetic_side_effects)]
#![expect(clippy::too_many_lines)]

mod command;
mod config;
mod event;
mod event_queue;
mod ffi;
mod io;
mod liburing;
mod modules;
mod sansio;
mod unix_socket;
mod user_data;
mod utils;

use command::Command;
use config::IOConfig;
pub use event::Event;
pub use ffi::FFIArray;

use crate::{io::IO, liburing::IoUring, utils::StringRef};

macro_rules! map_panic_to {
    ($fallback:expr; $code:expr) => {{
        match std::panic::catch_unwind(|| $code) {
            Ok(Ok(out)) => out,
            Ok(Err(err)) => {
                let err: anyhow::Error = err;
                log::error!("error returned: {err:?}");
                $fallback
            }
            Err(err) => {
                log::error!("panic: {err:?}");
                $fallback
            }
        }
    }};
}
macro_rules! map_panic_to_null {
    ($code:expr) => {
        map_panic_to!(std::ptr::null_mut(); $code)
    };
}
macro_rules! map_panic_to_false {
    ($code:expr) => {
        map_panic_to!(false; $code)
    };
}

#[unsafe(no_mangle)]
#[must_use]
pub extern "C" fn io_init(
    on_event: extern "C" fn(event: *const Event),
    logging_enabled: bool,
) -> bool {
    map_panic_to_false!({
        IO::init(on_event, logging_enabled)?;
        Ok(true)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_deinit() -> bool {
    map_panic_to_false!({
        IO::deinit()?;
        Ok(true)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_handle_readable() -> bool {
    map_panic_to_false!({
        IO::global()?.handle_readable()?;
        Ok(true)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_wait_readable() -> bool {
    map_panic_to_false!({
        IO::wait_readable()?;
        Ok(true)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_as_raw_fd() -> i32 {
    IoUring::as_raw_fd()
}

#[unsafe(no_mangle)]
pub extern "C" fn io_get_config() -> *const IOConfig {
    map_panic_to_null!(Ok(IO::global()?.io_config))
}

#[unsafe(no_mangle)]
pub extern "C" fn io_lock() -> bool {
    map_panic_to_false!({
        IO::global()?.process_command(Command::Lock)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot() -> bool {
    map_panic_to_false!({
        IO::global()?.process_command(Command::Reboot)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown() -> bool {
    map_panic_to_false!({
        IO::global()?.process_command(Command::Shutdown)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout() -> bool {
    map_panic_to_false!({
        IO::global()?.process_command(Command::Logout)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
#[expect(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn io_trigger_tray(uuid: *const core::ffi::c_char) -> bool {
    map_panic_to_false!({
        let uuid = unsafe { std::ffi::CStr::from_ptr(uuid) }.to_str()?;

        IO::global()?.process_command(Command::TriggerTray {
            uuid: StringRef::new(uuid)?,
        })?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_wifi_editor() -> bool {
    map_panic_to_false!({
        IO::global()?.process_command(Command::SpawnWiFiEditor)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_bluetooh_editor() -> bool {
    map_panic_to_false!({
        IO::global()?.process_command(Command::SpawnBluetoothEditor)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor() -> bool {
    map_panic_to_false!({
        IO::global()?.process_command(Command::SpawnSystemMonitor)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_theme() -> bool {
    map_panic_to_false!({
        IO::global()?.process_command(Command::ChangeTheme)?;
        Ok(true)
    })
}
