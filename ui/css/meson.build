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

sources += xxd.process(
    sassc.process('main.scss'),
)
