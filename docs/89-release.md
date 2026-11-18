# 89 - Rilis Biner GitHub

Dokumen ini mencatat workflow rilis biner di
`.github/workflows/release.yml` dan script `scripts/release.py`.

## 1. Versi

Versi diambil dari `[workspace.package]` di `Cargo.toml`:

```bash
python3 scripts/release.py --print-version
```

Tag rilis adalah `v<versi>`, misalnya `v0.1.0`. Jika versi workspace
naik menjadi `0.2.0`, workflow membuat tag dan release baru `v0.2.0`.

## 2. Trigger

Workflow berjalan setelah workflow `ci` selesai dengan status sukses
pada branch `main`:

```yaml
on:
  workflow_run:
    workflows: ["ci"]
    types: [completed]
    branches: [main]
  workflow_dispatch:
```

`workflow_dispatch` dipakai untuk rilis manual. Release tidak berjalan
jika CI gagal. Commit yang dipakai adalah `head_sha` dari CI run,
bukan sekadar HEAD branch saat workflow rilis berjalan.

## 3. Target

| Runner | Target triple | Format |
|---|---|---|
| `ubuntu-latest` | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| `macos-latest` | `aarch64-apple-darwin` | `.tar.gz` |
| `windows-latest` | `x86_64-pc-windows-msvc` | `.zip` |

Setiap target membangun `emucad` dan `emucad-gui` dengan
`cargo build --release -p cli -p gui`.

## 4. Isi Arsip

- `emucad` dan `emucad-gui` (atau `.exe` di Windows).
- `README.md` dan `LICENSE`.
- `docs/` dan `examples/`.
- `share/licenses/` untuk lisensi font dan simbol GUI.
- `share/applications/emucad-gui.desktop` di Linux.
- `build-info.txt` berisi versi, target, dan commit.

Nama arsip:

```
emucad-v<versi>-<target>.tar.gz
emucad-v<versi>-<target>.zip
```

## 5. SHA512

Setiap arsip memiliki file `<arsip>.sha512`. Workflow juga membuat
satu file gabungan:

```
SHA512SUMS
```

Checksum berubah setiap merge karena `build-info.txt` memuat commit
SHA. Verifikasi:

```bash
sha512sum -c SHA512SUMS
```

## 6. Model Release

Release memakai tag versi sebagai anchor. Setiap merge ke `main`
memperbarui aset pada release yang sama dengan `gh release upload
--clobber`. Commit terbaru terlihat di `build-info.txt` dan release
notes.

Konsekuensi:

- Aset di release `v0.1.0` selalu build terbaru untuk versi itu.
- Build lama tidak disimpan sebagai aset terpisah.
- Untuk menyimpan build lama, unduh arsip dan checksum sebelum merge
  berikutnya.
- Tag tidak dipindahkan; yang berubah adalah aset dan notes.

## 7. Izin

Workflow memakai `GITHUB_TOKEN` dengan `contents: write` untuk
membuat release dan mengunggah aset. Tidak ada secret tambahan.

## 8. Pemeriksaan Lokal

```bash
python3 scripts/release.py --print-version
python3 scripts/release.py --checksums dist
python3 -m py_compile scripts/release.py
```

Untuk uji packaging tanpa build release:

```bash
cargo build -p cli -p gui
python3 scripts/release.py \
  --version 0.1.0 \
  --target x86_64-unknown-linux-gnu \
  --commit local \
  --binary-dir target/debug \
  --output-dir /tmp/emucad-release
```

## 9. Batas

- Belum ada code signing atau notarization macOS.
- Belum ada installer Windows atau package Linux native.
- Target macOS masih arm64; belum ada build x86_64 terpisah.
- Target Linux masih x86_64; belum ada aarch64.
- Checksum menjamin integritas unduhan, bukan reproduksibilitas bit
  identik dari source yang sama.
- Rilis tidak menyertakan modul Python `python`.
