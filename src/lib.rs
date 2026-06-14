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
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::option_if_let_else)]
#![expect(clippy::redundant_pub_crate)]
#![expect(clippy::cast_possible_truncation)]
#![expect(clippy::arithmetic_side_effects)]

mod command;
mod config;
mod event;
mod event_queue;
mod ffi;
mod io;
mod liburing;
mod modules;
mod sansio;
mod user_data;
mod utils;

use std::os::fd::AsRawFd as _;

use command::Command;
use config::IOConfig;
pub use event::Event;
pub use ffi::FFIArray;

use crate::{
    io::IO,
    utils::{StringRef, StringRefExt as _},
};
use anyhow::{Context as _, Result};

static mut GLOBAL_IO: *mut IO = core::ptr::null_mut();

fn global_io() -> Result<&'static mut IO> {
    unsafe { GLOBAL_IO.as_mut() }.context("IO is not initialized. Call io_init() first.")
}

fn map_panic_to_exit_with_error<T>(f: impl core::panic::UnwindSafe + FnOnce() -> Result<T>) -> T {
    match std::panic::catch_unwind(f) {
        Ok(Ok(out)) => out,
        Ok(Err(err)) => {
            log::error!("error returned: {err:?}");
            std::process::exit(1);
        }
        Err(err) => {
            log::error!("panic: {err:?}");
            std::process::exit(1);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init(
    callback: extern "C" fn(event: *const Event, *mut std::ffi::c_void),
    data: *mut std::ffi::c_void,
) {
    if unsafe { !GLOBAL_IO.is_null() } {
        eprintln!("io_init() has been already called");
        std::process::exit(1);
    }

    map_panic_to_exit_with_error(|| {
        IO::init()?;
        unsafe {
            GLOBAL_IO = Box::into_raw(Box::new(IO::new((callback, data))?));
        }
        Ok(())
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn io_deinit() {
    map_panic_to_exit_with_error(|| {
        global_io()?.stop();
        Ok(())
    });

    unsafe {
        drop(Box::from_raw(GLOBAL_IO));
        GLOBAL_IO = core::ptr::null_mut();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_handle_readable() {
    map_panic_to_exit_with_error(|| global_io()?.handle_readable());
}

#[unsafe(no_mangle)]
pub extern "C" fn io_wait_readable() {
    map_panic_to_exit_with_error(|| {
        global_io()?.wait_readable();
        Ok(())
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn io_as_raw_fd() -> i32 {
    map_panic_to_exit_with_error(|| Ok(global_io()?.as_raw_fd()))
}

#[unsafe(no_mangle)]
#[must_use]
pub extern "C" fn io_get_config() -> *const IOConfig {
    map_panic_to_exit_with_error(|| Ok(global_io()?.io_config))
}

#[unsafe(no_mangle)]
pub extern "C" fn io_lock() {
    map_panic_to_exit_with_error(|| {
        global_io()?.process_command(Command::Lock);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot() {
    map_panic_to_exit_with_error(|| {
        global_io()?.process_command(Command::Reboot);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown() {
    map_panic_to_exit_with_error(|| {
        global_io()?.process_command(Command::Shutdown);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout() {
    map_panic_to_exit_with_error(|| {
        global_io()?.process_command(Command::Logout);
        Ok(())
    });
}
#[unsafe(no_mangle)]
#[expect(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn io_trigger_tray(uuid: *const core::ffi::c_char) {
    map_panic_to_exit_with_error(|| {
        let uuid = unsafe { core::ffi::CStr::from_ptr(uuid) }.to_str()?;

        global_io()?.process_command(Command::TriggerTray {
            uuid: StringRef::new(uuid),
        });

        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_wifi_editor() {
    map_panic_to_exit_with_error(|| {
        global_io()?.process_command(Command::SpawnWiFiEditor);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_bluetooh_editor() {
    map_panic_to_exit_with_error(|| {
        global_io()?.process_command(Command::SpawnBluetoothEditor);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor() {
    map_panic_to_exit_with_error(|| {
        global_io()?.process_command(Command::SpawnSystemMonitor);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_wallpaper() {
    map_panic_to_exit_with_error(|| {
        global_io()?.process_command(Command::ChangeWallpaper);
        Ok(())
    });
}
