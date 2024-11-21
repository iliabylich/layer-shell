#!/usr/bin/env bash

OUTFILE=ui/src/widgets/icons.rs

truncate -s 0 $OUTFILE

write() {
    echo "$1" >> $OUTFILE
}

write "use gtk4::{gdk::Texture, glib::Bytes};"
write ""

for filename in icons/*.svg; do
    icon="$(basename "$filename" .svg)"
    write "pub(crate) fn ${icon}_icon() -> Texture {"
    write "    const BYTES: &[u8] = include_bytes!(\"../../../icons/$icon.svg\");"
    write "    let bytes = Bytes::from_static(BYTES);"
    write "    Texture::from_bytes(&bytes).unwrap()"
    write "}"
done
