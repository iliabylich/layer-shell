project(
  'layer-shell',
  'cpp',
  version: '0.1',
  default_options: [
    'warning_level=3',
    'cpp_std=c++23',
  ],
)

gtk4 = dependency('gtk4', required: true)
gtkmm = dependency('gtkmm-4.0', required: true)
gtk4_layer_shell = dependency('gtk4-layer-shell-0', required: true)
vte = dependency('vte-2.91-gtk4', required: true)
just = find_program('just', required: true)

buildtype = get_option('buildtype')

sources = files(
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
  'windows/session.cpp',
  'windows/top-bar.cpp',
  'windows/weather.cpp',
  'main.cpp',
)

rustlib = custom_target(
  'liblayer_shell_io.so',
  command: [just, f'cargo-@buildtype@', '@OUTPUT@'],
  output: 'liblayer_shell_io.so',
  build_always_stale: true,
  build_by_default: true,
  install: true,
  install_dir: get_option('libdir'),
)

executable(
  meson.project_name(),
  sources,
  cpp_args: ['-Wno-c23-extensions'],
  dependencies: [gtk4, gtk4_layer_shell, vte, gtkmm],
  link_with: [rustlib],
  link_depends: [rustlib],
  cpp_pch: 'pch/pch.hpp',
  install: true,
  install_dir: get_option('bindir'),
)
