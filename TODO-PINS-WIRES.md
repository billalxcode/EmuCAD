# TODO - Pin Layout dan Wire Movement

Backlog perbaikan dua bug yang ditemukan pada schematic:

1. Wire tertinggal saat komponen dipindahkan.
2. Posisi pin tidak mengikuti simbol komponen.

**Status:** P0, P1, P2, W1, W2, dan W3 selesai.

**Sumber analisis:** diskusi screenshot R1/SW1 tanggal 2026-11-24.

---

## 1. Bug dan Dampak

### B1 - Wire tertinggal

- `Document::move_components` hanya mengubah posisi komponen.
- Endpoint wire tidak ikut berubah.
- `valid_wire_ends` hanya berjalan saat wire dibuat atau diubah.
- Akibatnya net terputus tanpa peringatan.

### B2 - Pin tidak mengikuti simbol

- `pin_layout(count)` menentukan posisi pin hanya dari jumlah pin.
- `catalog_definition` memasangkan layout itu dengan SVG yang berbeda.
- `draw_symbol` dan `draw_pin` memakai transform terpisah.
- Akibatnya pin model bisa berada di sisi yang salah, atau tidak bertemu
  dengan garis pin di SVG.

---

## 2. Rencana Perbaikan

### P0 - Analisis

- [x] P0-01 Identifikasi dangling wire setelah move.
- [x] P0-02 Identifikasi layout pin berbasis jumlah pin.
- [x] P0-03 Petakan simbol yang terpengaruh: switch, button, SPDT,
  transistor, opamp, dan katalog.

### P1 - Layout Pin per Simbol

- [x] P1-01 Tambahkan tabel layout pin per base symbol (switch, button,
  SPDT, BJT, JFET, MOSFET, IGBT, opamp, comparator).
- [x] P1-02 Gunakan tabel untuk katalog, bukan `pin_layout(count)`.
- [x] P1-03 Perbaiki base selection switch dan analog switch berdasarkan
  jumlah pin.
- [x] P1-04 Perbaiki definisi built-in button menjadi body 4x4 dengan pin
  tengah kiri/kanan.
- [x] P1-05 Tambahkan `PinDirection` eksplisit dan transform arah pin.
- [x] P1-06 Validasi jumlah pin, duplikasi anchor, boundary, dan arah pin.
- [x] P1-07 Test layout untuk resistor, button, switch, SPDT, IC, BJT,
  JFET, MOSFET, IGBT, opamp, dan comparator.

### P2 - Render Simbol dan Pin

- [x] P2-01 Gunakan pemetaan viewBox yang sama untuk body, pin anchor, dan
  seluruh geometri SVG.
- [x] P2-02 Audit simbol switch, button, SPDT, BJT, JFET, MOSFET, IGBT,
  opamp, dan comparator terhadap anchor pin.
- [x] P2-03 Tambahkan golden scene untuk simbol yang diperbaiki.
- [x] P2-04 Test bahwa anchor pin berada di boundary body dan arahnya
  sesuai layout.

### W1 - Wire Movement

- [x] W1-01 Hitung mapping pin lama ke pin baru saat move.
- [x] W1-02 Pindahkan endpoint wire yang terhubung secara atomik.
- [x] W1-03 Validasi ulang semua wire setelah move.
- [x] W1-04 Tolak move jika wire menjadi diagonal, bertabrakan, atau
  menabrak body komponen.
- [x] W1-05 Test move horizontal yang membuat wire mengikuti.
- [x] W1-06 Test move yang harus ditolak.
- [x] W1-07 Test netlist tetap terhubung setelah move.

### W2 - Rotate dan Mirror

- [x] W2-01 Tolak rotate atau mirror jika ada wire terhubung.
- [x] W2-02 Tampilkan pesan yang menjelaskan penolakan.
- [x] W2-03 Test rotate dan mirror dengan wire terhubung.
- [x] W2-04 Test rotate dan mirror tanpa wire tetap berhasil.

### W3 - Deteksi Dangling

- [x] W3-01 Tandai endpoint wire yang tidak terhubung dengan marker danger
  di canvas.
- [x] W3-02 Tambahkan pemeriksaan `WireStub` di netlist dan hitungan
  `Dangling` di status bar.
- [x] W3-03 Test marker dan peringatan dangling.

---

## 3. Definition of Done

- [x] Tidak ada wire yang tertinggal saat komponen dipindahkan.
- [x] Move yang tidak valid ditolak tanpa mengubah dokumen.
- [x] Rotate/mirror dengan wire terhubung tidak merusak net.
- [x] Pin model dan pin visual memakai layout yang sama.
- [x] Switch, button, SPDT, transistor, dan opamp memiliki pin yang
  sesuai simbol.
- [x] Endpoint wire yang dangling ditandai dan dilaporkan.
- [x] Semua test lama tetap lulus.
- [x] Test baru mencakup kedua bug.
- [x] ASCII, fmt, clippy, dan test lulus.
