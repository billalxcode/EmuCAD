# 95 - UI Icons Tabler

Dokumen ini mencatat keputusan, batas, dan integrasi ikon UI.

## 1. Keputusan

- Ikon UI memakai **Tabler Icons** versi 3.52.0.
- Lisensi MIT.
- Hanya ikon yang dipakai yang di-vendor.
- Simbol skematik tetap memakai `crates/gui/assets/symbols/`.
- Tidak ada crate ikon baru. Asset SVG masuk ke crate `gui`.

Sumber:

- https://tabler.io/icons
- https://github.com/tabler/tabler-icons

## 2. Lisensi

File lisensi dan atribusi ada di:

`crates/gui/assets/icons/NOTICE.md`

Nama pemegang hak cipta ditulis sebagai `Pawel Kuna` agar tetap ASCII.
Itu transliterasi dari nama pada lisensi upstream.

## 3. Asset

- Lokasi: `crates/gui/assets/icons/`.
- Format: SVG Tabler 24 x 24, stroke 2 px.
- Jumlah saat ini: 77 ikon.
- Semua file harus ASCII.
- Versi dan sumber dicatat di `NOTICE.md`.

Ikon mencakup:

- Aksi file dan edit.
- Simulasi: run, pause, step, reset, DC, transient.
- View: zoom, fit, grid, map, search, settings.
- Elektronik: resistor, capacitor, diode, inductor, CPU.
- Data: database, chart, math, ruler, binary, file, folder, package.
- Utilitas: alert, check, close, bug, crosshair, sun, moon.
- Tool: pointer, route.
- Layout: palette, export, align kiri/kanan/atas/bawah/center, dan
  distribute horizontal/vertikal.
- Library: battery, transistor, shield, switch, bolt, temperature,
  engine, antenna, history, dan filter.
- Edit: rotate, flip horizontal/vertical, pencil, dan junction.
- Arah: arrow atas/bawah/kiri/kanan dan wave square.

## 4. Integrasi

- `crates/gui/src/icons/mod.rs` memuat `IconKind` dan peta `OnceLock`.
- Asset dibaca dengan `include_str!`.
- Parser memakai `crates/gui/src/symbols/svg.rs`.
- Gambar memakai `symbols::draw_svg` yang sama dengan simbol skematik.
- `IconButton` menggambar ikon di atas tombol teks dan mendukung state
  selected, label dinamis, serta tooltip shortcut.
- Tooltip menampilkan label dan shortcut bila tersedia.

Parser SVG sekarang mendukung perintah path `A` atau `a` untuk busur.
Tanpa dukungan busur, banyak ikon Tabler gagal diparse.

## 5. Cakupan Saat Ini

Sudah memakai ikon:

- Toolbar edit: undo dan redo.
- Toolbar file: open, save, export, dan menu recent per file.
- Toolbar simulasi: load HEX, check netlist, DC, transient.
- Toolbar run: run, pause, step, dan reset.
- Toolbar tool: select, place, wire, probe, dan zoom.
- Toolbar sheet: library, inspector, dan legend.
- Command palette: ikon per command dan hint shortcut.
- Library: kategori, search, clear, recent, filter, dan status model.
- Inspector: aksi komponen, wire class, warna, label, dan delete.
- Context menu: edit, transform, net, probe, chart, dan tool.
- MCU pin mode: input, output, ADC, PWM, UART TX, dan UART RX.
- Canvas view: zoom in/out, fit, grid, minimap, flow, unit, dan tema.
- Status bar: crosshair, ruler, zoom, grid, snap, CPU, diode, FPS,
  alert, dan check.
- Panel simulasi: run, pause, step, reset, dan pengaturan mixed.
- Mixed panel: settings, probe net, dan chart trace.
- Grafik AVR: run dan pause.

## 6. Batas

- Ikon Tabler bukan simbol skematik presisi.
- Ikon tidak menggantikan label teks untuk aksesibilitas.
- Asset tidak boleh diambil dari CDN saat runtime.
- Menambah crate ikon eksternal memerlukan keputusan baru.
- Setiap ikon baru harus ditambahkan ke `NOTICE.md` bila versi berubah.

## 7. Verifikasi

- `all_icons_parse` memastikan semua `IconKind` berhasil diparse.
- `draw_icon_paints_known_asset` memastikan jalur gambar tidak panic.
- Test parser path mencakup busur absolut dan flag kompak.
- ASCII gate mencakup file SVG dan `NOTICE.md`.

## 8. Rencana Lanjutan

Lihat `TODO-UI.md` untuk rollout lengkap dan pemetaan per surface.
