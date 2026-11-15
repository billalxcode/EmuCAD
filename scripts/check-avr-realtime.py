#!/usr/bin/env python3
"""Verify that the CLI AVR realtime mode stays within 1 percent of 16 MHz."""

import re
import statistics
import subprocess
import tempfile
import time
from pathlib import Path

CLOCK_HZ = 16_000_000
TOLERANCE = 0.01
STEPS = 10_000_000
RUNS = 5
ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target" / "release" / "emucad"
PROGRAMS = {
    "alu": [0xE001, 0xE012, 0x0F01, 0x1B01, 0x2F01, 0xCFFC],
    "mixed": [0xE505, 0xE01F, 0x2301, 0x2B01, 0x2701, 0x9506, 0x9505, 0x9507, 0x9502, 0x9503, 0x950A, 0x9F01, 0xCFF5],
    "memory": [0xE0A0, 0xE0B2, 0xE505, 0x930C, 0x911C, 0x0F01, 0xCFFC],
    "timer": [0xE001, 0xBD05, 0xE012, 0x0F01, 0x1B01, 0x2F01, 0xCFFC],
}


def hex_file(words: list[int]) -> str:
    data = bytearray()
    for word in words:
        data += bytes([word & 0xFF, word >> 8])
    record = bytes([len(data), 0, 0, 0]) + data
    checksum = (-sum(record)) & 0xFF
    return ":" + record.hex().upper() + f"{checksum:02X}\n:00000001FF\n"


def measure(path: Path) -> tuple[int, float]:
    command = [str(BINARY), "run", str(path), str(STEPS), "--realtime"]
    start = time.monotonic()
    output = subprocess.run(command, capture_output=True, text=True, check=True).stdout
    wall = time.monotonic() - start
    cycles = int(re.search(r"(\d+) cycles", output).group(1))
    return cycles, wall


def main() -> None:
    if not BINARY.exists():
        raise SystemExit(f"missing release binary: {BINARY}")
    failed = False
    with tempfile.TemporaryDirectory() as directory:
        for name, words in PROGRAMS.items():
            path = Path(directory) / f"{name}.hex"
            path.write_text(hex_file(words))
            errors = []
            for _ in range(RUNS):
                cycles, wall = measure(path)
                target = cycles / CLOCK_HZ
                errors.append((wall - target) / target)
            median = statistics.median(errors)
            worst = max(errors, key=abs)
            print(
                f"{name}: median={median * 100:+.3f}% worst={worst * 100:+.3f}%"
            )
            failed |= abs(median) > TOLERANCE
    if failed:
        raise SystemExit("realtime tolerance exceeded")


if __name__ == "__main__":
    main()
