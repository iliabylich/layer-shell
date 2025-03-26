sassc = generator(
    find_program('sassc'),
    output: '@PLAINNAME@',
    arguments: ['@INPUT@', '@OUTPUT@'],
)

sources += xxd.process(
    sassc.process('main.scss'),
)
