#!/usr/bin/env bash

set -eu

blueprint-compiler compile Widgets.blp > Widgets.ui

XML=$(cat Widgets.ui)

WIDGETS="$(
    echo "$XML" |
    grep -o -E "class=\"([a-zA-Z]+)\" id=\"([a-zA-Z0-9]+)\"" |
    sed -e "s/class=\"Gtk//" |
    sed -e "s/\" id=\"/ /" |
    sed -e "s/\"//"
)"

MACROS="$(echo "$WIDGETS" | awk '{printf("widget!(%s, gtk4::%s);\n", $2, $1)}')"
LOADS="$(echo "$WIDGETS" | awk '{printf("    load_%s(builder);\n", $2)}')"

OUTPUT="$(
    echo "use crate::widgets::widget;"
    echo ""
    echo "$MACROS"
    echo ""
    echo "pub(crate) fn load_widgets(builder: &gtk4::Builder) {"
    echo "$LOADS"
    echo "}"
)"

echo "$OUTPUT" > "ui/src/widgets/load.rs"
