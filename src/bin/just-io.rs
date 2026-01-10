use layer_shell_io::{Event, io_handle_readable, io_init, io_wait_readable};

fn on_event(event: Event) {
    println!("on_event: {event:?}");
}

fn main() {
    let io = io_init(on_event);

    loop {
        println!("Waiting...");
        io_wait_readable(io);
        println!("Wait finished...");

        println!("Processing...");
        io_handle_readable(io);
    }
}
