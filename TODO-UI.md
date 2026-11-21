# TODO - UI Tabler dan Mixed Worker

Backlog UI setelah migrasi ikon Tabler dan worker mixed.

**Status:** U0 selesai. U1 sampai U7 belum.

**Sumber kebenaran:**

- [`docs/57-gui-egui.md`](docs/57-gui-egui.md) untuk desain GUI.
- [`docs/95-ui-icons.md`](docs/95-ui-icons.md) untuk kebijakan ikon.
- [`docs/90-mixed-signal.md`](docs/90-mixed-signal.md) untuk mixed signal.
- [`TODO-GUI.md`](TODO-GUI.md) untuk backlog GUI lama.
- [`TODO-SIMULATION.md`](TODO-SIMULATION.md) untuk backlog simulasi.

File ini tidak menggantikan backlog lama. Item yang sudah ada di
`TODO-GUI.md` atau `TODO-SIMULATION.md` dirujuk, bukan diduplikasi.

---

## 1. Peta Surface UI

| Surface | Lokasi utama | Ikon saat ini | Target |
|---|---|---|---|
| Toolbar edit | `app/toolbar.rs` | Undo, Redo | Selesai |
| Toolbar file | `app/toolbar.rs` | Open, Save, PNG | Recent dan export |
| Toolbar simulasi | `app/toolbar.rs` | HEX, netlist, DC, transient | Mixed dan probe |
| Toolbar run | `app/toolbar.rs` | Run, Pause, Step, Reset | Selesai |
| Toolbar tool | `app/toolbar.rs` | Teks | Select, wire, probe |
| Toolbar sheet | `app/toolbar.rs` | Teks | Pin, sheet, export |
| Library header | `app/library_ui.rs` | Teks dan search | Search, recent, filter |
| Library kategori | `app/library_ui.rs` | Teks | Ikon per kategori |
| Library row | `app/library_ui.rs` | Simbol dan teks | Kategori dan status |
| Inspector komponen | `app/scene_view.rs` | Teks | Rotate, mirror, delete |
| Inspector wire | `app/inspector.rs` | Teks | Warna, label, delete |
| Inspector MCU | `app/scene_view.rs` | Combo mode pin | Status pin dan mixed |
| Panel simulasi | `app/windows/panels.rs` | Run, pause, step, reset | Domain dan probe |
| Grafik AVR | `app/graph_ui/` | Run, pause | Probe dan sample |
| Grafik transient | `app/graph_ui/` | Teks | Probe dan cursor |
| Grafik mixed | `app/windows/panels.rs` | Grafik pin output | Net, ADC, PWM |
| Canvas | `app/view.rs`, `scene/` | Teks | Grid, zoom, fit, probe |
| Context menu | `app/context_ui.rs` | Teks | Aksi komponen dan net |
| Command palette | `app/palette_ui.rs` | Teks | Ikon per command |
| Status bar | `app/status.rs` | Teks | Status dan ikon |

---

## 2. Peta Ikon

Sudah di-vendor dan dipakai:

- `arrow-back-up`, `arrow-forward-up`
- `folder-open`, `device-floppy`, `photo`
- `file-import`, `list-check`
- `activity`, `wave-sine`
- `player-play`, `player-pause`, `player-skip-forward`, `refresh`

Sudah di-vendor, belum dipakai:

- `zoom-in`, `zoom-out`, `focus-2`, `grid-dots`, `map`
- `search`, `settings`, `help-circle`, `alert-triangle`
- `circle-check`, `x`
- `cpu`, `circuit-resistor`, `circuit-capacitor`
- `circuit-diode`, `circuit-inductor`
- `database`, `chart-line`, `bug`, `crosshair`
- `math-function`, `ruler-measure`, `binary`
- `file-text`, `folder`, `package`
- `trash`, `copy`, `cut`, `clipboard`
- `sun`, `moon`

Kandidat ikon yang perlu di-vendor:

- `rotate-clockwise`, `flip-horizontal`, `flip-vertical`
- `plus`, `minus`, `pencil`, `dots-vertical`
- `chevron-down`, `chevron-right`
- `arrow-up`, `arrow-down`, `arrow-left`, `arrow-right`
- `select`, `pointer`, `hand-stop`
- `route`, `vector`, `git-branch`
- `plug`, `plug-connected`, `circuit-switch-closed`
- `wave-square`, `gauge`, `temperature`
- `device-desktop`, `window-maximize`, `window-minimize`

Setiap ikon baru harus:

- berada di `crates/gui/assets/icons/`,
- tercatat di `NOTICE.md`,
- ASCII,
- diparse oleh test `all_icons_parse`.

---

## 3. Roadmap

### U0 - Fondasi Tabler

- [x] U0-01 Vendor 45 ikon Tabler dan `NOTICE.md`.
- [x] U0-02 Tambahkan `IconKind` dan peta `OnceLock`.
- [x] U0-03 Tambahkan dukungan path busur `A` dan `a`.
- [x] U0-04 Tambahkan `draw_svg` yang dipakai simbol dan ikon.
- [x] U0-05 Tambahkan `IconButton`.
- [x] U0-06 Ganti ikon toolbar edit, file, simulasi, dan run.
- [x] U0-07 Ganti tombol run di panel simulasi.
- [x] U0-08 Test parse semua ikon dan test gambar.

### U1 - Toolbar dan Command Palette

