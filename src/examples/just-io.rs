#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};
use layer_shell_io::{Event, io_deinit, io_handle_readable, io_init, io_wait_readable};

static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

extern "C" fn on_event(event: &Event, _data: *mut core::ffi::c_void) {
    log::trace!(target: "just-io", "{event:?}");

    if matches!(event, Event::Exit) {
        SHOULD_EXIT.store(true, Ordering::Relaxed);
    }
}

fn main() -> Result<(), ()> {
    let io = io_init(on_event, core::ptr::null_mut());

    while !SHOULD_EXIT.load(Ordering::Relaxed) {
        // log::info!("Waiting...");
        unsafe { io_wait_readable(io) };
        // log::info!("Wait finished...");

        // log::info!("Processing...");
        unsafe { io_handle_readable(io) };
    }

    log::info!("Exiting...");
    unsafe { io_deinit(io) };
    Ok(())
}
