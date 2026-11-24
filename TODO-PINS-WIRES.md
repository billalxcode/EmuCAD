# TODO - Pin Layout dan Wire Movement

Backlog perbaikan dua bug yang ditemukan pada schematic:

1. Wire tertinggal saat komponen dipindahkan.
2. Posisi pin tidak mengikuti simbol komponen.

**Status:** P0, P1 switch/button, W1, dan W2 selesai. P1 transistor/opamp, P2 penuh, dan W3 belum.

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

- [x] P1-01 Tambahkan tabel layout pin per base symbol (switch, button, SPDT).
- [x] P1-02 Gunakan tabel untuk katalog, bukan `pin_layout(count)`.
- [x] P1-03 Perbaiki base selection switch dan analog switch berdasarkan
  jumlah pin.
- [x] P1-04 Perbaiki definisi built-in button menjadi body 4x4 dengan pin
  tengah kiri/kanan.
- [x] P1-05 Tambahkan `PinDirection` eksplisit dan transform arah pin.
- [x] P1-06 Validasi jumlah pin, duplikasi anchor, boundary, dan arah pin.
- [~] P1-07 Test layout untuk resistor, button, switch, SPDT, dan IC.
  BJT dan opamp masih memakai fallback generik.

### P2 - Render Simbol dan Pin

- [~] P2-01 Button dan switch memakai body persegi yang sama dengan SVG.
  Transform generik untuk simbol lain belum disatukan.
- [~] P2-02 Switch dan button sudah diperbaiki; simbol lain belum diaudit.
- [ ] P2-03 Tambahkan golden screenshot untuk simbol yang diperbaiki.
- [ ] P2-04 Test bahwa anchor pin berada di boundary body atau di posisi
  yang didefinisikan layout.

### W1 - Wire Movement

- [x] W1-01 Hitung mapping pin lama ke pin baru saat move.
- [x] W1-02 Pindahkan endpoint wire yang terhubung secara atomik.
- [x] W1-03 Validasi ulang semua wire setelah move.
- [x] W1-04 Tolak move jika wire menjadi diagonal, bertabrakan, atau
  menabrak body komponen.
- [x] W1-05 Test move horizontal yang membuat wire mengikuti.
- [x] W1-06 Test move yang harus ditolak.
- [~] W1-07 Wire tetap valid setelah move; test netlist penuh belum
  ditambahkan karena fixture ground belum disiapkan.

### W2 - Rotate dan Mirror

- [x] W2-01 Tolak rotate atau mirror jika ada wire terhubung.
- [x] W2-02 Tampilkan pesan yang menjelaskan penolakan.
- [x] W2-03 Test rotate dan mirror dengan wire terhubung.
- [x] W2-04 Test rotate dan mirror tanpa wire tetap berhasil.

### W3 - Deteksi Dangling

- [ ] W3-01 Tandai endpoint wire yang tidak terhubung.
- [ ] W3-02 Tambahkan pemeriksaan netlist atau ERC.
- [ ] W3-03 Test marker dan peringatan dangling.

---

## 3. Definition of Done

- [x] Tidak ada wire yang tertinggal saat komponen dipindahkan.
- [x] Move yang tidak valid ditolak tanpa mengubah dokumen.
- [x] Rotate/mirror dengan wire terhubung tidak merusak net.
- [ ] Pin model dan pin visual memakai layout yang sama.
- [ ] Switch, button, SPDT, transistor, dan opamp memiliki pin yang
  sesuai simbol.
- [x] Semua test lama tetap lulus.
- [x] Test baru mencakup kedua bug.
- [x] ASCII, fmt, clippy, dan test lulus.