- [ ] U1-01 Ganti tombol tool dengan ikon select, wire, dan probe.
- [ ] U1-02 Ganti tombol sheet dengan ikon pin, sheet, dan export.
- [ ] U1-03 Tambahkan ikon pada menu Recent.
- [ ] U1-04 Tambahkan ikon pada command palette.
- [ ] U1-05 Tambahkan shortcut hint pada tooltip.
- [ ] U1-06 Test render toolbar dan palette headless.

### U2 - Library

- [ ] U2-01 Petakan 16 kategori ke ikon Tabler.
- [ ] U2-02 Tambahkan ikon kategori pada header library.
- [ ] U2-03 Tambahkan ikon search dan clear pada kolom pencarian.
- [ ] U2-04 Tambahkan ikon recent dan filter.
- [ ] U2-05 Tambahkan indikator status model per baris.
- [ ] U2-06 Test filter kategori dengan ikon.

### U3 - Inspector dan Context Menu

- [ ] U3-01 Tambahkan ikon rotate, mirror, dan delete.
- [ ] U3-02 Tambahkan ikon copy, cut, dan duplicate.
- [ ] U3-03 Tambahkan ikon wire, junction, dan label.
- [ ] U3-04 Tambahkan ikon probe dan chart pada context menu.
- [ ] U3-05 Tambahkan ikon mode pin MCU.
- [ ] U3-06 Test action context menu dan inspector.

### U4 - Canvas dan Status

- [ ] U4-01 Tambahkan ikon zoom in, zoom out, dan fit.
- [ ] U4-02 Tambahkan ikon grid dan minimap.
- [ ] U4-03 Tambahkan ikon probe dan crosshair.
- [ ] U4-04 Tambahkan ikon status simulation dan FPS.
- [ ] U4-05 Tambahkan ikon tema terang dan gelap.
- [ ] U4-06 Test status bar dan overlay canvas.

### U5 - Simulasi dan Mixed Worker

- [ ] U5-01 Tambahkan probe per net ke trace mixed.
- [ ] U5-02 Tambahkan channel ADC ke grafik mixed.
- [ ] U5-03 Tambahkan sinyal PWM ke grafik mixed.
- [ ] U5-04 Tambahkan setting level high, low, resistance, ADC reference,
  dan UART threshold.
- [ ] U5-05 Tambahkan pilihan sample dan durasi ke panel mixed.
- [ ] U5-06 Tambahkan ikon probe dan chart pada panel mixed.
- [ ] U5-07 Test trace mixed dengan dua net dan satu channel ADC.
- [ ] U5-08 Hubungkan kontrol mixed ke board dan panel simulasi.
  Item ini melengkapi `TODO-SIMULATION.md` S7-05.

### U6 - Aksesibilitas dan Polish

- [ ] U6-01 Pastikan setiap icon button punya tooltip teks.
- [ ] U6-02 Tambahkan mode ikon saja dengan tooltip.
- [ ] U6-03 Periksa kontras ikon pada tema gelap dan terang.
- [ ] U6-04 Periksa ukuran ikon pada high-DPI.
- [ ] U6-05 Tambahkan golden screenshot toolbar.
- [ ] U6-06 Tambahkan test keyboard navigation.

### U7 - Rilis dan Dokumentasi

- [ ] U7-01 Perbarui `docs/57-gui-egui.md` dengan kebijakan ikon.
- [ ] U7-02 Perbarui `docs/95-ui-icons.md` dengan daftar final.
- [ ] U7-03 Perbarui `TODO-GUI.md` G1-12 dan G9 bila perlu.
- [ ] U7-04 Perbarui `README.md` dan `docs/README.md`.
- [ ] U7-05 Audit ulang lisensi Tabler dan `NOTICE.md`.
- [ ] U7-06 Jalankan fmt, clippy, test, ASCII, dan packaging.

---

## 4. Peta Cross-Reference

| Kebutuhan | Sumber lama | Status di file ini |
|---|---|---|
| Ikon toolbar | `TODO-GUI.md` G1-12 | U0 dan U1 |
| Library kategori | `TODO-GUI.md` G2-04 | U2 |
| Inspector tiga section | `TODO-GUI.md` G3-09 | U3 |
| Panel simulasi | `TODO-GUI.md` G6 | U5 |
| Probe dan grafik | `TODO-GUI.md` G7 | U5 |
| Polish dan aksesibilitas | `TODO-GUI.md` G9 | U6 |
| Mixed worker dan pin map | `TODO-SIMULATION.md` S7-05 | U5 |
| Event mixed | `TODO-SIMULATION.md` S7-09 | U5 |
| Desain GUI | `docs/57-gui-egui.md` | Semua U |
| Kebijakan ikon | `docs/95-ui-icons.md` | Semua U |
| Mixed signal | `docs/90-mixed-signal.md` | U5 |

---

## 5. Definition of Done

- [ ] Semua surface pada bagian 1 memiliki ikon atau alasan eksplisit
  untuk tetap teks.
- [ ] Setiap icon button memiliki tooltip teks.
- [ ] Semua ikon berasal dari Tabler dan tercatat di `NOTICE.md`.
- [ ] Tidak ada crate ikon eksternal baru.
- [ ] Semua SVG ASCII dan lolos test parse.
- [ ] Mixed worker mendukung probe per net, ADC, dan PWM.
- [ ] Grafik mixed deterministik dan diuji.
- [ ] Toolbar, library, inspector, context menu, dan palette memakai
  ikon yang konsisten.
- [ ] `fmt`, clippy, test, ASCII, dan packaging lulus.
