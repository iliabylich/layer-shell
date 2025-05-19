xxd = find_program('xxd')
xxd = generator(
    xxd,
    output: '@PLAINNAME@.xxd',
    arguments: ['-n', '@PLAINNAME@', '-i', '@INPUT@', '@OUTPUT@'],
)
cat = find_program('cat')

icons = files(
    'question_mark.png',
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
