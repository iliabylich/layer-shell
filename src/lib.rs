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
// #![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

mod command;
mod config;
mod emitter;
mod event;
/// cbindgen:ignore
#[expect(
    dead_code,
    unsafe_op_in_unsafe_fn,
    trivial_casts,
    non_camel_case_types,
    clippy::indexing_slicing,
    clippy::ptr_as_ptr,
    clippy::ref_as_ptr,
    clippy::missing_const_for_fn,
    clippy::use_self,
    clippy::redundant_pub_crate,
    clippy::arithmetic_side_effects
)]
mod external;
mod io;
mod liburing;
mod logger;
mod modules;
#[cfg(feature = "standalone-staticlib")]
mod panic_handler;
mod sansio;
mod user_data;
mod utils;

pub use event::IoEvent;
pub use modules::{
    DAILY_WEATHER_FORECAST_LENGTH, HOURLY_WEATHER_FORECAST_LENGTH, KbModKind, MAX_CPU_COUNT,
    MaybeRootTrayElement, TrayElement, TrayLabel, TrayMenu, WeatherCode, WeatherOnDay,
    WeatherOnHour,
};
pub use utils::{FixedSizeArrray, StringRef};

use crate::{io::IO, logger::Logger, utils::log_err_and_exit};
use command::Command;

#[unsafe(no_mangle)]
pub extern "C" fn io_init(
    callback: extern "C" fn(event: &IoEvent, *mut core::ffi::c_void),
    data: *mut core::ffi::c_void,
) -> *mut IO {
    Logger::init();

    unsafe {
        let ptr = libc::malloc(size_of::<IO>()).cast::<IO>();
        if ptr.is_null() {
            log_err_and_exit!("failed to malloc()");
        }
        ptr.write(IO::new(callback, data));
        (&mut *ptr).start();
        ptr
    }
}

fn with_io<T>(io: *mut IO, f: impl FnOnce(&mut IO) -> T) -> T {
    let Some(mut io) = core::ptr::NonNull::new(io) else {
        log_err_and_exit!("IO pointer is null");
    };

    unsafe { f(io.as_mut()) }
}

#[expect(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn io_deinit(io: *mut IO) {
    with_io(io, IO::stop);

    unsafe {
        core::ptr::drop_in_place(io);
        libc::free(io.cast());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_handle_readable(io: *mut IO) {
    with_io(io, IO::handle_readable);
}

#[unsafe(no_mangle)]
pub extern "C" fn io_wait_readable(io: *mut IO) {
    with_io(io, IO::wait_readable);
}

#[unsafe(no_mangle)]
pub extern "C" fn io_as_raw_fd(io: *mut IO) -> i32 {
    with_io(io, |io| io.fd())
}

#[unsafe(no_mangle)]
pub extern "C" fn io_lock(io: *mut IO) {
    with_io(io, |io| io.process_command(Command::Lock));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot(io: *mut IO) {
    with_io(io, |io| io.process_command(Command::Reboot));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown(io: *mut IO) {
    with_io(io, |io| io.process_command(Command::Shutdown));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout(io: *mut IO) {
    with_io(io, |io| io.process_command(Command::Logout));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_trigger_tray(io: *mut IO, service: u32, id: i32) {
    with_io(io, |io| {
        io.process_command(Command::TriggerTray { service, id });
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_wifi_editor(io: *mut IO) {
    with_io(io, |io| io.process_command(Command::SpawnWiFiEditor));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_bluetooh_editor(io: *mut IO) {
    with_io(io, |io| io.process_command(Command::SpawnBluetoothEditor));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor(io: *mut IO) {
    with_io(io, |io| io.process_command(Command::SpawnSystemMonitor));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_wallpaper(io: *mut IO) {
    with_io(io, |io| io.process_command(Command::ChangeWallpaper));
}

#[repr(C)]
pub struct CommandToExec {
    pub ptr: *const StringRef,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn io_get_ping_cmd(io: *mut IO) -> CommandToExec {
    with_io(io, |io| {
        let ptr = io.config.ping.as_ptr();
        let len = io.config.ping.len();
        CommandToExec { ptr, len }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_get_terminal_label(io: *mut IO) -> StringRef {
    with_io(io, |io| io.config.terminal.label.clone())
}
#[unsafe(no_mangle)]
pub extern "C" fn io_get_terminal_cmd(io: *mut IO) -> CommandToExec {
    with_io(io, |io| {
        let ptr = io.config.terminal.command.as_ptr();
        let len = io.config.terminal.command.len();
        CommandToExec { ptr, len }
    })
}
