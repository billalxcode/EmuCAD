#!/usr/bin/env bash
# Build host binaries, then delegate packaging to the authoritative release script.
# Usage: ./scripts/package.sh [release.py options]
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

cargo build --release -p cli -p gui
exec python3 scripts/release.py "$@"
