use layer_shell_io::{io_init, io_spawn_thread};

fn main() {
    io_init();

    io_spawn_thread();
}
