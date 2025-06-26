find_program('cargo')

rustlib = custom_target(
    'liblayer_shell_io.a',
    command: [
        files('build.sh'),
        get_option('buildtype'),
        meson.project_source_root(),
        '@OUTPUT@',
    ],
    output: 'liblayer_shell_io.a',
    build_always_stale: true,
    build_by_default: true,
)
sources += rustlib

cbindgen = find_program('cbindgen', version: '>= 0.28')

bindings = custom_target(
    'bindings.h',
    command: [
        cbindgen,
        '--crate', 'io',
        '--config', files('cbindgen.toml'),
        '--quiet',
        '--output', '@OUTPUT@',
        '@SOURCE_ROOT@',
    ],
    output: 'bindings.h',
    build_always_stale: true,
    build_by_default: true,
)
sources += bindings
