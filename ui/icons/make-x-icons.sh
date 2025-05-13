#!/usr/bin/env bash

set -euo pipefail

for filepath in $*; do
    filename=$(basename "$filepath")
    icon="${filename%.*}"
    echo "X($icon)"
done
