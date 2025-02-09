use layer_shell_io::{layer_shell_io_init, layer_shell_io_spawn_thread};

fn main() {
    let ctx = layer_shell_io_init();
    layer_shell_io_spawn_thread(ctx);
    std::thread::sleep(std::time::Duration::MAX);
}
