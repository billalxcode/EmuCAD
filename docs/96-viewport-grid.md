# 96 - Viewport dan Grid: Referensi KiCad

Dokumen ini mencatat hasil riset implementasi viewport dan grid KiCad
sebagai acuan untuk EmuCAD GUI. Fokusnya adalah perilaku, struktur data,
dan keputusan desain, bukan salinan kode.

## 1. Ringkasan

KiCad memisahkan tiga hal yang sering tercampur di GUI kecil:

1. **VIEW** menyimpan kamera, layer, item, dan bounding box.
2. **GAL** mengubah koordinat dunia menjadi layar dan menggambar.
3. **VIEW_CONTROLS** menangani input pan, zoom, snap, dan cursor.

Grid bukan bagian dari model rangkaian. Grid adalah properti tampilan dan
snap yang hidup di VIEW, GAL, dan GRID_HELPER. Pemisahan ini yang perlu
ditiru EmuCAD.

Temuan utama:

- Zoom selalu berpusat pada titik di bawah kursor, bukan di tengah layar,
  kecuali pengguna memilih zoom di tengah.
- Zoom memiliki batas per editor. Eeschema memakai 0.01 sampai 100.
- Grid digambar sebelum item, dengan gaya titik, garis, atau salib.
- Grid yang terlalu rapat tidak dikecilkan, tetapi dinaikkan kelipatannya
  sampai jarak layar minimal tercapai.
- Snap dihitung terpisah dari gambar grid: pembulatan ke kelipatan grid
  terdekat terhadap grid origin.
- Snap grid, snap objek, dan tampilan grid adalah tiga pengaturan berbeda.
- Status bar menampilkan koordinat cursor yang sudah di-snap, grid aktif,
  unit, dan zoom absolut.

## 2. Sumber

Riset memakai sumber primer berikut:

- KiCad source, tag `10.0.8`, commit
  `e432beac2668303194ba2040274f9e5efe773fc0`.
  Clone sparse di `~/Downloads/kicad-10.0.8-src` tanpa build.
- Dokumentasi KiCad 10.0 untuk Schematic Editor:
  `https://docs.kicad.org/10.0/en/eeschema/eeschema.html`
- KiCad AppImage 10.0.8 di `~/Downloads`, dijalankan dengan demo
  `ecc83-pp`.
- Observasi GUI terbatas memakai `bayu-computer-use`: window KiCad,
  status bar, dan grid dots pada demo `ecc83-pp`.

File sumber yang paling relevan:

| Area | File |
|---|---|
| Transformasi dunia dan layar | `common/gal/graphics_abstraction_layer.cpp` |
| Kontrak GAL | `include/gal/graphics_abstraction_layer.h` |
| Kamera VIEW | `common/view/view.cpp`, `include/view/view.h` |
| Input pan dan zoom | `common/view/wx_view_controls.cpp` |
| Zoom controller | `common/view/zoom_controller.cpp` |
| Batas dan preset zoom | `include/zoom_defines.h` |
| Gambar grid OpenGL | `common/gal/opengl/opengl_gal.cpp` |
| Gambar grid Cairo | `common/gal/cairo/cairo_gal.cpp` |
| Snap grid dan objek | `common/tool/grid_helper.cpp` |
| Pengaturan grid | `include/settings/grid_settings.h` |
| Dialog grid | `common/dialogs/panel_grid_settings.cpp` |
| Preferensi tampilan grid | `common/dialogs/panel_gal_options.cpp` |
| Status bar | `eeschema/sch_base_frame.cpp`, `common/eda_draw_frame.cpp` |

## 3. Arsitektur Viewport KiCad

### 3.1 VIEW dan GAL

`VIEW` menyimpan daftar item per layer dan kamera dunia. `GAL` adalah
backend gambar yang menyimpan matriks transformasi, skala, dan ukuran
layar. `VIEW::Redraw()` mengambil semua layer yang terlihat dan meminta
`GAL` menggambarnya. Grid digambar oleh `GAL` sebelum item.

