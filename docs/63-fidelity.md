# 63 - Fidelity dan Audit Kelas

Dokumen ini mencatat label fidelity, audit 60 model class, dan hasil
benchmark kelayakan. Data per kelas ada di
[`data/class-fidelity.csv`](data/class-fidelity.csv).

## 1. Label Fidelity

| Label | Arti |
|---|---|
| `ideal` | Elemen linier ideal atau sumber ideal. |
| `compact` | Model listrik dengan persamaan fisik dan batas valid. |
| `behavioral` | Blok behavioral dengan input-output listrik. |
| `calibrated` | Model yang sudah dicocokkan ke reference vector. |
| `unsupported` | Belum punya model solver; harus ditolak dengan error. |

Saat ini belum ada entry `calibrated`. Reference vector dan golden
ngspice belum dijalankan untuk seluruh keluarga.

## 2. Audit Kelas

- 500 entry katalog.
- 60 model class.
- 379 entry solver-backed.
- 121 entry behavioral atau unsupported.
- Setiap entry solver-backed memiliki `SimulationBinding` yang
  memvalidasi jumlah pin, role unik, dan parameter utama.
- Setiap entry solver-backed memiliki label fidelity dari
  `SimulationBinding::fidelity`.

Pada jalur solver analog, entry non-solver-backed tetap `unsupported`
dan tidak diberi hasil simulasi analog. Entry yang punya model digital
dijalankan oleh `cells`; 15 entry `Family::Memory` termasuk kelompok ini.

Kelas `Sensor` sudah seluruhnya solver-backed: 35 sensor.
Kelas `Actuator` dan `Display` sudah seluruhnya solver-backed: 20
entry dengan model elektrik behavioral.
Kelas `RfBlock` sudah sebagian: 4 RF amplifier solver-backed dan 14
entry RF lain masih unsupported.
Kelas `DcDc` dan `ChargePump` sudah solver-backed: 8 dan 5 entry
memakai model konverter teregulasi.
Kelas `Supervisor` sudah solver-backed: 5 entry reference dan
supervisor.
Kelas `MotorDriver` sudah solver-backed: 3 entry switch satu kanal.
Kelas `BehavioralSource` sudah sebagian: 2 sine source solver-backed
dan 9 entry lain masih unsupported.
Kelas `Timer` sudah sebagian: 4 timer/oscillator solver-backed dan
PLL masih unsupported.
Kelas `BehavioralSource` sudah sebagian: 7 source solver-backed dan 4
entry behavioral/polynomial/table masih unsupported.
Kelas `BehavioralBlock` sudah sebagian: 24 filter, math, detector,
oscillator, converter, dan PLL solver-backed; 1 entry R-2R DAC masih unsupported.
Kelas `BehavioralSource` sudah sebagian: 9 source solver-backed dan 2
behavioral V/I masih unsupported.

## 3. Benchmark AVR

Perintah:

```bash
cargo run --release -p avr --example avr_bench -- 200000
```

Contoh hasil pada desktop Linux x86_64:

| Program | Mode | Realtime |
|---|---|---:|
| ALU | untraced | 3.185x |
| Mixed | untraced | 2.921x |
| Memory | untraced | 4.850x |
| Timer | untraced | 2.538x |
| Timer64 | untraced | 2.587x |
| GPIO | untraced | 3.584x |

Statistik sirkuit untuk program ALU: 120 gate, 3 komponen, 48 DFF,
0 komponen siklik.

Angka ini adalah sampel, bukan jaminan. AVR saat ini adalah core
hybrid: hanya GPIO yang structural dan terhubung ke `DigitalCircuit`;
register bank, instruction register, ALU, shift, dan multiply tersedia
sebagai blok structural terpisah, tetapi belum dipakai jalur eksekusi.
Kontrol dan peripheral belum gate-level. Benchmark ini belum
membuktikan AVR full-chip gate-level.

## 4. Benchmark Memory

Perintah:

```bash
cargo run --release -p cells --example memory_bench
```

Contoh hasil register file gate-level 32 register:

| Width | Gate | DFF | Ops/s | ns/op |
|---:|---:|---:|---:|---:|
| 8 | 992 | 256 | 165407 | 6046 |
| 16 | 1744 | 512 | 95913 | 10426 |
| 64 | 6256 | 2048 | 23580 | 42408 |

Array SRAM 128x8 (1024 bit) pada run yang sama:

| Macro | Gate | DFF | Ops/s | ns/op |
|---|---:|---:|---:|---:|
| SRAM 128x8 | 4665 | 1024 | 39683 | 25199.5 |

