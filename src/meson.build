cbindgen = find_program('cbindgen')

source_root = meson.project_source_root()

cargo_cmd = [cbindgen]
cargo_cmd += ['--config', source_root / 'src/cbindgen.toml']
cargo_cmd += '--quiet'
cargo_cmd += source_root
cargo_cmd += ['--output', '@OUTPUT@']

bindings = custom_target(
    'bindings.h',
    command: cargo_cmd,
    output: 'bindings.h',
    build_always_stale: true,
    build_by_default: true,
)
sources += bindings
