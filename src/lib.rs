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
use anyhow::Result;

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
) -> *mut std::ffi::c_void {
    map_panic_to_exit_with_error(|| {
        IO::init()?;
        Ok(Box::into_raw(Box::new(IO::new((callback, data))?)).cast())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_deinit(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let mut io = unsafe { Box::from_raw(io.cast::<IO>()) };
        io.stop();
        Ok(())
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn io_handle_readable(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        io.handle_readable()
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn io_wait_readable(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        io.wait_readable();
        Ok(())
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn io_as_raw_fd(io: *mut std::ffi::c_void) -> i32 {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        Ok(io.as_raw_fd())
    })
}

#[unsafe(no_mangle)]
#[must_use]
pub extern "C" fn io_get_config(io: *mut std::ffi::c_void) -> *const IOConfig {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        Ok(io.io_config)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_lock(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        io.process_command(Command::Lock);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        io.process_command(Command::Reboot);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        io.process_command(Command::Shutdown);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        io.process_command(Command::Logout);
        Ok(())
    });
}
#[unsafe(no_mangle)]
#[expect(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn io_trigger_tray(io: *mut std::ffi::c_void, uuid: *const core::ffi::c_char) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        let uuid = unsafe { core::ffi::CStr::from_ptr(uuid) }.to_str()?;

        io.process_command(Command::TriggerTray {
            uuid: StringRef::new(uuid),
        });

        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_wifi_editor(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        io.process_command(Command::SpawnWiFiEditor);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_bluetooh_editor(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        io.process_command(Command::SpawnBluetoothEditor);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        io.process_command(Command::SpawnSystemMonitor);
        Ok(())
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_wallpaper(io: *mut std::ffi::c_void) {
    map_panic_to_exit_with_error(|| {
        let io = unsafe { &mut *io.cast::<IO>() };
        io.process_command(Command::ChangeWallpaper);
        Ok(())
    });
}
