project(
    'layer-shell',
    'c',
    version: '0.1',
    default_options: [
        'warning_level=3',
        'c_std=gnu23',
    ],
)

sources = []
subdir('ui')
subdir('src')
subdir('systemd')

prefix = get_option('prefix')
bindir = get_option('bindir')

if get_option('buildtype') == 'debug'
    asan_options = ['-fsanitize=address']
else
    asan_options = []
endif

executable(
    'layer-shell',
    sources,
    c_args: [
        '-Wno-c23-extensions',
        '-DGDK_DISABLE_DEPRECATED',
        '-DGTK_DISABLE_DEPRECATED',
    ]
    + asan_options,
    dependencies: [
        dependency('glib-2.0'),
        dependency('gtk4'),
        dependency('gtk4-layer-shell-0'),
        dependency('vte-2.91-gtk4'),
        dependency('liburing'),
    ],
    link_args: ['-lm'] + asan_options,
    c_pch: 'ui/pch/pch.h',
    install: true,
    install_dir: join_paths(prefix, bindir),
)
