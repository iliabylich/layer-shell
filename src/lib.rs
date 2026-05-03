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
#![expect(clippy::unsafe_derive_deserialize)]

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

use std::os::fd::AsRawFd as _;

use command::Command;
use config::IOConfig;
pub use event::Event;
pub use ffi::FFIArray;

use crate::{io::IO, utils::StringRef};

macro_rules! map_panic_to_exit_with_error {
    ($code:expr) => {{
        match std::panic::catch_unwind(|| $code) {
            Ok(Ok(out)) => out,
            Ok(Err(err)) => {
                let err: anyhow::Error = err;
                log::error!("error returned: {err:?}");
                std::process::exit(1)
            }
            Err(err) => {
                log::error!("panic: {err:?}");
                std::process::exit(1)
            }
        }
    }};
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init(on_event: extern "C" fn(event: *const Event), logging_enabled: bool) {
    map_panic_to_exit_with_error!(IO::init(on_event, logging_enabled));
}

#[unsafe(no_mangle)]
pub extern "C" fn io_deinit() {
    IO::deinit();
}

#[unsafe(no_mangle)]
pub extern "C" fn io_handle_readable() {
    map_panic_to_exit_with_error!(IO::global()?.handle_readable());
}

#[unsafe(no_mangle)]
pub extern "C" fn io_wait_readable() {
    map_panic_to_exit_with_error!(IO::global()?.wait_readable());
}

#[unsafe(no_mangle)]
pub extern "C" fn io_as_raw_fd() -> i32 {
    map_panic_to_exit_with_error!(Ok(IO::global()?.as_raw_fd()))
}

#[unsafe(no_mangle)]
#[must_use]
pub extern "C" fn io_get_config() -> *const IOConfig {
    map_panic_to_exit_with_error!(Ok(IO::global()?.io_config))
}

#[unsafe(no_mangle)]
pub extern "C" fn io_lock() {
    map_panic_to_exit_with_error!(IO::global()?.process_command(Command::Lock));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot() {
    map_panic_to_exit_with_error!(IO::global()?.process_command(Command::Reboot));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown() {
    map_panic_to_exit_with_error!(IO::global()?.process_command(Command::Shutdown));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout() {
    map_panic_to_exit_with_error!(IO::global()?.process_command(Command::Logout));
}
#[unsafe(no_mangle)]
#[expect(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn io_trigger_tray(uuid: *const core::ffi::c_char) {
    map_panic_to_exit_with_error!({
        let uuid = unsafe { std::ffi::CStr::from_ptr(uuid) }.to_str()?;

        IO::global()?.process_command(Command::TriggerTray {
            uuid: StringRef::new(uuid)?,
        })
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_wifi_editor() {
    map_panic_to_exit_with_error!(IO::global()?.process_command(Command::SpawnWiFiEditor));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_bluetooh_editor() {
    map_panic_to_exit_with_error!(IO::global()?.process_command(Command::SpawnBluetoothEditor));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor() {
    map_panic_to_exit_with_error!(IO::global()?.process_command(Command::SpawnSystemMonitor));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_theme() {
    map_panic_to_exit_with_error!(IO::global()?.process_command(Command::ChangeTheme));
}
