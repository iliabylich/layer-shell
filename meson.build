project(
  'layer-shell',
  'c',
  version: '0.1',
  default_options: [
    'warning_level=3',
    'c_std=c23',
  ],
)

gtk4 = dependency('gtk4')
gtk4_layer_shell = dependency('gtk4-layer-shell-0')
vte = dependency('vte-2.91-gtk4')

buildtype = get_option('buildtype')
message('Build type', buildtype)

rust_dir = meson.project_source_root() / 'target' / buildtype
message('Rust dir', rust_dir)

custom_target(
  'so-file',
  build_always_stale: true,
  build_by_default: true,
  command: ['cargo', 'build', buildtype == 'release' ? '--release' : '--all-targets'],
  output: ['liblayer_shell_io.so'],
)

sources = [
  'utils/css.c',
  'utils/icons.c',
  'utils/weather-helper.c',
  'utils/window.c',
  # widgets
  'widgets/cpu.c',
  'widgets/htop.c',
  'widgets/language.c',
  'widgets/memory.c',
  'widgets/network.c',
  'widgets/session.c',
  'widgets/sound.c',
  'widgets/time.c',
  'widgets/tray.c',
  'widgets/weather.c',
  'widgets/workspaces.c',
  # windows
  'windows/htop.c',
  'windows/launcher.c',
  'windows/network.c',
  'windows/session.c',
  'windows/top-bar.c',
  'windows/weather.c',
  # main
  'main.c',
]

executable(
  meson.project_name(),
  sources,
  dependencies: [gtk4, gtk4_layer_shell, vte],
  link_args: [f'-L@rust_dir@', '-llayer_shell_io'],
  install: true,
  build_rpath: f'$ORIGIN/../target/@buildtype@',
)