Alur satu frame di `EDA_DRAW_PANEL_GAL::DoRePaint()`:

1. Jika tidak ada perubahan, frame dilewati.
2. Update item yang berubah.
3. Set warna background, grid, dan cursor.
4. Gambar grid jika target noncached kotor.
5. Gambar semua layer item.
6. Gambar cursor.
7. Swap buffer.

Grid tidak ikut cache item. Grid selalu digambar ulang saat target
noncached kotor. Ini membuat perubahan zoom dan pan langsung terlihat
tanpa membuang cache geometri item.

### 3.2 Matriks dunia ke layar

`GAL::ComputeWorldScreenMatrix()` menyusun matriks:

```
screen = translate(screen_size / 2)
       * rotate(rotation)
       * flip(flip_x, flip_y)
       * scale(world_scale)
       * translate(-look_at_point)
```

`world_scale` dihitung dari DPI layar, panjang satu unit dunia, dan
zoom factor:

```
world_scale = screen_dpi * world_unit_length * zoom_factor
```

Eeschema memakai unit dunia 100 nm (`SCH_WORLD_UNIT = 1e-7 / 0.0254`
inci). Pcbnew memakai 1 nm. Artinya zoom factor bukan persen grid, tetapi
skala absolut terhadap unit internal editor.

### 3.3 Zoom berpusat pada anchor

`VIEW::SetScale(scale, anchor)` menyimpan posisi layar anchor sebelum
zoom, mengganti skala, lalu menggeser center agar anchor kembali ke
posisi layar yang sama. Pseudocode:

```
anchor_screen = world_to_screen(anchor)
scale = clamp(scale, min_scale, max_scale)
center = center - (screen_to_world(anchor_screen) - anchor)
```

Ini sama dengan perilaku `GridCamera::zoom_at` yang sudah ada di EmuCAD.
Yang berbeda adalah batas dan rentang zoom.

### 3.4 Batas zoom

Batas zoom berbeda per editor:

| Editor | Minimum | Maksimum |
|---|---:|---:|
| Eeschema | 0.01 | 100 |
| Eeschema preview | 0.5 | 20 |
| Pcbnew | 0.1 | 50000 |
| Gerbview | 0.02 | 5000 |

EmuCAD saat ini memakai 0.25 sampai 4.0. Rentang itu hanya 25 sampai 400
persen dari grid 20 px, terlalu sempit untuk schematic besar atau
inspeksi pin.

### 3.5 Preset zoom

Eeschema memakai daftar:

```
0.05, 0.07, 0.1, 0.15, 0.2, 0.3, 0.5, 0.7, 1.0, 1.5, 2.0,
3.0, 4.5, 6.5, 10.0, 15.0, 20.0, 30.0, 45.0, 65.0, 100.0
```

Tombol zoom in mengalikan skala dengan 1.3, lalu memilih preset berikutnya
yang lebih besar. Zoom out membagi 1.3, lalu memilih preset sebelumnya.
Zoom dengan wheel tidak terikat preset; nilainya kontinu dan dibatasi
minimum serta maksimum editor.

### 3.6 Input pan

Perilaku default KiCad:

- Drag tombol tengah: pan.
- Drag tombol kanan: pan, tetapi klik kanan tetap membuka context menu.
- Wheel tanpa modifier: zoom.
- Shift + wheel: pan vertikal.
- Ctrl + wheel: pan horizontal.
- Pinch gesture: zoom dan pan sekaligus.
- Drag di tepi kanvas: autopan.
- Scrollbar: menggeser viewport berdasarkan boundary.

Pan drag mengubah center dengan delta dunia:

```
delta_world = screen_to_world(drag_delta)
center = center + delta_world
```

Tidak ada snapping saat pan. Grid dan item ikut bergerak sebagai satu
viewport.

