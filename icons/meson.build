cat = find_program('cat')

icon_names = [
    'change_theme',
    'download',
    'foggy',
    'partly_cloudy',
    'power',
    'question_mark',
    'rainy',
    'snowy',
    'sunny',
    'thunderstorm',
    'upload',
    'wifi',
]

icon_paths = []
foreach name : icon_names
    icon_paths += ''.join([name, '.png'])
endforeach

embedded_icons = xxd.process(icon_paths)

all_icons_embedded = custom_target(
    'all-icons-embedded',
    command: [cat, '@INPUT@'],
    capture: true,
    input: embedded_icons,
    output: 'all-icons-embedded.hpp',
)
sources += all_icons_embedded

cmd = [meson.project_source_root() / 'icons' / 'make-x-icons.sh']
cmd += icon_names
x_icons = custom_target(
    'x-icons',
    command: cmd,
    capture: true,
    output: 'x-icons.hpp',
)
sources += x_icons
