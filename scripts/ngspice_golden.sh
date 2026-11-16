#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

cat > "$tmp/rc.cir" <<'NET'
RC step response
V1 in 0 PULSE(0 5 0 1n 1n 1 2)
R1 in out 1k
C1 out 0 1u
.tran 1u 1m
.meas tran vout FIND v(out) AT=1m
.end
NET

ngspice -b -o "$tmp/rc.log" "$tmp/rc.cir" >/dev/null
python3 "$root/scripts/ngspice_check.py" "$tmp/rc.log" 3.16060 0.05
