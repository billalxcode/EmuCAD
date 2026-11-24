# 59 - Golden RGBA GUI

Dokumen ini menjelaskan golden image RGBA dan prosedur update untuk
`gui`.

## Status

Golden RGBA sudah tersedia dan berjalan headless di test Linux:

- Renderer: `crates/gui/src/golden.rs`, hanya dikompilasi untuk test.
- Ukuran: 320 x 240 piksel, skala piksel 1.0.
- Scene: resistor, LED, button, switch SPST, switch SPDT, BJT NPN, dan
  opamp berbasis SVG, label, dan teks tetap pada tema gelap.
- Tujuan scene: mengunci keselarasan pin model dengan terminal SVG setelah
  layout pin per simbol dan pemetaan viewBox diterapkan.
- Grid: dots, coarse tick setiap 10 unit, axes, dan ruler pada kamera
  yang sama dengan scene.
- Font: Source Sans 3 untuk label dan Source Code Pro untuk teks teknis.
- Golden scene: `crates/gui/tests/golden/scene-320x240.rgba`.
- Golden grid: `crates/gui/tests/golden/grid-320x240.rgba`.
- Golden toolbar: `crates/gui/tests/golden/toolbar-2400x120.rgba`.
- Toolbar scene: satu baris toolbar penuh dengan ikon, label, dan
  shortcut hint pada tooltip.
- Perbandingan: byte RGBA exact, tanpa toleransi.
- Diagnostik: piksel, channel, dan nilai byte pertama yang berbeda.
- Determinisme: test merender dua kali dan memastikan hasil identik.

Renderer memakai `egui::Context::tessellate` untuk menghasilkan triangle
mesh, lalu merasterisasi triangle dan font atlas ke buffer RGBA perangkat
lunak. Tidak ada GPU, jaringan, wall-clock, atau dependensi baru.

## Menjalankan Test

```bash
cargo test -p gui golden
```

Test yang berjalan:

- `golden_render_is_deterministic`
- `golden_scene_matches_snapshot`
- `golden_grid_matches_snapshot`
- `golden_toolbar_matches_snapshot`

## Prosedur Update

Jika perubahan visual disengaja:

1. Jalankan renderer untuk menulis semua golden baru:

   ```bash
   UPDATE_GOLDEN=1 cargo test -p gui golden
   ```

2. Periksa `scene-320x240.rgba`, `grid-320x240.rgba`, dan
   `toolbar-2400x120.rgba` di `crates/gui/tests/golden/`.
3. Konversi atau buka file sebagai RGBA sesuai ukurannya untuk review
   visual.
4. Pastikan hanya perubahan yang dimaksud yang terlihat.
5. Commit golden dan perubahan renderer dalam satu perubahan terpisah.
6. Jalankan `cargo test -p gui golden` tanpa env update.

## Batasan

- Golden hanya divalidasi di Linux.
- Perubahan versi `egui`, font default, atau tessellator dapat mengubah
  output dan memerlukan update golden.
- Renderer test memakai rasterizer internal, bukan backend GPU `eframe`.
  Golden menjaga regresi scene dan teks, bukan kesamaan piksel dengan GPU.
- Scene golden harus tetap deterministik: tanpa animasi, pointer, atau
  waktu.

## Kriteria Lulus

- Test deterministik.
- Tidak ada akses jaringan atau wall-clock.
- Golden dapat diregenerasi dari commit yang sama.
- Perbedaan byte dilaporkan dengan jumlah byte yang berbeda.
- Ukuran file harus sama dengan 320 x 240 x 4 byte.

## Referensi

- `docs/57-gui-egui.md`: desain renderer dan test visual.
- `crates/gui/src/golden.rs`: renderer dan test golden.
- `docs/07-verification.md`: kebijakan verifikasi umum.
