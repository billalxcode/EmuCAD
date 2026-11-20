#!/usr/bin/env python3
"""Compare an EmuCAD transient sample with an ngspice measurement.

The ngspice log must contain exactly one line such as:

    vout = 4.758129e-01

The EmuCAD output must contain exactly one line such as:

    transient t=100000000 ps vout=0.4756

Exit status: 0 match, 1 mismatch or parse error, 2 usage error.
"""

import argparse
import re
import sys
from pathlib import Path

NUMBER = r"[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?"
NGSPICE_LINE = re.compile(
    rf"^[ \t]*vout[ \t]*=[ \t]*({NUMBER})[ \t]*$", re.MULTILINE
)
EMUCAD_LINE = re.compile(
    rf"^transient t=(\d+) ps vout=({NUMBER})[ \t]*$", re.MULTILINE
)


class CompareError(Exception):
    """A parse or comparison failure."""


def positive_float(raw: str) -> float:
    value = float(raw)
    if value <= 0.0:
        raise argparse.ArgumentTypeError("must be positive")
    return value


def non_negative_float(raw: str) -> float:
    value = float(raw)
    if value < 0.0:
        raise argparse.ArgumentTypeError("must be non-negative")
    return value


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare an EmuCAD transient sample with ngspice."
    )
    parser.add_argument("--ngspice-log", required=True)
    parser.add_argument("--emucad-output", required=True)
    parser.add_argument("--time-ps", type=int, default=100_000_000)
    parser.add_argument("--rel-tol", type=positive_float, default=0.02)
    parser.add_argument("--abs-tol", type=non_negative_float, default=1e-6)
    return parser.parse_args(argv)


def read_text(path: str) -> str:
    try:
        return Path(path).read_text(encoding="utf-8")
    except OSError as error:
        raise CompareError(f"cannot read {path}: {error}") from error
    except UnicodeDecodeError as error:
        raise CompareError(f"{path} is not valid UTF-8: {error}") from error


def single_match(pattern, text: str, label: str):
    matches = pattern.findall(text)
    if len(matches) != 1:
        raise CompareError(f"expected one {label}, found {len(matches)}")
    return matches[0]


def parse_ngspice(text: str) -> float:
    raw = single_match(NGSPICE_LINE, text, "ngspice vout measurement")
    return float(raw)


def parse_emucad(text: str) -> tuple[int, float]:
    time_raw, value_raw = single_match(
        EMUCAD_LINE, text, "EmuCAD transient sample"
    )
    return int(time_raw), float(value_raw)


def check_time(actual: int, expected: int) -> None:
    if actual != expected:
        raise CompareError(
            f"EmuCAD sample is at {actual} ps, expected {expected} ps"
        )


def compare(
    emucad: float, ngspice: float, rel_tol: float, abs_tol: float
) -> tuple[float, float]:
    difference = abs(emucad - ngspice)
    allowed = abs_tol + rel_tol * abs(ngspice)
    relative = difference / abs(ngspice) if ngspice != 0.0 else difference
    if difference > allowed:
        raise CompareError(
            f"emucad={emucad:.6g} ngspice={ngspice:.6g} "
            f"diff={difference:.3g} allowed={allowed:.3g}"
        )
    return relative, allowed


def run(args: argparse.Namespace) -> str:
    ngspice = parse_ngspice(read_text(args.ngspice_log))
    time_ps, emucad = parse_emucad(read_text(args.emucad_output))
    check_time(time_ps, args.time_ps)
    relative, allowed = compare(emucad, ngspice, args.rel_tol, args.abs_tol)
    return (
        f"PASS emucad={emucad:.6g} ngspice={ngspice:.6g} "
        f"rel_diff={relative:.3g} allowed={allowed:.3g} "
        f"time={time_ps}ps"
    )


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        print(run(args))
    except CompareError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
