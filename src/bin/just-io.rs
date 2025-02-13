use layer_shell_io::{layer_shell_io_init, layer_shell_io_run_in_place};

fn main() {
    let ctx = layer_shell_io_init();

    layer_shell_io_run_in_place(ctx).unwrap();
}
