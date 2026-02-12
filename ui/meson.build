sources += files(
    'base_window.c',
    'caps_lock_window.c',
    'cpu_item.c',
    'cpu_model.c',
    'io_model.c',
    'css.c',
    'logger.c',
    'main.c',
    'ping_window.c',
    'session_window.c',
    'sound_window.c',
    'terminal_window.c',
    'top_bar.c',
    'tray_app_item.c',
    'tray_model.c',
    'weather_helper.c',
    'weather_window.c',
    'workspace_item.c',
    'workspaces_model.c',
)

blueprints = custom_target(
    'blueprints',
    input: files(
        'caps_lock_window.blp',
        'ping_window.blp',
        'session_window.blp',
        'sound_window.blp',
        'terminal_window.blp',
        'top_bar.blp',
        'weather_window.blp',
    ),
    output: [
        'caps_lock_window.ui',
        'ping_window.ui',
        'session_window.ui',
        'sound_window.ui',
        'terminal_window.ui',
        'top_bar.ui',
        'weather_window.ui',
    ],
    command: [
        find_program('blueprint-compiler'),
        'batch-compile',
        '@OUTDIR@',
        '@CURRENT_SOURCE_DIR@',
        '@INPUT@',
    ],
)

sources += gnome.compile_resources(
    'layer-shell-resources',
    'layer-shell.gresource.xml',
    source_dir: meson.current_build_dir(),
    dependencies: blueprints,
)

subdir('css')
