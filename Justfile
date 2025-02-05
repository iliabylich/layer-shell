dbus-generate:
    ./dbus/generate.sh

bindgen:
    cbindgen --output bindings.hpp

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
    CXX=clang++ meson setup builddir --buildtype={{build}}
setup-debug:
    @just setup debug
setup-release:
    @just setup release

install destdir:
    meson install -C builddir --destdir={{destdir}}

dev:
    ninja -C builddir
    ./builddir/layer-shell
