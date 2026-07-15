qt6_dep = dependency(
    'qt6',
    modules: [
        'Core',
        'Gui',
        'Widgets',
    ],
)

cpp = meson.get_compiler('cpp')
layershellqt_dep = cpp.find_library('LayerShellQtInterface')

qtermwidget_dep = dependency('qtermwidget6')

qt6 = import('qt6')
qt6_sources = qt6.preprocess(
    moc_headers: files(
        'KbModOverlay.hpp',
        'Overlay.hpp',
        'SessionOverlay.hpp',
        'SoundOverlay.hpp',
        'TerminalOverlay.hpp',
        'TopBar.hpp',
        'Tray.hpp',
        'UiModel.hpp',
        'WeatherOverlay.hpp',
    ),
    dependencies: [qt6_dep],
)

xxd = find_program('xxd')
xxd = generator(
    xxd,
    output: '@PLAINNAME@.xxd',
    arguments: ['-n', '@PLAINNAME@', '-i', '@INPUT@', '@OUTPUT@'],
)

sassc = find_program('sassc')
sassc = generator(
    sassc,
    output: '@PLAINNAME@',
    arguments: ['@INPUT@', '@OUTPUT@'],
)

if get_option('buildtype') == 'debug'
    asan_options = ['-fsanitize=address']
    gc_sections_options = []
else
    asan_options = []
    gc_sections_options = ['-Wl,--gc-sections']
endif

executable(
    'layer-shell',
    [
        files(
            'Config.cpp',
            'Event.cpp',
            'KbModOverlay.cpp',
            'Overlay.cpp',
            'SessionOverlay.cpp',
            'SoundOverlay.cpp',
            'TerminalOverlay.cpp',
            'TopBar.cpp',
            'Tray.cpp',
            'UiModel.cpp',
            'WeatherOverlay.cpp',
            'main.cpp',
        ),
        qt6_sources,
        xxd.process(
            sassc.process('style.scss'),
        ),
        rustlib,
        bindings,
    ],
    cpp_args: asan_options,
    dependencies: [
        qt6_dep,
        layershellqt_dep,
        qtermwidget_dep,
        dependency('liburing'),
    ],
    include_directories: include_directories('..'),
    install: true,
    install_dir: join_paths(get_option('prefix'), get_option('bindir')),
    link_args: asan_options + gc_sections_options,
)
