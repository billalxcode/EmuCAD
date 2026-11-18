#!/usr/bin/env python3
"""Package EmuCAD release archives and SHA512 checksums."""

import argparse
import hashlib
import re
import shutil
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", help="release version; defaults to workspace version")
    parser.add_argument("--target", help="target triple; defaults to the host")
    parser.add_argument("--commit", help="commit SHA written into build-info.txt")
    parser.add_argument("--binary-dir", default="target/release", help="directory with built binaries")
    parser.add_argument("--output-dir", default="dist", help="directory for archives and checksums")
    parser.add_argument("--print-version", action="store_true", help="print the workspace version")
    parser.add_argument("--checksums", metavar="DIR", help="write SHA512 checksums for DIR")
    return parser.parse_args()


def workspace_version() -> str:
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    if "[workspace.package]" not in text:
        raise SystemExit("workspace package section not found")
    section = text.split("[workspace.package]", 1)[1]
    match = re.search(r'^version\s*=\s*"([^"]+)"', section, re.MULTILINE)
    if match is None:
        raise SystemExit("workspace version not found")
    return match.group(1)


def host_target() -> str:
    output = subprocess.check_output(["rustc", "-vV"], text=True)
    for line in output.splitlines():
        if line.startswith("host: "):
            return line[6:].strip()
    raise SystemExit("host target not found")


def git_commit() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def safe_name(value: str, label: str) -> str:
    if re.fullmatch(r"[0-9A-Za-z._-]+", value) is None:
        raise SystemExit(f"invalid {label}: {value}")
    return value


def binary_name(name: str, target: str) -> str:
    return f"{name}.exe" if "windows" in target else name


def copy_binaries(stage: Path, binary_dir: Path, target: str) -> None:
    for name in ("emucad", "emucad-gui"):
        source = binary_dir / binary_name(name, target)
        if not source.is_file():
            raise SystemExit(f"missing binary: {source}")
        shutil.copy2(source, stage / source.name)


def copy_directory(source: Path, destination: Path) -> None:
    if source.is_dir():
        shutil.copytree(source, destination, dirs_exist_ok=True)


def copy_common(stage: Path) -> None:
    for name in ("README.md", "LICENSE"):
        shutil.copy2(ROOT / name, stage / name)
    copy_directory(ROOT / "docs", stage / "docs")
    copy_directory(ROOT / "examples", stage / "examples")


LICENSE_FILES = [
    ("crates/gui/assets/symbols/NOTICE.md", "gui-symbols.md"),
    ("crates/gui/assets/fonts/NOTICE.md", "gui-fonts.md"),
    ("crates/gui/assets/fonts/OFL-SourceSans.txt", "OFL-SourceSans.txt"),
    ("crates/gui/assets/fonts/OFL-SourceCodePro.txt", "OFL-SourceCodePro.txt"),
]


def copy_licenses(stage: Path) -> None:
    licenses = stage / "share" / "licenses"
    licenses.mkdir(parents=True, exist_ok=True)
    for source, name in LICENSE_FILES:
        shutil.copy2(ROOT / source, licenses / name)


def copy_desktop(stage: Path) -> None:
    applications = stage / "share" / "applications"
    applications.mkdir(parents=True, exist_ok=True)
    shutil.copy2(
        ROOT / "packaging/emucad-gui.desktop",
        applications / "emucad-gui.desktop",
    )


def copy_packaging(stage: Path, target: str) -> None:
    copy_licenses(stage)
    if "linux" in target:
        copy_desktop(stage)


def write_build_info(stage: Path, version: str, target: str, commit: str) -> None:
    text = f"version={version}\ntarget={target}\ncommit={commit}\n"
    (stage / "build-info.txt").write_text(text, encoding="ascii")


def prepare_stage(
    stage: Path, version: str, target: str, commit: str, binary_dir: Path
) -> None:
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True)
    copy_binaries(stage, binary_dir, target)
    copy_common(stage)
    copy_packaging(stage, target)
    write_build_info(stage, version, target, commit)


def add_tar_tree(tar: tarfile.TarFile, directory: Path, prefix: str) -> None:
    for path in sorted(directory.rglob("*")):
        arcname = f"{prefix}/{path.relative_to(directory).as_posix()}"
        info = tar.gettarinfo(str(path), arcname)
        info.mtime = 0
        info.uid = 0
        info.gid = 0
        info.uname = ""
        info.gname = ""
        if path.is_file():
            with path.open("rb") as handle:
                tar.addfile(info, handle)
        else:
            tar.addfile(info)


def create_tar(stage: Path, archive: Path) -> None:
    with tarfile.open(archive, "w:gz") as tar:
        add_tar_tree(tar, stage, stage.name)


def create_zip(stage: Path, archive: Path) -> None:
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as bundle:
        for path in sorted(stage.rglob("*")):
            if path.is_dir():
                continue
            arcname = f"{stage.name}/{path.relative_to(stage).as_posix()}"
            info = zipfile.ZipInfo(arcname, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            bundle.writestr(info, path.read_bytes())


def create_archive(stage: Path, target: str, output_dir: Path) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    if "windows" in target:
        archive = output_dir / f"{stage.name}.zip"
        create_zip(stage, archive)
    else:
        archive = output_dir / f"{stage.name}.tar.gz"
        create_tar(stage, archive)
    return archive


def sha512(path: Path) -> str:
    digest = hashlib.sha512()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_checksums(directory: Path) -> None:
    archives = sorted(
        path
        for path in directory.iterdir()
        if path.is_file()
        and path.name.startswith("emucad-")
        and (path.name.endswith(".tar.gz") or path.name.endswith(".zip"))
    )
    lines = []
    for archive in archives:
        digest = sha512(archive)
        line = f"{digest}  {archive.name}"
        archive.with_name(f"{archive.name}.sha512").write_text(
            line + "\n", encoding="ascii"
        )
        lines.append(line)
    (directory / "SHA512SUMS").write_text("\n".join(lines) + "\n", encoding="ascii")


def build_release(args: argparse.Namespace) -> Path:
    version = safe_name(args.version or workspace_version(), "version")
    target = safe_name(args.target or host_target(), "target")
    commit = args.commit or git_commit()
    binary_dir = Path(args.binary_dir)
    if not binary_dir.is_absolute():
        binary_dir = ROOT / binary_dir
    output_dir = Path(args.output_dir)
    if not output_dir.is_absolute():
        output_dir = ROOT / output_dir
    stage = output_dir / f"emucad-v{version}-{target}"
    prepare_stage(stage, version, target, commit, binary_dir)
    archive = create_archive(stage, target, output_dir)
    write_checksums(output_dir)
    shutil.rmtree(stage)
    return archive


def main() -> int:
    args = parse_args()
    if args.print_version:
        print(workspace_version())
        return 0
    if args.checksums:
        write_checksums(Path(args.checksums))
        return 0
    print(build_release(args))
    return 0


if __name__ == "__main__":
    sys.exit(main())
