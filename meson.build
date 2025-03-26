project(
    'layer-shell',
    'cpp',
    version: '0.1',
    default_options: [
        'warning_level=3',
        'cpp_std=c++23',
    ],
)

dependencies = [
    dependency('gtk4'),
    dependency('gtkmm-4.0'),
    dependency('gtk4-layer-shell-0'),
    dependency('vte-2.91-gtk4'),
]

xxd = generator(
    find_program('xxd'),
    output: '@PLAINNAME@.xxd',
    arguments: ['-i', '@INPUT@', '@OUTPUT@'],
)

sources = files(
    'utils/css.cpp',
    'utils/icons.cpp',
    'utils/strings.cpp',
    'utils/subscriber.cpp',
    'utils/weather-helper.cpp',
    'widgets/change_theme.cpp',
    'widgets/cpu.cpp',
    'widgets/cpu/label.cpp',
    'widgets/htop.cpp',
    'widgets/language.cpp',
    'widgets/launcher/row.cpp',
    'widgets/memory.cpp',
    'widgets/network.cpp',
    'widgets/network/popover.cpp',
    'widgets/session.cpp',
    'widgets/sound.cpp',
    'widgets/time.cpp',
    'widgets/tray.cpp',
    'widgets/weather.cpp',
    'widgets/weather/daily_grid.cpp',
    'widgets/weather/grid.cpp',
    'widgets/weather/hourly_grid.cpp',
    'widgets/workspaces.cpp',
    'windows/base.cpp',
    'windows/htop.cpp',
    'windows/launcher.cpp',
    'windows/ping.cpp',
    'windows/session.cpp',
    'windows/top-bar.cpp',
    'windows/weather.cpp',
    'main.cpp',
)

subdir('icons')
subdir('css')
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
    cpp_args: ['-Wno-c23-extensions'],
    dependencies: dependencies,
    link_with: [rustlib],
    link_depends: [rustlib],
    cpp_pch: 'pch/pch.hpp',
    install: true,
    install_dir: join_paths(get_option('prefix'), get_option('bindir')),
)
