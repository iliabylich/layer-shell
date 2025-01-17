dbus-generate:
    ./dbus/generate.sh

start:
    make clean && make -j`nproc` main && make run
