#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use layer_shell_io::{IoEvent, io_deinit, io_handle_readable, io_init, io_wait_readable};

static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

extern "C" fn on_event(event: &IoEvent, _data: *mut core::ffi::c_void) {
    log::trace!(target: "just-io", "{event:?}");

    if matches!(event, IoEvent::Exit) {
        SHOULD_EXIT.store(true, Ordering::Relaxed);
    }
}

#[unsafe(no_mangle)]
extern "C" fn main() {
    let io = io_init(on_event, core::ptr::null_mut());

    while !SHOULD_EXIT.load(Ordering::Relaxed) {
        // log::info!("Waiting...");
        io_wait_readable(io);
        // log::info!("Wait finished...");

        // log::info!("Processing...");
        io_handle_readable(io);
    }

    log::info!("Exiting...");
    unsafe {
        io_deinit(io);
        libc::exit(0);
    }
}

include!("../panic_handler.rs");
