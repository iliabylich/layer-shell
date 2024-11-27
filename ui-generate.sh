#!/usr/bin/env bash

set -eu

OUTFILE=ui/src/widgets/gen.rs
truncate -s 0 $OUTFILE

blueprint-compiler compile Widgets.blp > Widgets.ui

NL="
"
SP=" "

parse_xml() {
    local xml="$(cat $1)"
    local -n out=$2

    local pairlist="$(
        echo "$xml" |
        grep -o -E "class=\"([a-zA-Z]+)\" id=\"([a-zA-Z0-9]+)\"" |
        sed -e "s/class=\"Gtk//" |
        sed -e "s/\" id=\"/ /" |
        sed -e "s/\"//"
    )"

    local ifs="$IFS"
    IFS="$NL"
    while read -r line; do
        out+=("$line")
    done <<<"$pairlist"
    IFS="$ifs"
}

parse_line() {
    local line="$1"
    local -n _name=$2
    local -n _type=$3
    local ifs="$IFS"
    IFS="$SP" read -a parts <<< "$line"
    _type="${parts[0]}"
    _name="${parts[1]}"
}

lines=()
parse_xml "Widgets.ui" lines

write() {
    echo "$1" >> $OUTFILE
}

write "// This file is auto-generated by $0"
write ""

for line in "${lines[@]}"; do
    parse_line "$line" name type
    NAME="${name^^}"

    write "static mut $NAME: Option<gtk4::$type> = None;"
    write "pub(crate) fn $name() -> &'static gtk4::$type {"
    write "    unsafe {"
    write "        match $NAME.as_ref() {"
    write "            Some(v) => v,"
    write "            None => {"
    write "                eprintln!(\"widget $name is not initialised\");"
    write "                std::process::exit(1);"
    write "            }"
    write "        }"
    write "    }"
    write "}"
done

write "pub(crate) unsafe fn init_widgets(builder: &gtk4::Builder) {"
for line in "${lines[@]}"; do
    parse_line "$line" name type
    NAME="${name^^}"

    write "    $NAME = builder.object(\"$name\");"
done
write "}"