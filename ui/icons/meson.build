xxd = find_program('xxd')
xxd = generator(
    xxd,
    output: '@PLAINNAME@.xxd',
    arguments: ['-n', '@PLAINNAME@', '-i', '@INPUT@', '@OUTPUT@'],
)
cat = find_program('cat')

icons = files(
    'change_theme.png',
    'download_speed.png',
    'foggy.png',
    'partly_cloudy.png',
    'power.png',
    'question_mark.png',
    'rainy.png',
    'snowy.png',
    'sunny.png',
    'thunderstorm.png',
    'upload_speed.png',
    'wifi.png',
)

embedded_icons = xxd.process(icons)

all_icons_embedded = custom_target(
    'all-icons-embedded',
    command: [cat, '@INPUT@'],
    capture: true,
    input: embedded_icons,
    output: 'all-icons-embedded.h',
)
sources += all_icons_embedded

x_icons = custom_target(
    'x-icons',
    command: ['make-x-icons.sh', icons],
    capture: true,
    output: 'x-icons.h',
)
sources += x_icons
