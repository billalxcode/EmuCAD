# TODO - Performa Simulasi AVR

Target: simulasi AVR mendekati real-time 16 MHz tanpa mengorbankan
kebenaran fungsional. Setiap perubahan diukur sebelum dan sesudah.

## Status

- Terakhir diperbarui: 2026-11-14.
- Commit implementasi: `813ab09` (fast core AVR).
- Commit dokumentasi: `320e7ff` (hasil benchmark).
- Build benchmark: release.
- Test workspace: 786 lulus.
- Hasil: 2,552x sampai 4,557x real-time pada tiga loop benchmark.
- Sisa pekerjaan: pacing real-time CLI/GUI dan benchmark firmware penuh.

## Aturan

- Benchmark memakai build release.
- Program benchmark: `alu`, `mixed`, dan `memory`.
- Setiap langkah mencatat instructions/detik, cycles/detik,
  ns/instruksi, dan real-time factor.
- Perubahan perilaku harus punya test.
- Optimasi yang tidak menaikkan performa dicatat dan dibatalkan.

## Baseline dan Riwayat

Angka di bawah adalah hasil representatif pada mesin pengembangan Linux.
Baseline awal memakai loop ALU dengan `step` bertrace.

| Tahap | traced instr/s | untraced instr/s | cycles/s | ns/instr | real-time |
|---|---:|---:|---:|---:|---:|
| Baseline `step` | 60.003 | - | 75.004 | 16.665,8 | 0,005x |
| `step_untraced` | - | 67.308 | 84.135 | 14.857,0 | 0,005x |
| NetId langsung + cache kontrol | 70.735 | 79.045 | 98.807 | 12.651,0 | 0,006x |
| Topological order + dirty fanout | 95.908 | 99.929 | 124.911 | 10.007,1 | 0,008x |
| Bitset dirty per komponen | 234.625 | 252.626 | 315.783 | 3.958,4 | 0,020x |
| Hapus read port dan gate duplikat | 364.227 | 414.070 | 517.588 | 2.415,1 | 0,032x |
| Register file fungsional | 844.273 | 889.575 | 1.111.969 | 1.124,1 | 0,069x |
| Fast path ADD/SUB/ADC/SBC/CP | 20.526.350 | 26.226.413 | 32.783.016 | 38,1 | 2,049x |
| Fast path logic/INC/DEC/shift/mul | 24.503.970 | 32.982.126 | 41.227.658 | 30,3 | 2,577x |
| Cache decode instruksi | 28.727.904 | 38.745.929 | 48.432.411 | 25,8 | 3,027x |
| Trace menyalin array register | 31.441.360 | 37.894.437 | 47.368.046 | 26,4 | 2,961x |
| Fast skip peripheral serial | 32.745.683 | 38.753.664 | 48.442.079 | 25,8 | 3,028x |
| Final, median 3 run, loop ALU | 32.667.451 | 37.830.936 | 47.288.670 | 26,4 | 2,956x |

## Hasil Final

Median tiga run, 1.000.000 instruksi, release build:

| Program | mode | instructions/detik | cycles/detik | ns/instruksi | real-time |
|---|---|---:|---:|---:|---:|
| ALU | traced | 32.667.451 | 40.834.314 | 30,6 | 2,552x |
| ALU | untraced | 37.830.936 | 47.288.670 | 26,4 | 2,956x |
| Mixed | traced | 34.835.892 | 41.169.720 | 28,7 | 2,573x |
| Mixed | untraced | 38.925.844 | 46.003.303 | 25,7 | 2,875x |
| Memory | traced | 35.460.085 | 62.055.149 | 28,2 | 3,878x |
| Memory | untraced | 41.667.510 | 72.918.143 | 24,0 | 4,557x |

Ukuran circuit CPU setelah optimasi: 120 gate, 3 komponen, 48 DFF,
komponen terbesar 40 gate.

Reproduksi:

```bash
bash scripts/bench-avr.sh 1000000
```

## Fase Optimasi

