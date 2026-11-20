#!/usr/bin/env python3
"""Self-test for scripts/ngspice_check.py using checked-in fixtures."""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CHECKER = ROOT / "ngspice_check.py"
NGSPICE_FIXTURE = ROOT / "fixtures" / "ngspice_rc_synthetic.log"
EMUCAD_FIXTURE = ROOT / "fixtures" / "emucad_rc.txt"


def run_checker(
    ngspice_log: Path, emucad_output: Path, *extra: str
) -> subprocess.CompletedProcess:
    command = [
        sys.executable,
        str(CHECKER),
        "--ngspice-log",
        str(ngspice_log),
        "--emucad-output",
        str(emucad_output),
        *extra,
    ]
    return subprocess.run(
        command, capture_output=True, text=True, encoding="utf-8"
    )


class CheckerTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)

    def write_variant(
        self, name: str, source: Path, old: str, new: str
    ) -> Path:
        text = source.read_text(encoding="utf-8")
        if old not in text:
            self.fail(f"fixture {source} does not contain {old!r}")
        target = Path(self.temp.name) / name
        target.write_text(text.replace(old, new), encoding="ascii")
        return target

    def test_matching_fixtures_pass(self) -> None:
        result = run_checker(NGSPICE_FIXTURE, EMUCAD_FIXTURE)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("PASS", result.stdout)
        self.assertIn("emucad=", result.stdout)
        self.assertIn("ngspice=", result.stdout)

    def test_mismatched_measurement_fails(self) -> None:
        log = self.write_variant(
            "mismatch.log", NGSPICE_FIXTURE, "4.758129e-01", "5.500000e-01"
        )
        result = run_checker(log, EMUCAD_FIXTURE)
        self.assertEqual(result.returncode, 1)
        self.assertIn("FAIL", result.stderr)

    def test_missing_measurement_fails(self) -> None:
        log = self.write_variant(
            "missing.log", NGSPICE_FIXTURE, "vout = 4.758129e-01\n", ""
        )
        result = run_checker(log, EMUCAD_FIXTURE)
        self.assertEqual(result.returncode, 1)
        self.assertIn("found 0", result.stderr)

    def test_duplicate_measurement_fails(self) -> None:
        text = NGSPICE_FIXTURE.read_text(encoding="utf-8")
        log = Path(self.temp.name) / "duplicate.log"
        log.write_text(text + "vout = 4.758129e-01\n", encoding="ascii")
        result = run_checker(log, EMUCAD_FIXTURE)
        self.assertEqual(result.returncode, 1)
        self.assertIn("found 2", result.stderr)

    def test_time_mismatch_fails(self) -> None:
        output = self.write_variant(
            "time.txt", EMUCAD_FIXTURE, "t=100000000", "t=1000"
        )
        result = run_checker(NGSPICE_FIXTURE, output)
        self.assertEqual(result.returncode, 1)
        self.assertIn("expected 100000000", result.stderr)

    def test_tight_tolerance_fails(self) -> None:
        result = run_checker(
            NGSPICE_FIXTURE,
            EMUCAD_FIXTURE,
            "--rel-tol",
            "0.0001",
            "--abs-tol",
            "0",
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("FAIL", result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