Autopan aktif saat cursor berada dalam 2 persen tepi kanvas. Timer
menggeser center dengan kecepatan yang naik mendekati tepi. Autopan
default nonaktif di preferensi pengguna, tetapi aktif saat tool tertentu
membutuhkannya.

### 3.7 Wheel zoom

Controller konstan di GTK memakai:

```
factor = 1 + rotation * 0.002
```

`rotation` adalah nilai wheel mentah, biasanya 120 per notch. Jadi satu
notch menghasilkan faktor sekitar 1.24. Controller akselerasi memakai
timeout 500 ms dan menaikkan faktor jika wheel diputar cepat dalam arah
yang sama.

EmuCAD memakai `exp(scroll * 0.002)`. Untuk delta scroll yang setara,
hasilnya mirip. Yang perlu disamakan adalah normalisasi delta dan
pemusatan pada kursor.

### 3.8 Fit dan zoom area

`doZoomFit()`:

1. Set skala awal 1.0 agar matriks dan viewport valid.
2. Ambil bounding box dokumen, objek, atau seleksi.
3. Jika kosong, pakai default view bbox.
4. Hitung skala agar bbox masuk layar.
5. Bagi dengan margin 1.04.
6. Jika tinggi layar kurang dari 768 px, margin menjadi 1.10.
7. Untuk library viewer margin 1.30; symbol editor 1.48.
8. Set center ke pusat bbox.

Zoom tool menarik kotak seleksi. Drag kiri memperbesar area, drag kanan
memperkecil area. Skala dihitung dari rasio ukuran kotak terhadap
viewport, lalu center dipindah ke pusat kotak.

Eeschema juga menyimpan zoom terakhir untuk "Undo Last Zoom" dan
mengembalikannya saat berpindah sheet.

## 4. Grid KiCad

### 4.1 Data grid

`GAL` menyimpan:

| Field | Arti |
|---|---|
| `m_gridVisibility` | Grid terlihat atau tidak |
| `m_gridStyle` | Dots, Lines, atau SmallCross |
| `m_gridSize` | Ukuran grid dalam unit dunia |
| `m_gridOrigin` | Titik asal grid dalam unit dunia |
| `m_gridOffset` | Sisa origin modulo grid size |
| `m_gridTick` | Setiap kelipatan ini digambar lebih tebal, default 10 |
| `m_gridLineWidth` | Ketebalan garis grid |
| `m_gridMinSpacing` | Jarak layar minimum agar grid tidak terlalu rapat |
| `m_gridColor` | Warna grid dari tema |

`GRID_SETTINGS` di level aplikasi menyimpan daftar grid, index grid
aktif, dua fast grid, gaya, snap mode, dan override per tipe objek.

### 4.2 Default

Default umum:

- Gaya: dots.
- Ketebalan: 1.0 px, diskalakan DPI.
- Jarak minimum: 10 px.
- Coarse tick: setiap 10.
- Snap: always.
- Grid terlihat: true.
- Axes: false.
- Warna tema terang: sekitar 181, 181, 181.
- Warna tema gelap: dark gray.

Eeschema memakai daftar grid 100, 50, 25, dan 10 mil. Default aktif
adalah 50 mil. Pcbnew memakai daftar yang lebih panjang dari 1000 mil
sampai 0.01 mm.

### 4.3 Algoritma gambar

Pseudocode dari `DrawGrid()`:

```
world_start = inverse_matrix * (0, 0)
world_end   = inverse_matrix * screen_size

grid_screen = visible_grid_size(grid_size)
start_x = round((world_start.x - origin.x) / grid_screen.x)
end_x   = round((world_end.x   - origin.x) / grid_screen.x)
normalize(start_x, end_x)
start_x -= 1
end_x   += 1

for each y in start_y..end_y:
    for each x in start_x..end_x:
        draw marker at (x * grid_screen.x + origin.x,
                        y * grid_screen.y + origin.y)
```

