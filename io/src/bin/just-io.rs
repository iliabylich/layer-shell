use layer_shell_io::{io_init, io_run_in_place, io_take_ctx};

fn main() {
    io_init();

    let (etx, crx, config, pipe_writer) = io_take_ctx();

    io_run_in_place(config, etx, crx, pipe_writer).unwrap();
}
