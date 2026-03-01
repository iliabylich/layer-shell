use layer_shell_io::{Event, io_handle_readable, io_init, io_wait_readable};

extern "C" fn on_event(event: *const Event) {
    let event = unsafe { &*event };
    log::info!("{event:?}");
}

fn main() {
    io_init(on_event, false);

    loop {
        // log::info!("Waiting...");
        io_wait_readable();
        // log::info!("Wait finished...");

        // log::info!("Processing...");
        io_handle_readable();
    }
}
