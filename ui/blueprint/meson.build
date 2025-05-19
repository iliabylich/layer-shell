blueprint = find_program('blueprint-compiler')
blueprint = generator(
    blueprint,
    output: '@BASENAME@.xml',
    arguments: ['compile', '--output', '@OUTPUT@', '@INPUT@'],
)

xxd = find_program('xxd')
xxd = generator(
    xxd,
    output: '@PLAINNAME@.xxd',
    arguments: ['-n', '@PLAINNAME@', '-i', '@INPUT@', '@OUTPUT@'],
)

inputs = files(
    'htop.blp',
    'ping.blp',
    'session.blp',
    'top_bar.blp',
    'weather.blp',
)

sources += xxd.process(
    blueprint.process(inputs),
)
