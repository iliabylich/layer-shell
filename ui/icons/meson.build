icon_names = [
    'change_theme',
    'download_speed',
    'foggy',
    'partly_cloudy',
    'power',
    'question_mark',
    'rainy',
    'snowy',
    'sunny',
    'thunderstorm',
    'upload_speed',
    'wifi',
]

icon_paths = []
foreach name : icon_names
    icon_paths += ''.join([name, '.png'])
endforeach

embedded_icons = xxd.process(icon_paths)

cat = find_program('cat')

all_icons_embedded = custom_target(
    'all-icons-embedded',
    command: [cat, '@INPUT@'],
    capture: true,
    input: embedded_icons,
    output: 'all-icons-embedded.h',
)
sources += all_icons_embedded

cmd = [meson.project_source_root() / 'ui' / 'icons' / 'make-x-icons.sh']
cmd += icon_names
x_icons = custom_target(
    'x-icons',
    command: cmd,
    capture: true,
    output: 'x-icons.h',
)
sources += x_icons