Benchmark SRAM memakai port bernama katalog, sehingga angka ns/op
termasuk overhead lookup input dan output. Satu operasi benchmark
mencakup tulis, clock tick, pembacaan, dan settle. Pada clock 16 MHz,
budget satu siklus adalah 62.5 ns. Memory macro 32x8 dan SRAM 128x8
berada jauh di atas budget tersebut jika setiap akses disimulasikan
gate-level penuh.

Kesimpulan: target real-time semua MCU dengan memory gate-level penuh
belum terbukti. Fallback Q55 perlu dipakai sampai ada model memory
yang lebih cepat atau batas abstraksi yang disepakati.

## 5. Status Memory

Matriks ini memisahkan cakupan katalog, model digital, dan verifikasi.
Angka 15/15 hanya berarti setiap entry katalog punya model digital,
bukan verifikasi penuh.

| Tingkat | Jumlah | Bukti | Arti |
|---|---:|---|---|
| Entry katalog `Family::Memory` | 15 | `crates/device/src/catalog_data/memory.rs`; `crates/device/src/catalog/tests.rs:50` | Seluruh entry memory di katalog. |
| Entry dengan model digital `cells` | 15/15 | `crates/cells/src/catalog_digital.rs:135-169` | Setiap entry dipetakan ke `DigitalEntry` dua keadaan. |
| Macro, array, atau controller digital | 12/15 | `crates/cells/src/catalog_memory.rs` | SRAM cell/array, DRAM cell, ROM, PROM, EEPROM, Flash, FRAM, MRAM, NVRAM, module, dan controller. |
| Protocol wrapper digital | 3/15 | `crates/cells/src/catalog_flash.rs`, `crates/cells/src/catalog_eeprom.rs`, `crates/cells/src/catalog_sd.rs` | SPI flash, I2C EEPROM, dan SPI mode SD card. |
| Entry dengan test unit langsung | 11/15 | modul `tests` di file sumber yang sama | EEPROM, Flash, MRAM, dan NVRAM memakai jalur `memory_array_entry` yang sama dengan FRAM, tetapi tidak punya test khusus entry. |
| Entry terverifikasi penuh | 0/15 | tidak ada reference vector atau data kalibrasi | Unit test fungsional belum sama dengan verifikasi. |

Ukuran model: 2 entry memakai sel 1-bit (`sram_cell` dan `dram_cell`),
12 entry memakai array 128x8 (1024 bit), dan 1 entry memakai array
512x8 (4096 bit, `sd_card`). Lihat `crates/cells/src/catalog_memory.rs`
dan `crates/cells/src/catalog_sd.rs`.

Baris `Memory,15,0,unsupported` di `docs/data/class-fidelity.csv`
menghitung solver analog: 0 entry solver-backed. Baris itu tidak berarti
15 entry tanpa model digital.

Catatan:
- `sram6t` di `crates/cells/src/sequential/memory.rs` adalah sel SRAM 6T
  level transistor untuk referensi analog. Ini bukan model gate-level
  dan tidak dipakai array katalog.
- `memory_array_entry` di `crates/cells/src/catalog_memory.rs` adalah
  array DFF digital, bukan sel 6T.
- EEPROM, Flash, FRAM, MRAM, dan NVRAM memakai
  `memory_array_entry(..., true)`, yaitu array DFF yang mempertahankan
  isi saat reset. Model ini belum memodelkan program/erase, endurance,
  atau retensi fisik.
- `dram_cell` adalah model 1-bit dengan counter refresh, bukan sel
  kapasitor.
- SRAM data dan Flash program AVR tetap behavioral: `Cpu.data` berupa
  `Vec<u8>` dan `ProgramMemory` berupa `Vec<u16>` di
  `crates/avr/src/cpu/mod.rs` dan `crates/avr/src/fetch.rs`. Karena itu
  label behavioral di `docs/07-verification.md` berlaku untuk memory
  jalur eksekusi AVR, bukan untuk 15 entry katalog.
- Benchmark di bagian 4 hanya mengukur `memory.device.sram_array`
  (1/15 entry) dan register file 32x8 yang bukan entry katalog. 14 entry
  katalog lain belum diukur.

## 6. Batas

- Belum ada reference vector per entry.
- Belum ada golden ngspice di CI.
- Belum ada full-chip AVR gate-level.
- 0 dari 15 entry memory terverifikasi penuh.
- Benchmark bergantung pada CPU dan build `--release`.
