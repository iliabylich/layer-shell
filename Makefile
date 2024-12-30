BUILD_TYPE ?= debug

RUST_TARGET_DIR = target/$(BUILD_TYPE)
SO = $(RUST_TARGET_DIR)/liblayer_shell_io.so

CFLAGS = -O3 -g

CFLAGS += `pkg-config --cflags gtk4`
CFLAGS += `pkg-config --cflags gtk4-layer-shell-0`
CFLAGS += `pkg-config --cflags vte-2.91-gtk4`

LDFLAGS += `pkg-config --libs gtk4`
LDFLAGS += `pkg-config --libs gtk4-layer-shell-0`
LDFLAGS += `pkg-config --libs vte-2.91-gtk4`

LDFLAGS += -L$(RUST_TARGET_DIR) -llayer_shell_io
CFLAGS += -Wl,-rpath='$$ORIGIN/$(RUST_TARGET_DIR)'

run: main
	./main start

main: main.c css.h icons.h widgets.h $(SO) bindings.h
	$(CC) main.c $(CFLAGS) $(LDFLAGS) -o main

target/debug/liblayer_shell_io.so:
	cargo build

target/release/liblayer_shell_io.so:
	cargo build --release

bindings.h:
	cbindgen --output bindings.h

clean:
	rm -f bindings.h $(SO) main

compile_commands.json:
	# pipx install compiledb
	compiledb make main
