dbus-generate:
    ./dbus/generate.sh

liburing-generate:
    bindgen src/liburing-wrapper.h -o src/liburing/generated.rs

setup build:
    meson setup builddir --buildtype={{build}}

compile:
    meson compile -C builddir

dev:
    @just compile
    ASAN_OPTIONS=detect_leaks=1 LSAN_OPTIONS=suppressions=lsan.supp ./builddir/layer-shell

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

io log="info":
    RUST_LOG={{log}} cargo run --bin just-io

test-install:
    @just clean
    rm -rf test-install
    meson setup builddir --buildtype=release --prefix=$PWD/test-install/usr
    meson compile -C builddir
    meson install -C builddir
    tree -C test-install
