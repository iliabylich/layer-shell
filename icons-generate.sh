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
    ICON="${icon^^}"
    write "static mut $ICON: Option<Texture> = None;"
    write "pub(crate) fn ${icon}_icon() -> &'static Texture {"
    write "    unsafe {"
    write "        match $ICON.as_ref() {"
    write "            Some(v) => v,"
    write "            None => {"
    write "                eprintln!(\"icon $icon is not initialised\");"
    write "                std::process::exit(1);"
    write "            }"
    write "        }"
    write "    }"
    write "}"
done

write "pub(crate) unsafe fn init_icons() {"
for filename in icons/*.svg; do
    icon="$(basename "$filename" .svg)"
    ICON="${icon^^}"
    write "    const ${ICON}_BYTES: &[u8] = include_bytes!(\"../../../icons/$icon.svg\");"
    write "    let ${icon}_bytes = Bytes::from_static(${ICON}_BYTES);"
    write "    let ${icon}_texture = Texture::from_bytes(&${icon}_bytes).unwrap();"
    write "    $ICON = Some(${icon}_texture);"
    write ""
done
write "    ()"
write "}"
