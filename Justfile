dbus-generate:
    ./dbus/generate.sh

css-generate:
    sassc main.scss main.css

lint:
    ruff format ui
    ruff check ui
    cargo clippy

dev:
    cargo build
    DEV=1 ./ui/main.py

build-release:
    @just css-generate
    cargo build --release
    python3 -m zipapp ui --python "/usr/bin/env python3" -m "main:main"

install destdir:
    install -Dm0755 ui.pyz {{destdir}}/bin/layer-shell
    install -Dm0644 target/release/liblayer_shell_io.so {{destdir}}/lib/x86_64-linux-gnu/liblayer_shell_io.so

perf-io:
    #!/usr/bin/env -S bash -x
    rm -f perf.data
    cargo build --bin just-io --release
    perf record -g --delay 2 target/release/just-io &
    perf_pid=$!
    echo "waiting for 5 seconds"
    sleep 5
    pkill just-io
    wait -n $perf_pid
    perf report

strace-io:
    cargo build --bin just-io
    strace target/debug/just-io
