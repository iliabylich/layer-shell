dbus-generate:
    ./dbus/generate.sh

clean:
    cargo clean
    rm -rf builddir

cargo-debug out:
    cargo build
    cp target/debug/liblayer_shell_io.so builddir/{{out}}
cargo-release out:
    cargo build --release
    cp target/release/liblayer_shell_io.so builddir/{{out}}

setup build:
    meson setup builddir --buildtype={{build}}

install destdir:
    meson install -C builddir --destdir={{destdir}}

dev:
    meson compile -C builddir
    ./builddir/layer-shell

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
