BUILD_TYPE ?= debug

RUST_TARGET_DIR = target/$(BUILD_TYPE)
SO = $(RUST_TARGET_DIR)/liblayer_shell_io.so

CFLAGS = -O3 -g
CFLAGS += `pkg-config --cflags gtk4`
LDFLAGS += `pkg-config --libs gtk4` -L$(RUST_TARGET_DIR) -llayer_shell_io
CFLAGS += -Wl,-rpath='$$ORIGIN/$(RUST_TARGET_DIR)'

run: main
	./main

main: main.c $(SO) bindings.h
	$(CC) main.c $(CFLAGS) $(LDFLAGS) -o main

target/debug/liblayer_shell_io.so:
	cargo build

target/release/liblayer_shell_io.so:
	cargo build --release

bindings.h:
	cbindgen --output bindings.h

clean:
	rm -f bindings.h $(SO) main
