dbus-generate:
    ./dbus/generate.sh

bindgen:
    cbindgen --output bindings.h

clean:
    rm -rf builddir

setup-dev:
    CC=clang meson setup builddir --buildtype=debug

dev:
    cargo build
    ninja -C builddir
    ./builddir/layer-shell
