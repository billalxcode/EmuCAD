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
- 353 entry solver-backed.
- 147 entry behavioral atau unsupported.
- Setiap entry solver-backed memiliki `SimulationBinding` yang
  memvalidasi jumlah pin, role unik, dan parameter utama.
- Setiap entry solver-backed memiliki label fidelity dari
  `SimulationBinding::fidelity`.

Kelas yang belum solver-backed tetap `unsupported` dan tidak diberi
hasil simulasi. Entry unsupported harus ditolak oleh netlist GUI.

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
Kelas `BehavioralBlock` sudah sebagian: 5 filter solver-backed dan 20
entry lain masih unsupported.

## 3. Benchmark AVR

Perintah:

```bash
cargo run --release -p sim-avr --example avr_bench -- 200000
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
hybrid: datapath, register bank, dan beberapa unit memakai
`sim-cells`; kontrol dan peripheral belum seluruhnya gate-level.
Benchmark ini belum membuktikan AVR full-chip gate-level.

## 4. Benchmark Memory

Perintah:

```bash
cargo run --release -p sim-cells --example memory_bench
```

Contoh hasil register file gate-level 32 register:

| Width | Gate | DFF | Ops/s | ns/op |
|---:|---:|---:|---:|---:|
| 8 | 992 | 256 | 165407 | 6046 |
| 16 | 1744 | 512 | 95913 | 10426 |
| 64 | 6256 | 2048 | 23580 | 42408 |

Satu operasi benchmark mencakup tulis, clock tick, pembacaan, dan
settle. Pada clock 16 MHz, budget satu siklus adalah 62.5 ns. Memory
macro 32x8 berada jauh di atas budget tersebut jika setiap akses
disimulasikan gate-level penuh.

Kesimpulan: target real-time semua MCU dengan memory gate-level penuh
belum terbukti. Fallback Q55 perlu dipakai sampai ada model memory
yang lebih cepat atau batas abstraksi yang disepakati.

## 5. Batas

- Belum ada reference vector per entry.
- Belum ada golden ngspice di CI.
- Belum ada full-chip AVR gate-level.
- Belum ada 15 memory gate-level.
- Benchmark bergantung pada CPU dan build `--release`.
