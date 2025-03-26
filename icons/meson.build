cat = find_program('cat')

icons = files(
    'change_theme.png',
    'download.png',
    'foggy.png',
    'partly_cloudy.png',
    'power.png',
    'question_mark.png',
    'rainy.png',
    'snowy.png',
    'sunny.png',
    'thunderstorm.png',
    'upload.png',
    'wifi.png',
)

embedded_icons = xxd.process(icons)

all_icons_embedded = custom_target(
    'all-icons-embedded',
    command: [cat, '@INPUT@'],
    capture: true,
    input: embedded_icons,
    output: 'all-icons-embedded.hpp',
)

sources += all_icons_embedded
