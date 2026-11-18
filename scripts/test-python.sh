#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    fi
fi

cargo build -p python

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
ln -s "$PWD/target/debug/libemucad.so" "$tmp_dir/emucad.so"
PYTHONPATH="$tmp_dir" python3 crates/python/tests/python_smoke.py
PYTHONPATH="$tmp_dir" python3 examples/python/build_netlists.py