Untuk gaya garis, gambar semua garis vertikal lalu horizontal. Untuk
gaya titik atau salib, gambar marker per persimpangan. Setiap indeks
yang habis dibagi `m_gridTick` digambar dengan lebar dua kali.

Grid digambar pada depth di belakang item. Jika axes aktif, garis grid
yang menutupi sumbu 0 dilewati, lalu sumbu digambar dengan warna khusus.

### 4.4 Grid adaptif

Grid logical tidak diubah saat zoom. Yang berubah hanya ukuran grid yang
terlihat:

```
grid_screen = grid_size
threshold = min_spacing / world_scale
while min(grid_screen) <= threshold:
    grid_screen *= grid_tick
```

Hasilnya, saat zoom out grid dapat melompat ke 10x atau 100x ukuran
logical. Snap tetap memakai grid logical. Ini penting: pengguna tidak
kehilangan presisi hanya karena marker grid tidak muat di layar.

EmuCAD sekarang memakai urutan 1, 2, 5 berdasarkan skala. Itu lebih halus
secara visual, tetapi tidak sesuai dengan coarse tick KiCad dan membuat
jarak grid major berubah tidak beraturan.

### 4.5 Snap

`GAL::GetGridPoint()` membulatkan ke kelipatan terdekat:

```
grid_point.x = round((point.x - offset.x) / size.x) * size.x + offset.x
grid_point.y = round((point.y - offset.y) / size.y) * size.y + offset.y
```

`GRID_HELPER::computeNearest()` melakukan hal yang sama dengan grid dan
origin yang sudah dibulatkan ke integer. Pembulatan ke integer ini
mencegah error floating point pada grid seperti 0.254 mm.

Snap mode:

| Mode | Arti |
|---|---|
| `ALWAYS` | Snap grid selalu aktif, termasuk saat grid disembunyikan |
| `WITH_GRID` | Snap hanya aktif saat grid terlihat |
| `NEVER` | Snap grid tidak aktif |

Snap grid bukan satu-satunya snap. GRID_HELPER juga memiliki snap objek:
pin, ujung wire, junction, label, dan anchor lain. Snap objek dapat
mengalahkan snap grid. `GetSnappedPoint()` mengembalikan posisi objek
jika ada.

Modifier default di Eeschema:

| Modifier | Efek |
|---|---|
| Ctrl | Menonaktifkan snap grid sementara |
| Shift | Menonaktifkan snap objek sementara |

### 4.6 Grid origin

Pcbnew menyimpan grid origin di design settings dan dapat diubah lewat
dialog. Eeschema mengembalikan origin 0 tetap:

```
const VECTOR2I& GetGridOrigin() const override { static VECTOR2I zero; return zero; }
void SetGridOrigin( const VECTOR2I& ) override {}
```

Jadi untuk schematic, grid origin selalu 0. EmuCAD yang hanya memiliki
domain schematic tidak perlu origin yang dapat diubah. Yang perlu
ditambahkan adalah indikator origin dan sumbu.

### 4.7 Daftar grid dan fast grid

Pengguna dapat:

- Menambah, menghapus, mengedit, dan mengurutkan grid.
- Memberi nama grid.
- Memakai ukuran X dan Y yang berbeda.
- Menentukan dua fast grid.
- Berpindah grid dengan N dan Shift+N.
- Berpindah fast grid dengan Alt+1 dan Alt+2.
- Memakai Alt+4 untuk bolak-balik fast grid 1 dan 2.
- Mengaktifkan override grid per tipe objek dengan Ctrl+Shift+G.

Override memungkinkan wire memakai grid 50 mil sementara teks memakai
grid lebih halus. Fitur ini berguna di PCB, tetapi belum diperlukan untuk
MVP EmuCAD yang seluruh modelnya integer.

### 4.8 Status bar

Eeschema menampilkan:

```
Z 0.59 | X 50 Y 2600 | dx -50 dy 2600 dist 2600 | kisi 50 | mils
```

Empat kelompok:

