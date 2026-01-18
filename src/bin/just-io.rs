use layer_shell_io::{Event, io_handle_readable, io_init, io_wait_readable};

extern "C" fn on_event(event: *const Event) {
    println!("{:?}", unsafe { event.as_ref().unwrap() });
}

fn main() {
    let io = io_init(on_event);

    loop {
        // println!("Waiting...");
        io_wait_readable(io);
        // println!("Wait finished...");

        // println!("Processing...");
        io_handle_readable(io);
    }
}