- [x] P-A01 Buat benchmark release yang representatif.
- [x] P-A02 Ukur baseline `step` bertrace.
- [x] P-A03 Tambah `step_untraced` dan `run_untraced`.
- [x] P-A04 Ukur setelah trace dihilangkan dari hot path.
- [x] P-A05 Tambah `DigitalCircuit::set_net` dan cache instruksi.
- [x] P-A06 Ukur setelah input instruksi memakai NetId.
- [x] P-A07 Cache semua net kontrol ALU dan register bank.
- [x] P-A08 Ukur setelah string lookup dihapus.
- [x] P-A09 Kurangi polling peripheral saat idle.
- [x] P-A10 Ukur setelah polling peripheral dikurangi.
- [x] P-A11 Optimasi evaluasi gate dan settle.
- [x] P-A12 Ukur setelah optimasi gate.
- [x] P-A13 Tambah fast skip peripheral serial.
- [x] P-A14 Tambah functional core sebagai hot path.
- [x] P-A15 Hapus unit struktural yang tidak lagi dipakai dari hot path.
- [x] P-A16 Cache decode instruksi per alamat.
- [x] P-A17 Tambah test signed multiply, shift flags, dan INC/DEC flags.
- [x] P-A18 Dokumentasikan benchmark dan batas performa.
- [x] P-A19 Tambah trace cepat dengan menyalin array register.
- [x] P-A20 Tambah benchmark loop memory dan test pointer.
- [x] P-A21 Inline jalur fetch dan step, lalu ukur ulang.
- [x] P-A22 Verifikasi final 786 test, clippy, ASCII, dan package.
- [ ] P-A23 Tambah real-time pacing untuk CLI dan GUI.
- [ ] P-A24 Ukur firmware Arduino penuh dengan peripheral aktif.

## Temuan Akurasi 1% (2026-11-15)

- Target: 16.000.000 cycle/detik dengan toleransi 1% (15,84-16,16 juta).
- Tanpa timer: 2,55x-4,56x real-time, jadi terlalu cepat dan belum di-pace.
- Timer0 prescaler 64: 1,78x-1,95x real-time.
- Timer0 prescaler 1: 0,126x-0,127x real-time, sekitar 8x lebih lambat.
- Penyebab utama: counter Timer0/1/2 masih struktural dan di-tick per cycle.
- Gap timing lain: UART U2X/parity/stop bit, SPI divider CS=3, ADC
  konversi pertama, rumus TWI, Timer1/2 belum terhubung ke IO, dan
  wake-up sleep belum menambah cycle.
- Instruction cycle belum diaudit menyeluruh; EICALL/EIJMP perlu dicek.
- Belum ada pacing 16 MHz di CLI maupun GUI.

- [ ] P-A25 Ganti counter Timer0/1/2 dengan counter integer fungsional.
- [ ] P-A26 Tambah pacing 16 MHz dengan toleransi 1% per 1 detik.
- [ ] P-A27 Perbaiki timing UART U2X, parity, stop bit, dan 9-bit.
- [ ] P-A28 Perbaiki tabel divider SPI untuk CS=3 dan SPI2X.
- [ ] P-A29 Perbaiki ADC first conversion dan rumus TWI.
- [ ] P-A30 Audit cycle seluruh instruksi dan benchmark firmware penuh.

## Riwayat Commit

| Commit | Isi |
|---|---|
| `5f7e1f3` | Optimasi topological order, dirty cone, bitset, dan read port |
| `813ab09` | Functional fast core, decode cache, benchmark, dan test |
| `320e7ff` | Dokumentasi hasil benchmark dan status performa |

## Catatan

- Threading tidak dijadikan prioritas untuk satu core AVR.
- Threading hanya dipertimbangkan untuk UI, batch, atau banyak instance.
- Hot path CPU sekarang fungsional. Primitif struktural tetap tersedia di
  `sim-cells` dan `sim-avr` untuk validasi dan pengembangan mode struktural.
- Angka benchmark adalah loop sintetis; firmware nyata dengan peripheral
  aktif perlu diukur terpisah.
