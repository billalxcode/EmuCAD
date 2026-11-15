# 61 - Design System GUI

Dokumen ini menetapkan token dan aturan tampilan GUI EmuCAD.

## 1. Tujuan

- Tampilan bersih dan tenang seperti aplikasi Adobe.
- Font jelas, ukuran konsisten, dan warna punya peran semantik.
- Nilai dapat direproduksi dari kode dan dokumen.
- Tidak memakai font proprietary Adobe Clean.

Referensi:

- Spectrum typography: https://opensource.adobe.com/spectrum-design-data/tokens/typography/
- Spectrum typography system: https://spectrum.adobe.com/foundations/typography/typography-system
- Spectrum color palette: https://opensource.adobe.com/spectrum-design-data/tokens/color-palette/
- Source Sans 3: https://github.com/adobe-fonts/source-sans
- Source Code Pro: https://github.com/adobe-fonts/source-code-pro

## 2. Font

| Peran | Font | Berkas | Lisensi |
|---|---|---|---|
| UI regular | Source Sans 3 Regular | `SourceSans3-Regular.ttf` | OFL 1.1 |
| UI medium | Source Sans 3 Medium | `SourceSans3-Medium.ttf` | OFL 1.1 |
| UI semibold | Source Sans 3 Semibold | `SourceSans3-Semibold.ttf` | OFL 1.1 |
| Judul | Source Sans 3 Bold | `SourceSans3-Bold.ttf` | OFL 1.1 |
| Data teknis | Source Code Pro Regular | `SourceCodePro-Regular.ttf` | OFL 1.1 |

Aturan:

- Teks UI, label, tombol, menu, dan inspector memakai Source Sans 3.
- Pin, koordinat, cycle, log, UART, dan nilai teknis memakai Source Code Pro.
- Jangan memakai font di luar daftar ini tanpa lisensi yang kompatibel.
- Jangan memakai Adobe Clean karena font tersebut proprietary.

## 3. Skala Tipografi

Skala mengikuti kelipatan Major Second dari Spectrum, dipangkas untuk
kepadatan aplikasi desktop.

| Token | Ukuran | Berat | Pemakaian |
|---|---:|---|---|
| `caption` | 11 px | Regular | Teks bantu |
| `small` | 11 px | Regular | Metadata |
| `body` | 14 px | Regular | Label dan isi |
| `button` | 14 px | Medium | Tombol |
| `heading` | 18 px | Semibold | Judul panel |
| `title` | 22 px | Bold | Judul utama |
| `mono` | 13 px | Regular | Data teknis |
| `pin` | 10 px | Regular | Label pin dan ruler |

Line height:

- Body dan code memakai sekitar 1.5x.
- Heading dan title memakai sekitar 1.3x.
- egui menambahkan 2 px `extra_text_line_spacing` untuk body.

## 4. Warna Semantik

Nilai ada di `crates/sim-gui/src/design.rs`.

| Peran | Dark | Light |
|---|---|---|
| Background | `#1B1B1B` | `#F8F8F8` |
| Panel | `#1B1B1B` | `#F8F8F8` |
| Surface | `#2C2C2C` | `#FFFFFF` |
| Surface raised | `#323232` | `#F3F3F3` |
| Surface hover | `#393939` | `#E9E9E9` |
| Surface active | `#444444` | `#DADADA` |
| Input | `#111111` | `#FFFFFF` |
| Border | `#444444` | `#DADADA` |
| Border strong | `#6D6D6D` | `#C6C6C6` |
| Text | `#DBDBDB` | `#292929` |
| Text strong | `#F2F2F2` | `#131313` |
| Text muted | `#AFAFAF` | `#717171` |
| Text disabled | `#6D6D6D` | `#C6C6C6` |
| Accent | `#5681FF` | `#3B63FB` |
| Success | `#0EAF62` | `#05834E` |
| Warning | `#E06400` | `#C24E00` |
| Danger | `#FF6B6B` | `#C9252D` |
| Canvas | `#1B1B1B` | `#F8F8F8` |

Aturan:

- Teks utama wajib memakai `text` atau `text_strong`.
- Teks sekunder memakai `text_muted`.
- Aksen hanya untuk fokus, pilihan, dan aksi utama.
- Success, warning, dan danger tidak dipakai sebagai warna dekoratif.
- Warna net punya pasangan dark dan light agar tetap lolos kontras WCAG AA.

## 5. Spacing, Radius, dan Kontrol

| Token | Nilai |
|---|---:|
| Item spacing | 8 x 4 px |
| Button padding | 10 x 4 px |
| Window margin | 12 px |
| Menu margin | 6 px |
| Interact size | 48 x 24 px |
| Slider width | 120 px |
| Combo width | 140 px |
| Text edit width | 220 px |
| Library row height | 28 px |
| Widget radius | 4 px |
| Window radius | 6 px |
| Focus ring | 2 px |

Gunakan kelipatan 4 px untuk spacing baru.

## 6. Implementasi

Modul: `crates/sim-gui/src/design.rs`.

- `DARK` dan `LIGHT` menyimpan token warna.
- `palette(dark)` dan `palette_for(context)` memilih token aktif.
- `install_fonts` memuat font tertanam.
- `configure_context` menerapkan style ke tema dark dan light.
- `medium`, `semibold`, `bold`, dan `mono` membuat `FontId` yang konsisten.
- `is_dark(context)` dipakai untuk warna net yang bergantung tema.

Aturan implementasi:

- Jangan menulis `Color32::from_rgb` baru di UI.
- Ambil warna dari `Palette` atau `Visuals`.
- Jangan menulis `FontId::monospace` baru; pakai `design::mono`.
- Jangan menulis ukuran font baru di luar skala.
- Komponen custom wajib membaca `palette_for(painter.ctx())`.
- Baris library memakai tinggi tetap 28 px, nama di kiri, nilai di kanan.
- Nilai yang sama dengan nama tidak ditampilkan dua kali.
- Teks yang terlalu panjang dipotong, bukan dibungkus, agar scrollbar stabil.

## 7. Verifikasi

- Test kontras teks dan teks muted pada dark dan light.
- Test registrasi font dan family semibold.
- Test style dark dan light memakai panel fill yang benar.
- Test warna net lolos WCAG AA pada kedua canvas.
- Golden RGBA diperbarui setelah perubahan style yang disengaja.

## 8. Batas

- egui tidak punya berat font per `RichText`; semibold dan bold memakai
  family khusus.
- Line height egui tidak identik dengan Spectrum Web.
- Source Sans 3 tidak identik dengan Adobe Clean.
- Token ini terinspirasi Spectrum, bukan salinan penuh design system Adobe.
