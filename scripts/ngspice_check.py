#!/usr/bin/env python3
"""Compare one ngspice measurement against an analytic value."""

import re
import sys


def main():
    if len(sys.argv) != 4:
        raise SystemExit("usage: ngspice_check.py LOG EXPECTED TOLERANCE")
    text = open(sys.argv[1], encoding="utf-8").read()
    match = re.search(r"vout\s*=\s*([-+0-9.eE]+)", text)
    if not match:
        raise SystemExit("vout measurement not found")
    value = float(match.group(1))
    expected = float(sys.argv[2])
    tolerance = float(sys.argv[3])
    if abs(value - expected) > tolerance:
        raise SystemExit(
            f"ngspice vout={value:.6f} expected={expected:.6f} tolerance={tolerance}"
        )
    print(
        f"ngspice vout={value:.6f} expected={expected:.6f} tolerance={tolerance}"
    )


if __name__ == "__main__":
    main()
