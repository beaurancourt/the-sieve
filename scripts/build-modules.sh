#!/usr/bin/env bash
# Rebuild every PDF under modules/ from its .md source.
#
# Usage:
#   scripts/build-modules.sh
#
# By default uses the release binary at target/release/the-sieve. Override
# the binary path with TS_BIN=… if you want to test a different build.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MODULES_DIR="$ROOT/modules"
TS_BIN="${TS_BIN:-$ROOT/target/release/the-sieve}"

if [ ! -x "$TS_BIN" ]; then
    echo "the-sieve binary not found at $TS_BIN" >&2
    echo "Build it first with: cargo build --release" >&2
    exit 1
fi

if [ ! -d "$MODULES_DIR" ]; then
    echo "No modules directory at $MODULES_DIR" >&2
    exit 1
fi

failures=()
count=0

while IFS= read -r -d '' md; do
    count=$((count + 1))
    rel="${md#$ROOT/}"
    printf '[%2d] %s ... ' "$count" "$rel"
    if "$TS_BIN" "$md" >/dev/null 2>&1; then
        echo "ok"
    else
        echo "FAILED"
        failures+=("$rel")
    fi
done < <(find "$MODULES_DIR" -type f -name '*.md' -print0)

echo
if [ "$count" -eq 0 ]; then
    echo "No .md files found under $MODULES_DIR"
    exit 0
fi

if [ "${#failures[@]}" -eq 0 ]; then
    echo "Built $count module(s) successfully."
else
    echo "Built $((count - ${#failures[@]})) of $count modules; ${#failures[@]} failed:"
    printf '  - %s\n' "${failures[@]}"
    exit 1
fi
