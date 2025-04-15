use layer_shell_io::{io_init, io_run_in_place};

fn main() {
    let (io_ctx, _ui_ctx) = io_init();

    io_run_in_place(io_ctx).unwrap();
}