1. Zoom absolut.
2. Koordinat cursor yang sudah di-snap.
3. Delta terhadap local origin.
4. Ukuran grid dan unit.

EmuCAD sekarang menampilkan pointer mentah, zoom persen, ukuran sel
piksel, dan label "Snap grid" yang selalu sama. Itu belum mencerminkan
state snap yang sebenarnya.

### 4.9 Cursor

KiCad menggambar cursor di atas semua item. Mode cursor:

- Small cross.
- Fullscreen cross.
- Fullscreen diagonal.

Cursor dapat dipaksa selalu tampil. Posisi cursor yang dipakai tool
adalah posisi yang sudah di-snap, sedangkan posisi mouse mentah tetap
disimpan terpisah. Panah keyboard menggerakkan cursor per grid; Ctrl
ditambah panah menggerakkan 10x grid.

## 5. Observasi GUI

Observasi dilakukan pada demo `ecc83-pp` dengan KiCad 10.0.8 AppImage.
Hal yang terlihat:

- Grid memakai titik, bukan garis, pada pengaturan default.
- Status bar menampilkan zoom, koordinat X/Y, delta, ukuran grid, dan
  unit. Contoh: `Z 0.70`, `X -400 Y 1300`, `kisi 50`, `mils`.
- Grid tetap terlihat rapi saat zoom berubah. Marker tidak pernah
  menumpuk menjadi bidang abu-abu.
- Cursor berbentuk cross kecil dan berada di atas grid.
- Pan dan zoom tidak mengubah posisi logis komponen.

Kontrol GUI sempat terhenti karena `bayu-computer-use` memberi batas
"input paused while the user is using the pointer or keyboard". Karena
itu, verifikasi perilaku yang lebih rinci dilakukan dari source dan
dokumentasi resmi, bukan dari klik tambahan.

## 6. Peta EmuCAD Saat Ini

| Fitur | KiCad | EmuCAD sekarang | Gap |
|---|---|---|---|
| Pemisahan kamera dan render | VIEW + GAL | `GridCamera` + `canvas::show` | Cukup, tetap dipertahankan |
| Tipe koordinat | double | f64 untuk kamera, f32 di API egui | Sebagian |
| Zoom range | 0.01 sampai 100 | 0.01 sampai 100 | Selesai |
| Zoom anchor | Kursor | Kursor | Sudah sesuai |
| Preset zoom | 21 langkah eeschema | 21 langkah eeschema | Selesai |
| Wheel zoom | `1 + rot * 0.002` | `1.24` per notch | Selesai |
| Wheel pan | Shift/Ctrl | Shift/Ctrl | Selesai |
| Drag pan | Tengah dan kanan | Tengah dan Space+kiri | Sebagian |
| Autopan | 2 persen tepi | Tidak ada | Belum ada |
| Fit | All, objects, selection, margin | Fit content dan selection | Sebagian |
| Zoom area | Drag kotak | Tidak ada | Belum ada |
| Zoom history | Undo last zoom | 8 level dan command palette | Selesai |
| Grid logical | Ukuran tetap | 1 unit integer | Sudah, tetapi tidak bisa diubah |
| Grid tampilan | Coarse tick 10, min 10 px | 1/2/5, major 5 | Berbeda |
| Gaya grid | Dots, lines, crosses | Dots + major lines | Belum bisa dipilih |
| Axes | Opsional | Selalu digambar | Perlu toggle |
| Ruler | Tidak ada di kanvas | Selalu digambar | Pertahankan sebagai overlay opsional |
| Grid origin | 0 untuk schematic | 0 | Sudah sesuai |
| Snap grid | Mode always/with grid/never | Selalu | Belum ada mode |
| Snap objek | Pin, wire, junction, label | Pin untuk wire | Sebagian |
| Modifier snap | Ctrl dan Shift | Tidak ada | Belum ada |
| Cursor crosshair | 3 mode | Tidak ada | Belum ada |
| Status bar | Snap coords, delta, grid, unit | Raw coords, zoom, cell px | Perlu diperbaiki |
| Keyboard cursor | Panah per grid | Tidak ada | Belum ada |
| Grid settings UI | Dialog lengkap | Tidak ada | Belum ada |
| Persistensi grid | Gaya, daftar, fast grid | Hanya visible | Belum ada |

