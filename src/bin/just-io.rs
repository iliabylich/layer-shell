use layer_shell_io::{io_init, io_run_in_place};

fn main() {
    let ctx = io_init();
    let io_ctx = unsafe { *Box::from_raw(ctx.io) };

    io_run_in_place(io_ctx).unwrap();
}
