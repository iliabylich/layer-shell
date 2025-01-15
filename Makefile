BUILD_TYPE ?= debug
CC = clang

RUST_TARGET_DIR = target/$(BUILD_TYPE)
SO = $(RUST_TARGET_DIR)/liblayer_shell_io.so

CFLAGS = -std=c23 -O3 -g -Wall -Wextra -Werror

CFLAGS += `pkg-config --cflags gtk4`
CFLAGS += `pkg-config --cflags gtk4-layer-shell-0`
CFLAGS += `pkg-config --cflags vte-2.91-gtk4`

LDFLAGS += `pkg-config --libs gtk4`
LDFLAGS += `pkg-config --libs gtk4-layer-shell-0`
LDFLAGS += `pkg-config --libs vte-2.91-gtk4`

LDFLAGS += -L$(RUST_TARGET_DIR) -llayer_shell_io
LDFLAGS += -Wl,-rpath='$$ORIGIN/$(RUST_TARGET_DIR)'

run: main
	./main start

SRC=$(wildcard *.c)
HDRS=$(wildcard *.h)
OBJS = utils/css.o \
		utils/icons.o \
		utils/weather-helper.o \
		utils/window.o \
		windows/weather.o \
		windows/session.o \
		windows/network.o \
		windows/launcher.o \
		windows/htop.o \
		windows/top-bar.o \
		widgets/workspaces.o \
		widgets/htop.o \
		widgets/weather.o \
		widgets/language.o \
		widgets/sound.o \
		widgets/cpu.o \
		widgets/network.o \
		widgets/memory.o \
		widgets/time.o \
		widgets/session.o

%.o: %.c %.h bindings.h
	$(CC) -c $(CFLAGS) $< -o $@

main: main.c $(OBJS) $(SO) bindings.h
	$(CC) main.c $(OBJS) $(CFLAGS) $(LDFLAGS) -o main

target/debug/liblayer_shell_io.so:
	cargo build

target/release/liblayer_shell_io.so:
	cargo build --release

bindings.h:
	cbindgen --output bindings.h

clean:
	rm -f bindings.h $(SO) main *.o

compile_commands.json:
	# pipx install compiledb
	compiledb make main
