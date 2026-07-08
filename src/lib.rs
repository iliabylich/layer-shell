#![no_std]
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
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::option_if_let_else)]
#![expect(clippy::redundant_pub_crate)]
#![expect(clippy::cast_possible_truncation)]
#![expect(clippy::arithmetic_side_effects)]
#![expect(clippy::missing_safety_doc)]

extern crate alloc;

mod actor;
mod command;
mod config;
mod event;
mod event_queue;
mod ffi;
mod io;
mod liburing;
mod logger;
mod modules;
mod sansio;
mod user_data;
mod utils;

use alloc::boxed::Box;
use core::ptr::NonNull;

use command::Command;
pub use event::Event;
pub use ffi::FFIArray;

use crate::{
    io::IO,
    utils::{StringRef, StringRefExt as _},
};
use anyhow::Result;

fn exit_if_err<T>(f: impl FnOnce() -> Result<T>) -> T {
    match f() {
        Ok(out) => out,
        Err(err) => {
            log::error!("error returned: {err:?}");
            unsafe { libc::exit(1) };
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init(
    callback: extern "C" fn(event: &Event, *mut core::ffi::c_void),
    data: *mut core::ffi::c_void,
) -> NonNull<IO> {
    exit_if_err(|| {
        IO::init()?;
        let ptr = Box::into_raw(Box::new(IO::new((callback, data))?));
        Ok(unsafe { NonNull::new_unchecked(ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_deinit(io: NonNull<IO>) {
    exit_if_err(|| {
        let mut io = unsafe { Box::from_raw(io.as_ptr()) };
        io.stop();
        Ok(())
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_handle_readable(mut io: NonNull<IO>) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        io.handle_readable()
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_wait_readable(mut io: NonNull<IO>) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        io.wait_readable()
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_as_raw_fd(io: NonNull<IO>) -> i32 {
    exit_if_err(move || {
        let io = unsafe { io.as_ref() };
        Ok(io.fd())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_lock(mut io: NonNull<IO>) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        io.process_command(Command::Lock)
    });
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_reboot(mut io: NonNull<IO>) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        io.process_command(Command::Reboot)
    });
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_shutdown(mut io: NonNull<IO>) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        io.process_command(Command::Shutdown)
    });
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_logout(mut io: NonNull<IO>) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        io.process_command(Command::Logout)
    });
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_trigger_tray(mut io: NonNull<IO>, uuid: *const core::ffi::c_char) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        let uuid = unsafe { core::ffi::CStr::from_ptr(uuid) }.to_str()?;

        io.process_command(Command::TriggerTray {
            uuid: StringRef::new(uuid),
        })
    });
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_spawn_wifi_editor(mut io: NonNull<IO>) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        io.process_command(Command::SpawnWiFiEditor)
    });
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_spawn_bluetooh_editor(mut io: NonNull<IO>) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        io.process_command(Command::SpawnBluetoothEditor)
    });
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_spawn_system_monitor(mut io: NonNull<IO>) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        io.process_command(Command::SpawnSystemMonitor)
    });
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_change_wallpaper(mut io: NonNull<IO>) {
    exit_if_err(move || {
        let io = unsafe { io.as_mut() };
        io.process_command(Command::ChangeWallpaper)
    });
}

#[repr(C)]
pub struct CommandToExec {
    pub ptr: *const StringRef,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub const unsafe extern "C" fn io_get_ping_cmd(mut io: NonNull<IO>) -> CommandToExec {
    let io = unsafe { io.as_mut() };
    let ptr = io.config.ping.as_ptr();
    let len = io.config.ping.len();
    CommandToExec { ptr, len }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_get_terminal_label(mut io: NonNull<IO>) -> StringRef {
    let io = unsafe { io.as_mut() };
    io.config.terminal.label.clone()
}
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn io_get_terminal_cmd(mut io: NonNull<IO>) -> CommandToExec {
    let io = unsafe { io.as_mut() };
    let ptr = io.config.terminal.command.as_ptr();
    let len = io.config.terminal.command.len();
    CommandToExec { ptr, len }
}
