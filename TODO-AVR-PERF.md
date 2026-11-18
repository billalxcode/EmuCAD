# TODO - Performa Simulasi AVR

Target: simulasi AVR mendekati real-time 16 MHz tanpa mengorbankan
kebenaran fungsional. Setiap perubahan diukur sebelum dan sesudah.

## Status

- Terakhir diperbarui: 2026-11-15.
- Commit timer fungsional: `0393547`.
- Commit perbaikan timing: `9ec28d1`.
- Commit pacing CLI: `71b1fd5`.
- Commit pacing GUI: `e173f6b`.
- Commit verifikasi realtime: `63279a9`.
- Build benchmark: release.
- Test workspace: 829 lulus.
- Headroom: 2,14x sampai 4,50x real-time tanpa pacing.
- Pacing CLI: median 0,275-0,536%, worst di bawah 0,9% pada verifikasi final.
- Pacing GUI: release test 16.000.000 cycle selesai dalam sekitar 1,00 detik.
- Sisa pekerjaan: Timer1/2 register IO, sleep wake, dan firmware penuh.

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
| Timer0 fungsional, prescaler 1 | 27.368.581 | 29.366.545 | 36.708.181 | 34,1 | 2,294x |
| Final, median 3 run, loop ALU | 34.385.807 | 40.184.314 | 50.230.393 | 24,9 | 3,139x |

## Hasil Final

Median tiga run, 1.000.000 instruksi, release build:

| Program | mode | instructions/detik | cycles/detik | ns/instruksi | real-time |
|---|---|---:|---:|---:|---:|
| ALU | traced | 34.385.807 | 42.982.259 | 29,1 | 2,686x |
| ALU | untraced | 40.184.314 | 50.230.393 | 24,9 | 3,139x |
| Mixed | traced | 35.348.628 | 41.775.680 | 28,3 | 2,611x |
| Mixed | untraced | 38.913.534 | 45.988.754 | 25,7 | 2,874x |
| Memory | traced | 35.832.351 | 62.706.614 | 27,9 | 3,919x |
| Memory | untraced | 41.166.762 | 72.041.833 | 24,3 | 4,503x |
| Timer0 prescaler 1 | traced | 27.368.581 | 34.210.726 | 36,5 | 2,138x |
| Timer0 prescaler 1 | untraced | 29.366.545 | 36.708.181 | 34,1 | 2,294x |
| Timer0 prescaler 64 | traced | 29.420.540 | 36.775.675 | 34,0 | 2,298x |
| Timer0 prescaler 64 | untraced | 32.874.932 | 41.093.665 | 30,4 | 2,568x |

Ukuran circuit CPU setelah optimasi: 120 gate, 3 komponen, 48 DFF,
komponen terbesar 40 gate.

## Pacing Realtime

Target 16.000.000 cycle/detik, toleransi 1%. Pengukuran 10.000.000
instruksi, lima run, mode `emucad run --realtime`:

| Program | median error | worst error |
|---|---:|---:|
| ALU | +0,454% | +0,606% |
| Mixed | +0,536% | +0,842% |
| Memory | +0,275% | +0,416% |
| Timer0 prescaler 1 | +0,351% | +0,441% |

Reproduksi:

```bash
bash scripts/bench-avr.sh 1000000
bash scripts/check-avr-realtime.sh
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
- [x] P-A22 Verifikasi final 829 test, clippy, ASCII, dan package.
- [x] P-A23 Tambah real-time pacing untuk CLI dan GUI.

## Temuan Akurasi 1% (2026-11-15)

- Target: 16.000.000 cycle/detik dengan toleransi 1% (15,84-16,16 juta).
- Tanpa timer: 2,69x-4,50x real-time.
- Timer0 prescaler 64: 2,30x-2,57x real-time.
- Timer0 prescaler 1: 2,14x-2,29x real-time setelah counter fungsional.
- Pacing CLI: median 0,275-0,536%, worst di bawah 0,9% pada verifikasi final.
- Pacing GUI: release test 16.000.000 cycle selesai dalam sekitar 1,00 detik.
- Pacing GUI memakai algoritma yang sama; release test worker lulus.
- Gap yang sudah diperbaiki: UART U2X/parity/stop/9-bit, SPI CS=3,
  ADC first conversion, rumus TWI, dan EICALL/EIJMP.
- Gap yang tersisa: Timer1/2 belum terhubung ke IO, sleep wake belum
  menambah cycle, watchdog belum dimodelkan, dan matrix cycle seluruh
  instruksi belum lengkap.

- [x] P-A25 Ganti counter Timer0/1/2 dengan counter integer fungsional.
- [x] P-A26 Tambah pacing 16 MHz dengan toleransi 1% per 1 detik.
- [x] P-A27 Perbaiki timing UART U2X, parity, stop bit, dan 9-bit.
- [x] P-A28 Perbaiki tabel divider SPI untuk CS=3 dan SPI2X.
- [x] P-A29 Perbaiki ADC first conversion dan rumus TWI.
- [x] P-A30 Perbaiki EICALL/EIJMP dan verifikasi cycle utama.
- [ ] P-A31 Hubungkan Timer1 dan Timer2 ke register IO CPU.
- [ ] P-A32 Modelkan sleep wake-up cycles.
- [ ] P-A33 Audit matrix cycle seluruh instruksi.
- [ ] P-A34 Benchmark firmware Arduino penuh dengan Timer1/2 dan UART.

## Riwayat Commit

| Commit | Isi |
|---|---|
| `5f7e1f3` | Optimasi topological order, dirty cone, bitset, dan read port |
| `813ab09` | Functional fast core, decode cache, benchmark, dan test |
| `320e7ff` | Dokumentasi hasil benchmark dan status performa |
| `bdd67c6` | Kurangi overhead board GUI dan verifikasi pacing |

## Catatan

- Threading tidak dijadikan prioritas untuk satu core AVR.
- Threading hanya dipertimbangkan untuk UI, batch, atau banyak instance.
- Hot path CPU sekarang fungsional. Primitif struktural tetap tersedia di
  `cells` dan `avr` untuk validasi dan pengembangan mode struktural.
- Angka benchmark adalah loop sintetis; firmware nyata dengan peripheral
  aktif perlu diukur terpisah.
