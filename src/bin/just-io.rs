use layer_shell_io::{io_init, io_run_in_place};

fn main() {
    let ctx = io_init();

    unsafe { io_run_in_place(ctx.io).unwrap() };
}
