use layer_shell_io::{Event, io_handle_readable, io_init, io_wait_readable};

extern "C" fn on_event(event: *const Event, _data: *mut std::ffi::c_void) {
    let event = unsafe { &*event };
    log::trace!(target: "just-io", "{event:?}");
}

fn main() -> Result<(), ()> {
    let io = io_init(on_event, std::ptr::null_mut());

    loop {
        // log::info!("Waiting...");
        io_wait_readable(io);
        // log::info!("Wait finished...");

        // log::info!("Processing...");
        io_handle_readable(io);
    }
}
