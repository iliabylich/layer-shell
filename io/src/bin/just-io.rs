use io::{io_init, io_run_in_place};

fn main() {
    io_init();

    io_run_in_place().unwrap()
}
