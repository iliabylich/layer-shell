project(
    'layer-shell',
    'c',
    version: '0.1',
    default_options: [
        'warning_level=3',
        'c_std=gnu23',
    ],
)

dependencies = [
    dependency('glib-2.0'),
    dependency('gtk4'),
    dependency('gtk4-layer-shell-0'),
    dependency('vte-2.91-gtk4'),
]

xxd = generator(
    find_program('xxd'),
    output: '@PLAINNAME@.xxd',
    arguments: ['-i', '@INPUT@', '@OUTPUT@'],
)

subdir('ui')
subdir('src')

cargo = find_program('cargo')
cargo_cmd = [cargo, 'build']
if get_option('buildtype') == 'release'
    cargo_cmd += '--release'
    outfile = source_root / 'target/release/liblayer_shell_io.so'
else
    outfile = source_root / 'target/debug/liblayer_shell_io.so'
endif
cargo_cmd += ['&&', 'cp', outfile, '@OUTPUT@']
rustlib = custom_target(
    'liblayer_shell_io.so',
    command: cargo_cmd,
    output: 'liblayer_shell_io.so',
    build_always_stale: true,
    build_by_default: true,
    install: true,
    install_dir: join_paths(get_option('prefix'), get_option('libdir')),
)

executable(
    meson.project_name(),
    sources,
    c_args: ['-Wno-c23-extensions'],
    dependencies: dependencies,
    link_with: [rustlib],
    link_args: ['-lm'],
    link_depends: [rustlib],
    c_pch: 'ui/pch/pch.h',
    install: true,
    install_dir: join_paths(get_option('prefix'), get_option('bindir')),
)
