use layer_shell_io::{Event, io_handle_readable, io_init, io_wait_readable};

fn on_event(event: *const Event) {
    let Some(event) = (unsafe { event.as_ref() }) else {
        log::error!("NULL event received");
        std::process::exit(1);
    };
    log::info!("{event:?}");
}

fn main() {
    let io = io_init(on_event);

    loop {
        // log::info!("Waiting...");
        io_wait_readable(io);
        // log::info!("Wait finished...");

        // log::info!("Processing...");
        io_handle_readable(io);
    }
}
