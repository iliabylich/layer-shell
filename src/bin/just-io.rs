use layer_shell_io::IO;

fn main() {
    let (io_ctx, _ui_ctx) = IO::init();

    IO::run_in_place(io_ctx).unwrap();
}
