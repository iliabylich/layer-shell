external:
    bindgen \
        src/external.h \
        \
        --allowlist-function "__liburing_.*" \
        --opaque-type "io_uring_sq" \
        --opaque-type "io_uring_cq" \
        --opaque-type ".*bindgen.*" \
        --opaque-type "sockaddr" \
        \
        --allowlist-function "BIO_ctrl" \
        --allowlist-function "BIO_new" \
        --allowlist-function "BIO_read" \
        --allowlist-function "BIO_s_mem" \
        --allowlist-function "BIO_write" \
        --allowlist-function "SSL_CTX_free" \
        --allowlist-function "SSL_CTX_new" \
        --allowlist-function "SSL_CTX_set_default_verify_paths" \
        --allowlist-function "SSL_CTX_set_verify" \
        --allowlist-function "SSL_connect" \
        --allowlist-function "SSL_free" \
        --allowlist-function "SSL_get0_param" \
        --allowlist-function "SSL_get_error" \
        --allowlist-function "SSL_new" \
        --allowlist-function "SSL_read" \
        --allowlist-function "SSL_set_bio" \
        --allowlist-function "SSL_set_connect_state" \
        --allowlist-function "SSL_write" \
        --allowlist-function "TLS_client_method" \
        --allowlist-function "X509_VERIFY_PARAM_set1_host" \
        --allowlist-function "X509_VERIFY_PARAM_set_hostflags" \
        --allowlist-function "__openssl_.*" \
        --allowlist-var "SSL_ERROR_WANT_READ" \
        --allowlist-var "SSL_ERROR_WANT_WRITE" \
        --allowlist-var "SSL_ERROR_ZERO_RETURN" \
        --allowlist-var "SSL_VERIFY_PEER" \
        --allowlist-var "TLS1_2_VERSION" \
        --allowlist-var "BIO_CTRL_PENDING" \
        --opaque-type "BIO" \
        --opaque-type "BIO_METHOD" \
        --opaque-type "SSL" \
        --opaque-type "SSL_CTX" \
        --opaque-type "SSL_METHOD" \
        --opaque-type "X509_STORE_CTX" \
        --opaque-type "X509_VERIFY_PARAM" \
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
