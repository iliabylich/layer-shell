cargo = find_program('cargo')
cargo_args = [
    '--manifest-path', meson.project_source_root() / 'Cargo.toml',
    '-p', 'layer-shell-io-staticlib',
]
if get_option('buildtype') == 'release'
    cargo_args += ['--release']
endif
cargo_args += ['--features standalone-staticlib']

rustlib = custom_target(
    'liblayer_shell_io.a',
    command: [
        'sh',
        '-c', 'cargo build @0@ && cp @1@ @OUTPUT@'.format(
            ' '.join(cargo_args),
            meson.project_source_root() / 'target' / get_option('buildtype') / 'liblayer_shell_io.a',
        ),
    ],
    output: 'liblayer_shell_io.a',
    build_always_stale: true,
    build_by_default: true,
    console: true,
)

cbindgen = find_program('cbindgen', version: '>= 0.28')

bindings = custom_target(
    'bindings.hpp',
    command: [
        cbindgen,
        '--config', files('cbindgen.toml'),
        '--quiet',
        '--output', '@OUTPUT@',
        '@SOURCE_ROOT@',
    ],
    output: 'bindings.hpp',
    build_always_stale: true,
    build_by_default: true,
)
