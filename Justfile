dbus-generate:
    ./dbus/generate.sh

bindgen:
    cbindgen --output bindings.h

dev:
    rm -rf builddir
    CC=clang meson setup builddir --buildtype=debug
    ninja -C builddir
    ./builddir/layer-shell

release:
    CC=clang meson setup builddir --buildtype=release
    ninja -C builddir
