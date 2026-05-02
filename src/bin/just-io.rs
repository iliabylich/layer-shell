use layer_shell_io::{Event, io_handle_readable, io_init, io_wait_readable};

extern "C" fn on_event(event: *const Event) {
    let event = unsafe { &*event };
    log::info!("{event:?}");
}

fn main() -> Result<(), ()> {
    assert!(io_init(on_event, false), "init failed");

    loop {
        // log::info!("Waiting...");
        assert!(io_wait_readable(), "wait_readable failed");
        // log::info!("Wait finished...");

        // log::info!("Processing...");
        assert!(io_handle_readable(), "wait_readable failed");
    }
}
