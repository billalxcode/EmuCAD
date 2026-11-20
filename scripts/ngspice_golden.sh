#!/usr/bin/env bash
# Compare EmuCAD RC transient output with ngspice.
# Run with --help for options. Exit codes: 0 pass or skip, 1 failure, 2 usage.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
ngspice_bin=""
docker_image=""
emucad_bin=""
require=0
rel_tol="0.02"
self_test=0
engine=""

usage() {
    cat <<'TEXT'
Compare EmuCAD RC transient output with ngspice.

Usage:
  scripts/ngspice_golden.sh [options]
  scripts/ngspice_golden.sh --self-test

Options:
  --self-test            Run parser/comparator self-test with fixtures.
  --require              Missing ngspice, Docker, or cargo is an error.
  --ngspice PATH         ngspice executable (default: ngspice from PATH).
  --docker-image NAME    Run ngspice in this Docker image.
  --emucad PATH          Prebuilt rc_analysis example (default: cargo run).
  --rel-tol VALUE        Relative tolerance (default: 0.02).
  -h, --help             Show this help.
TEXT
}

option_value() {
    if [[ $# -lt 2 || -z $2 ]]; then
        echo "error: $1 requires a value" >&2
        exit 2
    fi
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --self-test) self_test=1; shift ;;
            --require) require=1; shift ;;
            --ngspice) option_value "$@"; ngspice_bin=$2; shift 2 ;;
            --docker-image) option_value "$@"; docker_image=$2; shift 2 ;;
            --emucad) option_value "$@"; emucad_bin=$2; shift 2 ;;
            --rel-tol) option_value "$@"; rel_tol=$2; shift 2 ;;
            -h|--help) usage; exit 0 ;;
            *) echo "error: unknown argument: $1" >&2; usage >&2; exit 2 ;;
        esac
    done
}

skip_or_fail() {
    local reason
    reason=$1
    if [[ $require -eq 1 ]]; then
        echo "FAIL: $reason" >&2
        exit 1
    fi
    echo "SKIP: $reason"
    echo "SKIP: pass --require to treat missing tools as failure"
    exit 0
}

resolve_local_ngspice() {
    local candidate
    candidate=${ngspice_bin:-ngspice}
    if ! command -v "$candidate" >/dev/null 2>&1; then
        if [[ -n $ngspice_bin ]]; then
            echo "FAIL: ngspice is not executable: $ngspice_bin" >&2
            exit 1
        fi
        skip_or_fail "ngspice is not installed and no --docker-image was given"
    fi
    ngspice_bin=$(command -v "$candidate")
    engine=local
}

resolve_engine() {
    if [[ -n $docker_image ]]; then
        if ! command -v docker >/dev/null 2>&1; then
            skip_or_fail "Docker is not available for --docker-image $docker_image"
        fi
        if ! docker image inspect "$docker_image" >/dev/null 2>&1; then
            skip_or_fail "Docker image is not available (is the daemon running?): $docker_image"
        fi
        engine=docker
        return
    fi
    resolve_local_ngspice
}

check_emucad_binary() {
    if [[ -n $emucad_bin ]]; then
        if [[ ! -x $emucad_bin ]]; then
            echo "FAIL: EmuCAD binary is not executable: $emucad_bin" >&2
            exit 1
        fi
        return
    fi
    if ! command -v cargo >/dev/null 2>&1; then
        skip_or_fail "cargo is not available to build the rc_analysis example"
    fi
}

# rc_analysis uses dt=1us for 100 steps, so the comparison point is 100us.
write_deck() {
    cat > "$tmp/rc.cir" <<'NET'
RC step response
V1 in 0 PULSE(0 5 0 1n 1n 1 2)
R1 in out 1k
C1 out 0 1u
.tran 1u 100u
.meas tran vout FIND v(out) AT=100u
.end
NET
}

run_ngspice() {
    if [[ $engine == docker ]]; then
        docker run --rm -v "$tmp:/data" "$docker_image" \
            ngspice -b -o /data/ngspice.log /data/rc.cir >/dev/null
        return
    fi
    "$ngspice_bin" -b -o "$tmp/ngspice.log" "$tmp/rc.cir" >/dev/null
}

run_emucad() {
    if [[ -n $emucad_bin ]]; then
        "$emucad_bin" > "$tmp/emucad.txt"
        return
    fi
    (cd "$root" && cargo run --quiet -p analysis --example rc_analysis) \
        > "$tmp/emucad.txt"
}

compare_outputs() {
    python3 "$root/scripts/ngspice_check.py" \
        --ngspice-log "$tmp/ngspice.log" \
        --emucad-output "$tmp/emucad.txt" \
        --time-ps 100000000 \
        --rel-tol "$rel_tol"
}

run_golden() {
    resolve_engine
    check_emucad_binary
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    write_deck
    run_ngspice
    run_emucad
    compare_outputs
}

main() {
    parse_args "$@"
    if [[ $self_test -eq 1 ]]; then
        python3 "$root/scripts/test-ngspice-check.py"
        return
    fi
    run_golden
}

main "$@"