## 7. Keputusan Desain untuk EmuCAD

### 7.1 Pertahankan model grid integer

Model `GridPos { x: i32, y: i32 }` tidak diubah pada tahap ini. Satu unit
tetap menjadi satuan koneksi schematic. Grid step ditambahkan sebagai
pengali integer:

```
GridStep = 1, 2, 5, 10
snap(x) = round(x / step) * step
```

Dengan cara ini:

- Tidak ada migrasi dokumen.
- Semua pin dan wire tetap integer.
- Grid yang lebih kasar hanya mengubah snap dan marker.
- Grid yang lebih halus dari 1 unit tidak didukung. Itu keputusan sadar,
  bukan bug.

Alternatif sub-grid (float atau unit 1 mil) memerlukan migrasi model,
format proyek, routing, dan seluruh test. Itu tidak dilakukan tanpa
keputusan terpisah.

### 7.2 Kamera f64

Field kamera `pan`, `zoom`, dan `cell_px` memakai f64. Konversi ke f32
dilakukan di boundary egui saat menggambar dan membaca pointer. Ini
mengikuti presisi double KiCad dan mencegah jitter saat zoom 100x.

Batas zoom yang dipakai: 0.01 sampai 100, sama dengan Eeschema.
Preset memakai daftar Eeschema di atas, ditampilkan sebagai persen.
Perubahan ini selesai di U8-01.

### 7.3 Grid style dan coarse tick

Tambahkan enum:

```
GridStyle { Dots, Lines, Cross }
```

Default `Dots`. Marker major setiap 10 unit logis, dengan ukuran dua
kali. Grid adaptif memakai kelipatan 10 dan `min_spacing = 10 px`.
Sumbu X/Y menjadi toggle, default mati. Ruler dipertahankan sebagai
overlay EmuCAD, default mati atau opsional.

### 7.4 Snap state

Tambahkan state eksplisit:

```
SnapGrid { Always, WithGrid, Never }
snap_objects: bool
```

Snap grid dihitung dari `GridStep`. Snap objek mencakup pin, ujung wire,
junction, dan label. Jika snap objek aktif dan ada kandidat, posisi objek
menang. Status bar menampilkan sumber snap: `grid`, `object`, atau `off`.

Ctrl untuk menonaktifkan snap grid tidak dapat diterapkan sepenuhnya
karena commit posisi tetap harus integer. Yang dapat dilakukan:

- Shift menonaktifkan snap objek sementara.
- Ctrl menampilkan cursor mentah dan menonaktifkan preview snap, tetapi
  commit tetap dibulatkan ke integer terdekat.

Batasan ini harus ditulis di UI, bukan disembunyikan.

### 7.5 Input

Urutan prioritas implementasi:

1. Wheel zoom di kursor dengan delta yang dinormalisasi.
2. Shift/Ctrl + wheel untuk pan.
3. Middle drag dan Space + drag yang sudah ada.
4. F1 dan F2 untuk zoom in/out di kursor.
5. Home untuk fit all; Ctrl+Home untuk fit objects.
6. Drag kanan untuk pan hanya jika context click tetap bekerja.
7. Autopan ditunda sampai ada tool drag yang berjalan lama.

U8-02 menyelesaikan langkah 1, 2, dan 4. Langkah 3 sudah ada sebelumnya.
Drag kanan, fit baru, dan autopan belum dikerjakan.

### 7.6 Fit dan riwayat zoom

