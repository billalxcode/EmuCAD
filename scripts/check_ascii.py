#!/usr/bin/env python3
"""Fail when tracked text files contain non-ASCII characters.

The file list comes from `git ls-files`, so untracked build output is
never scanned. Binary assets, fonts, golden images, and excluded build
directories are skipped. Run from the repository root:

    python3 scripts/check_ascii.py
    python3 scripts/check_ascii.py --list-skipped
"""

import subprocess
import sys
from pathlib import Path

TEXT_SUFFIXES = frozenset({
    ".bash", ".c", ".cc", ".cfg", ".conf", ".cpp", ".css", ".csv",
    ".desktop", ".dockerfile", ".h", ".hh", ".hpp", ".html", ".ini",
    ".js", ".json", ".lock", ".log", ".md", ".mk", ".py", ".rules",
    ".rs", ".s", ".sh", ".svg", ".toml", ".ts", ".txt", ".xml",
    ".yaml", ".yml",
})
TEXT_NAMES = frozenset({
    ".dockerignore", ".editorconfig", ".gitattributes", ".gitignore",
    "Dockerfile", "LICENSE", "Makefile",
})
EXCLUDED_DIRS = frozenset({
    ".caches", ".git", "build", "dist", "node_modules", "target",
})


def tracked_files() -> list[Path]:
    raw = subprocess.check_output(["git", "ls-files", "-z"])
    names = raw.decode("utf-8").split("\0")
    return [Path(name) for name in names if name]


def has_excluded_dir(path: Path) -> bool:
    return any(part in EXCLUDED_DIRS for part in path.parts)


def is_text_file(path: Path) -> bool:
    if path.name in TEXT_NAMES:
        return True
    return path.suffix.lower() in TEXT_SUFFIXES


def select_files(paths: list[Path]) -> tuple[list[Path], list[Path]]:
    scanned: list[Path] = []
    skipped: list[Path] = []
    for path in paths:
        if has_excluded_dir(path) or not is_text_file(path):
            skipped.append(path)
            continue
        scanned.append(path)
    return scanned, skipped


def check_line(path: Path, lineno: int, line: str) -> bool:
    bad = False
    for colno, char in enumerate(line, 1):
        if ord(char) <= 127:
            continue
        print(f"{path}:{lineno}:{colno}: U+{ord(char):04X} {char!r}")
        bad = True
    return bad


def check_file(path: Path) -> bool:
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        print(f"{path}: cannot read as UTF-8: {exc}", file=sys.stderr)
        return True
    bad = False
    for lineno, line in enumerate(text.splitlines(), 1):
        bad = check_line(path, lineno, line) or bad
    return bad


def parse_args(argv: list[str]) -> bool | None:
    unknown = [arg for arg in argv if arg != "--list-skipped"]
    if unknown:
        return None
    return "--list-skipped" in argv


def scan(paths: list[Path]) -> tuple[bool, list[Path], list[Path]]:
    scanned, skipped = select_files(paths)
    bad = False
    for path in scanned:
        bad = check_file(path) or bad
    return bad, scanned, skipped


def report_skipped(skipped: list[Path]) -> None:
    for path in skipped:
        print(f"skipped: {path}")


def main(argv: list[str]) -> int:
    list_skipped = parse_args(argv)
    if list_skipped is None:
        print("usage: check_ascii.py [--list-skipped]", file=sys.stderr)
        return 2
    try:
        paths = tracked_files()
    except (OSError, subprocess.CalledProcessError) as exc:
        print(f"cannot list tracked files: {exc}", file=sys.stderr)
        return 2
    bad, scanned, skipped = scan(paths)
    print(f"checked {len(scanned)} text files; skipped {len(skipped)} others")
    if list_skipped:
        report_skipped(skipped)
    if bad:
        print("non-ASCII characters found", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
