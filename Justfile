dbus-generate:
    ./dbus/generate.sh

setup build:
    meson setup builddir --buildtype={{build}}

dev:
    meson compile -C builddir
    ./builddir/layer-shell

clean:
    rm -rf builddir

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
