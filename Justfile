dbus-generate:
    ./dbus/generate.sh

bindgen:
    cbindgen --output bindings.hpp

clean:
    rm -rf builddir

setup-dev:
    CXX=clang++ meson setup builddir --buildtype=debug

dev:
    cargo build
    ninja -C builddir
    ./builddir/layer-shell
