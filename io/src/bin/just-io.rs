use layer_shell_io::{io_init, io_poll_events, io_spawn_thread};
use std::time::Duration;

fn main() {
    io_init();

    io_spawn_thread();

    loop {
        let _ = io_poll_events();
        std::thread::sleep(Duration::from_millis(50));
    }
}
