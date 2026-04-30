use layer_shell_io::{Event, io_handle_readable, io_init, io_wait_readable};

extern "C" fn on_event(event: *const Event) {
    let event = unsafe { &*event };
    log::info!("{event:?}");
}

fn main() -> Result<(), ()> {
    if !io_init(on_event, false) {
        eprintln!("io_init failed");
        return Ok(());
    }

    loop {
        // log::info!("Waiting...");
        io_wait_readable();
        // log::info!("Wait finished...");

        // log::info!("Processing...");
        io_handle_readable();
    }
}
