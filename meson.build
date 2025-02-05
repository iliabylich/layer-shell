project(
  'layer-shell',
  'cpp',
  version: '0.1',
  default_options: [
    'warning_level=3',
    'cpp_std=c++23',
  ],
)

gtk4 = dependency('gtk4')
gtkmm = dependency('gtkmm-4.0')
gtk4_layer_shell = dependency('gtk4-layer-shell-0')
vte = dependency('vte-2.91-gtk4')

buildtype = get_option('buildtype')
message('Build type', buildtype)

rust_out_dir = meson.project_source_root() / 'target' / buildtype
message('Rust dir', rust_out_dir)

sources = [
  'utils/css.cpp',
  'utils/icons.cpp',
  'utils/weather-helper.cpp',
  'widgets/cpu.cpp',
  'widgets/htop.cpp',
  'widgets/language.cpp',
  'widgets/memory.cpp',
  'widgets/network.cpp',
  'widgets/session.cpp',
  'widgets/sound.cpp',
  'widgets/time.cpp',
  'widgets/tray.cpp',
  'widgets/weather.cpp',
  'widgets/workspaces.cpp',
  'windows/htop.cpp',
  'windows/launcher.cpp',
  'windows/network.cpp',
  'windows/session.cpp',
  'windows/top-bar.cpp',
  'windows/weather.cpp',
  'main.cpp',
]

executable(
  meson.project_name(),
  sources,
  cpp_args: ['-Wno-c23-extensions'],
  dependencies: [gtk4, gtk4_layer_shell, vte, gtkmm],
  cpp_pch: 'pch/pch.hpp',
  link_args: [f'-L@rust_out_dir@', '-llayer_shell_io'],
  build_rpath: f'$ORIGIN/../target/@buildtype@',
  install: true,
  install_dir: get_option('bindir'),
)

install_data(
  rust_out_dir / 'liblayer_shell_io.so',
  install_dir: get_option('libdir'),
)
