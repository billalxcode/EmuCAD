# 52 - Lisensi dan Notices

Dokumen ini mencatat status lisensi kode, model, dan data.

---

## 1. Kode

- Lisensi proyek: MIT, lihat `LICENSE`.
- Semua crate workspace memakai lisensi yang sama.
- Tidak ada crate Rust eksternal.
- Tidak ada kode Rust pihak ketiga yang disalin.
- Aset simbol SVG pihak ketiga dicatat di bagian 3.

---

## 2. Model Device

Persamaan device mengikuti model publik yang umum:

- Shockley diode.
- Shichman-Hodges MOSFET.
- Ebers-Moll BJT.
- Meyer capacitance.
- Lossless transmission line.
- Controlled sources.

Implementasi ditulis ulang dari persamaan, bukan salinan kode simulator
lain.

---

## 3. Aset Simbol SVG

Simbol library GUI memakai aset SVG dari sumber berikut:

- Sebagian besar base symbol kelas: `chris-pikul/electronic-symbols`,
  lisensi MIT, copyright (c) 2022 Chris Pikul.
- MCU: adaptasi dari `GenericIC.svg` di Wikimedia Commons, public domain.
- VCC, thermistor, transmission line, diode array, IGBT, SCR, TRIAC,
  DIAC, connector, IC, flip-flop, regulator, DC-DC, sensor, display, RF,
  behavioral, board, dan motor: geometri sederhana yang digambar untuk
  proyek ini.
- Setiap entri katalog mendapat kode varian 3 x 3 yang digambar oleh
  renderer. Kode ini membedakan 500 entri tanpa lisensi tambahan.

Teks lisensi lengkap, daftar aset, dan URL sumber ada di
`crates/gui/assets/symbols/NOTICE.md`. Paket rilis menyalin file itu
ke `share/licenses/gui-symbols.md`.

## 4. Font GUI

GUI membundel font open-source dari Adobe:

- Source Sans 3 Regular, Medium, Semibold, dan Bold: UI, tombol, heading, dan judul.
- Source Code Pro Regular: nilai teknis, pin, koordinat, dan log.

Keduanya memakai SIL Open Font License 1.1. Teks lisensi dan URL sumber
ada di `crates/gui/assets/fonts/NOTICE.md`. Paket rilis menyalin
`NOTICE.md`, `OFL-SourceSans.txt`, dan `OFL-SourceCodePro.txt` ke
`share/licenses/`.

## 5. Data Datasheet

Kalibrasi pin memakai DC characteristics ATmega328P:

- VOH 4.1 V pada 20 mA.
- VOL 0.8 V pada 20 mA.
- Pull-up 20-50 kOhm.

Sumber:

<https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf>

Repository hanya menyimpan nilai turunan dan prosedur kalibrasi, bukan
salinan tabel datasheet.

---

## 6. CI

Workflow memakai `actions/checkout@v4` untuk checkout repository. Tidak ada
dependency Rust eksternal di CI.

---

## 7. Batas

- Dokumen ini bukan nasihat hukum.
- Belum ada audit lisensi untuk firmware contoh.
- NOTICE aset SVG dan font sudah disertakan di paket rilis.
- Belum ada verifikasi lisensi datasheet untuk redistribusi penuh.
