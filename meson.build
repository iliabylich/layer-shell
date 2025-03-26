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
xxd = find_program('xxd', required: true)
cat = find_program('cat', required: true)
sassc = find_program('sassc', required: true)

buildtype = get_option('buildtype')

rustlib = custom_target(
  'liblayer_shell_io.so',
  command: [just, f'cargo-@buildtype@', '@OUTPUT@'],
  output: 'liblayer_shell_io.so',
  build_always_stale: true,
  build_by_default: true,
  install: true,
  install_dir: get_option('libdir'),
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

icons = files(
  'icons/change_theme.png',
  'icons/download.png',
  'icons/foggy.png',
  'icons/partly_cloudy.png',
  'icons/power.png',
  'icons/question_mark.png',
  'icons/rainy.png',
  'icons/snowy.png',
  'icons/sunny.png',
  'icons/thunderstorm.png',
  'icons/upload.png',
  'icons/wifi.png',
)

xxd_gen = generator(
  xxd,
  output: '@PLAINNAME@.xxd',
  arguments: ['-i', '@INPUT@', '@OUTPUT@'],
)
embedded_icons = xxd_gen.process(icons)
all_icons_embedded = custom_target(
  'all-icons-embedded',
  command: [cat, '@INPUT@'],
  capture: true,
  input: embedded_icons,
  output: 'all-icons-embedded.hpp',
)
sources += all_icons_embedded

sassc_gen = generator(
  sassc,
  output: '@PLAINNAME@',
  arguments: ['@INPUT@', '@OUTPUT@'],
)
main_css = sassc_gen.process('main.scss')
main_css_embedded = xxd_gen.process(main_css)
sources += main_css_embedded

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
