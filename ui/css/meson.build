sassc = generator(
    find_program('sassc'),
    output: '@PLAINNAME@',
    arguments: ['@INPUT@', '@OUTPUT@'],
)
css = xxd.process(sassc.process('main.scss'))
sources += css
