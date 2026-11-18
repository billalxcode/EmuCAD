#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    fi
fi

cargo build --release -p cli
python3 scripts/check-avr-realtime.py
cargo test --release -p gui worker_realtime_pacing_matches_clock -- --ignored