- `fit_all`: seluruh konten termasuk sheet.
- `fit_objects`: konten tanpa sheet.
- `fit_selection`: hanya seleksi.
- `fit_sheet`: area sheet.
- Margin 1.04; 1.10 jika tinggi viewport kurang dari 768 px.
- Simpan maksimal 8 level zoom terakhir untuk "Undo last zoom".

### 7.7 Cursor dan status

- Crosshair mode: small, full, diagonal.
- Crosshair digambar di atas scene.
- Status bar: X/Y snapped, dx/dy/dist, grid step, zoom persen, snap state,
  unit.
- Panah menggerakkan cursor per grid; Ctrl+panah 10x grid.
- Posisi mouse mentah tetap dipakai untuk hover dan context menu.

### 7.8 Persistensi

Proyek menyimpan:

```
view.grid = {
  visible: bool,
  step: 1,
  style: "dots",
  axes: bool,
  crosshair: "small",
  snap: "always",
  snap_objects: bool
}
```

Field baru memakai default jika tidak ada, sehingga proyek lama tetap
bisa dibuka. Format versi tidak perlu dinaikkan jika semua field
opsional.

## 8. Rencana Verifikasi

Test yang harus ada:

- Transformasi bolak-balik dunia dan layar pada zoom minimum dan
  maksimum.
- Zoom anchor mempertahankan titik dunia di bawah kursor.
- Clamp zoom tidak pernah melewati batas.
- Preset zoom memilih langkah berikutnya dan sebelumnya dengan benar.
- Fit all, fit objects, dan fit selection menghasilkan center dan skala
  yang diharapkan.
- Snap grid membulatkan ke kelipatan step terhadap origin.
- Grid adaptif tidak pernah menghasilkan jarak marker di bawah 10 px.
- Coarse marker muncul setiap 10 unit.
- Status bar menampilkan koordinat snapped, bukan raw.
- Keyboard cursor bergerak per grid dan 10x grid.
- Persistensi field grid baru membaca proyek lama tanpa error.
- Golden screenshot grid dots, lines, dan cross pada zoom rendah dan
  tinggi.

U8-01 sudah ditutup oleh test clamp zoom, daftar preset, round-trip
koordinat pada zoom maksimum, dan round-trip pan serta zoom di JSON.

U8-02 sudah ditutup oleh test normalisasi satu notch wheel, pan delta
terhadap ukuran viewport, dan riwayat zoom yang mengembalikan pan serta
zoom sebelumnya.

## 9. Batas dan Risiko

- Model integer membatasi grid halus dan free placement. Ini keputusan
  sadar sampai ada migrasi model.
- Grid non-persegi (X != Y) belum didukung. KiCad mendukungnya, tetapi
  EmuCAD belum membutuhkannya.
- Autopan dan infinite pan bergantung pada platform serta event loop
  egui. Implementasinya ditunda.
- Ruler adalah tambahan EmuCAD, bukan fitur KiCad. Jangan memakai ruler
  sebagai alasan untuk mengubah grid logical.
- Perbedaan DPI dan skala UI dapat membuat 10 px tidak sama dengan 10
  point. Test harus memakai skala 1.0 dan high-DPI.
- Source KiCad adalah GPL. Dokumen ini hanya mencatat perilaku dan
  algoritma; tidak ada kode KiCad yang disalin ke EmuCAD.

## 10. Rujukan Backlog

Implementasi viewport dan grid dipetakan ke `TODO-UI.md` bagian U8.
Item lama yang terkait:

- `TODO-GUI.md` G1-03, G1-04, G1-05, G1-10, G1-15.
- `TODO-GUI.md` G2-13 (snap sementara, tetap terblokir model integer).
- `TODO-GUI.md` G5-10 (ruler dan probe).
- `TODO-GUI.md` G9 (aksesibilitas dan golden image).
- `docs/57-gui-egui.md` bagian 5 dan 8.

U8 tidak menggantikan item lama. U8 memperluas perilaku yang sudah ada
dan menutup gap yang ditemukan dari KiCad.
