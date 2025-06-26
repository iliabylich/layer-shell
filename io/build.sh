#!/usr/bin/env bash

set -euo pipefail

buildtype="$1"
source_root="$2"
output="$3"

case "$buildtype" in
    "release")
        cargo build --release --manifest-path "$source_root/Cargo.toml"
        cp "$source_root/target/release/liblayer_shell_io.a" "$output"
        ;;
    "debug")
        cargo build --manifest-path "$source_root/Cargo.toml"
        cp "$source_root/target/debug/liblayer_shell_io.a" "$output"
        ;;
    *)
        echo "Usage: $0 [debug|release] <source-root> <output>"
        exit 1
        ;;
esac
