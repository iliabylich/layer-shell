sources += files(
    'view_models/clock_model.c',
    'view_models/cpu_item.c',
    'view_models/cpu_model.c',
    'view_models/io_model.c',
    'view_models/kb_mod_model.c',
    'view_models/language_model.c',
    'view_models/memory_model.c',
    'view_models/network_model.c',
    'view_models/overlays_model.c',
    'view_models/sound_model.c',
    'view_models/tray_app_item.c',
    'view_models/tray_model.c',
    'view_models/weather_day_item.c',
    'view_models/weather_hour_item.c',
    'view_models/weather_model.c',
    'base_overlay.c',
    'css.c',
    'gobject_helper.c',
    'kb_mod_overlay.c',
    'logger.c',
    'main.c',
    'ping_overlay.c',
    'session_overlay.c',
    'sound_overlay.c',
    'terminal_overlay.c',
    'top_bar.c',
    'weather_helper.c',
    'weather_overlay.c',
)

blueprints = custom_target(
    'blueprints',
    input: files(
        'kb_mod_overlay.blp',
        'ping_overlay.blp',
        'session_overlay.blp',
        'sound_overlay.blp',
        'terminal_overlay.blp',
        'top_bar.blp',
        'weather_overlay.blp',
    ),
    output: [
        'kb_mod_overlay.ui',
        'ping_overlay.ui',
        'session_overlay.ui',
        'sound_overlay.ui',
        'terminal_overlay.ui',
        'top_bar.ui',
        'weather_overlay.ui',
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
