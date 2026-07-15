external:
    bindgen \
        src/external.h \
        \
        --allowlist-function "__liburing_.*" \
        --opaque-type "io_uring_sq" \
        --opaque-type "io_uring_cq" \
        --opaque-type ".*bindgen.*" \
        --opaque-type "sockaddr" \
        --default-macro-constant-type signed \
        \
        --allowlist-function "_exit" \
        --allowlist-function "calloc" \
        --allowlist-function "close" \
        --allowlist-function "dup2" \
        --allowlist-function "execvp" \
        --allowlist-function "exit" \
        --allowlist-function "fork" \
        --allowlist-function "free" \
        --allowlist-function "getenv" \
        --allowlist-function "localtime_r" \
        --allowlist-function "malloc" \
        --allowlist-function "open" \
        --allowlist-function "read" \
        --allowlist-function "strftime" \
        --allowlist-function "strerror" \
        --allowlist-function "time" \
        --allowlist-function "timerfd_create" \
        --allowlist-function "timerfd_settime" \
        --allowlist-function "write" \
        --allowlist-type "in_addr" \
        --allowlist-type "itimerspec" \
        --allowlist-type "sa_family_t" \
        --allowlist-type "sockaddr" \
        --allowlist-type "sockaddr_un" \
        --allowlist-type "socklen_t" \
        --allowlist-type "__socket_type" \
        --allowlist-type "time_t" \
        --allowlist-type "timespec" \
        --allowlist-type "tm" \
        --allowlist-var "AF_UNIX" \
        --allowlist-var "AT_FDCWD" \
        --allowlist-var "CLOCK_MONOTONIC" \
        --allowlist-var "ETIME" \
        --allowlist-var "O_RDONLY" \
        --allowlist-var "O_WRONLY" \
        --allowlist-var "STDERR_FILENO" \
        --allowlist-var "STDOUT_FILENO" \
        \
        --use-core \
        -o src/external.rs
    sed -i 's/pub /pub(crate) /g' src/external.rs

setup build:
    meson setup builddir --buildtype={{build}}

compile:
    meson compile -C builddir

dev:
    @just compile
    RUST_BACKTRACE=1 ASAN_OPTIONS=detect_leaks=1 LSAN_OPTIONS=suppressions=lsan.supp ./builddir/ui/layer-shell

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
    cargo build --example just-io
    strace target/debug/examples/just-io

io log="info":
    RUST_BACKTRACE=1 RUST_LOG={{log}} cargo run --example just-io --features debug-backtrace

test-install:
    @just clean
    rm -rf test-install
    meson setup builddir --buildtype=release --prefix=$PWD/test-install/usr
    meson compile -C builddir
    meson install -C builddir
    tree -C test-install

format:
    clang-format -i ui/*.cpp
    clang-format -i ui/*.hpp
    cargo fmt
