#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

SDIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd) && cd "$SDIR"

PDIR=$(cd "$SDIR/.." && pwd)

shells=$(cargo run -q -- completions --help |
    grep -Po '(?<=\[possible values: )(.*)(?=\])' |
    tr ',' '\n' | tr -d ' ')

for shell in $shells; do
    dir="$PDIR/completions/$shell"
    mkdir -p "$dir"
    cargo run -q -- completions "$shell" -d "$dir"
done
