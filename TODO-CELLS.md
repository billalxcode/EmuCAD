# TODO - Cells untuk Multi-MCU

Status: perencanaan.
Tanggal: 2026-11-20.
Sumber analisis: `docs/92-cells-mcu-requirements.md`.
Target: `cells` menjadi library digital yang cukup untuk membangun AVR,
8051, PIC, STM32, ESP32, dan CPU lain dari spesifikasi.

## Aturan

- Satu item selesai = kode + test + dokumentasi singkat.
- Tidak menambah dependency eksternal.
- Tidak memakai `unsafe`.
- Reuse primitive `cells` yang sudah ada sebelum membuat yang baru.
- Jangan menduplikasi item yang sudah ada di TODO lain.
- Jika item bersinggungan dengan TODO lain, tulis rujukannya.
- Perubahan structural tidak boleh menurunkan performa fast path AVR
  tanpa benchmark.

## Status Awal

Sudah tersedia:

- Gate dasar, mux, buffer, transmission gate, tri-state inverter.
- D latch, D flip-flop, SRAM 6T, ring oscillator.
- Adder, ALU 8-bit, register, counter, shift register.
- Register file 32 x 8.
- PC/SP 16-bit.
- CPU mainan 16-bit.
- Digital circuit dua keadaan, zero-delay, satu clock ideal.
- Entry katalog digital untuk timer, UART, SPI, I2C, CAN, USB token,
  Ethernet, dan beberapa memory macro.

Belum memadai:

- Width besar, bus fabric, multi-clock, timing, high-Z, cache, DMA,
  pin mux, dan spec-driven builder.

## Konflik dan Rujukan

Item di bawah ini tidak diduplikasi di TODO ini.

| TODO lain | Item | Hubungan |
|---|---|---|
| `TODO.md` | P0-09 | Crate `cells` sudah dibuat |
| `TODO.md` | P5 | Characterization dan switching cell |
| `TODO.md` | P22 | Performa AVR dan fast path |
| `TODO-AVR-PERF.md` | P-A15 | Unit structural dikeluarkan dari hot path |
| `TODO-AVR-PERF.md` | P-A31..P-A34 | Timer1/2, sleep wake, cycle audit |
| `TODO-SIMULATION.md` | S5-02 | Adopsi `cells` untuk logic/FF/counter |
| `TODO-SIMULATION.md` | S6-02..S6-06 | Memory/MCU gate-level dan timing |
| `TODO-SIMULATION.md` | S7-02, S7-04 | AVR structural dan wrapper |
| `TODO-COMPONENTS.md` | C6-08 | Entry katalog memory |
| `TODO-COMPONENTS.md` | C6-09 | Entry katalog MCU core 11 keluarga |
| `TODO-COMPONENTS.md` | C6-10 | Entry katalog ADC/DAC/PWM |
| `TODO-COMPONENTS.md` | C6-11 | Entry katalog UART/SPI/I2C/CAN/USB/Ethernet |
| `TODO-COMPONENTS.md` | C6-12 | Entry katalog timer/RTC/watchdog |
| `TODO-COMPONENTS.md` | C1-06..C1-10 | Harness reference, KCL, convergence |
| `TODO-GUI.md` | - | Tidak ada perubahan GUI di TODO ini |

Catatan:

- `TODO-COMPONENTS` mencatat entry katalog yang terlihat pengguna.
- `TODO-CELLS` mencatat library, generator, dan engine yang dipakai
  untuk membangun entry tersebut.
- `TODO-SIMULATION` mencatat integrasi ke MCU dan co-sim.
- Jika satu item membutuhkan entry katalog, item itu menjadi
  dependency, bukan duplikat.

## P0 - Fondasi Library

- [x] CELL-00-01 Buat tipe `DigitalVector` atau `Bus` dengan width,
  slice, concat, zero extension, dan sign extension.
  - `cells::DigitalVector` tersedia dengan allocate, new, slice,
    concat, zero_extend, sign_extend, dan read.
  - Width dibatasi 1..=64; slice dan extension divalidasi.
- [x] CELL-00-02 Tambahkan module hierarchy dengan port bernama dan
  parameter width.
  - `cells::DigitalModule` membangun template dengan port input/output.
  - `instantiate` menyalin gate dan DFF dengan remap net.
  - Instansiasi berulang menghasilkan net yang independen.
- [x] CELL-00-03 Tambahkan high-Z, pull-up, pull-down, wired-AND, dan
  resolusi bus contention.
  - Tri-state inverter sudah ada.
  - `BusResolver` menangani beberapa driver, pull low/high, dan
    contention.
  - `resolve_open_drain` menangani wired-AND dengan pull-up.
- [x] CELL-00-04 Tambahkan API clock domain dan tick per domain.
  - `ClockDomain` dengan default domain dan index.
  - `DigitalBuilder::dff_in` dan `dff_into_in`.
  - `DigitalCircuit::tick_domain`.
  - Penjadwalan periode per domain tetap di luar engine.
- [x] CELL-00-05 Tambahkan reset domain, async assert, dan sync deassert.
  - `ResetDomain` dan `dff_with` menandai DFF per domain.
  - `reset_domain` mereset hanya domain yang diminta.
  - `reset_synchronizer` menyediakan async assert dan sync deassert.
- [x] CELL-00-06 Definisikan `McuSpec`, `CpuSpec`, `BusSpec`,
  `MemorySpec`, `PeripheralSpec`, `PinSpec`, dan `ClockSpec`.
  - Semua tipe ada di crate `soc`.
  - `BusSpec` mencakup protocol, data width, dan address width.
  - Elaboration ke `cells` belum; lihat `docs/94-soc-spec.md`.
- [ ] CELL-00-07 Buat validator address map, IRQ map, pin map, dan
  memory map.
- [ ] CELL-00-08 Tambahkan enum fidelity per blok: behavioral,
  cycle-accurate, gate-level.
- [ ] CELL-00-09 Buat elaboration pipeline dari spec ke top-level
  circuit dan daftar peripheral.
- [ ] CELL-00-10 Buat manifest/snapshot deterministik untuk spec yang
  sudah di-elaborate.

Exit P0:

- Satu modul dua clock domain dapat dibangun dari spec.
- Bus high-Z dan konflik dapat dideteksi.
- Spec yang tidak valid ditolak dengan error yang jelas.

## P1 - Arithmetic, CPU, dan Memory

- [x] CELL-01-01 Generalisasi adder dan ALU ke width 8/16/32/64.
  - `digital_alu_width` mendukung width 1..=64.
  - `digital_alu` tetap wrapper 8-bit.
  - Test 32-bit arithmetic dan logic ditambahkan.
- [x] CELL-01-02 Tambahkan barrel shifter, rotate, dan funnel shift.
  - `barrel_shift_left`, `barrel_shift_right`, dan
    `arithmetic_shift_right` untuk width power-of-two 1..=64.
  - `rotate_left` dan `rotate_right` memakai funnel shift.
  - `funnel_shift_left` dan `funnel_shift_right` untuk dua operand.
- [x] CELL-01-03 Tambahkan multiplier array dan Booth multiplier.
  - `multiplier_array` untuk operand unsigned.
  - `multiplier_booth` untuk operand signed two's complement.
  - Width power-of-two 1..=32; produk 2 x width bit.
- [x] CELL-01-04 Tambahkan divider restored dan non-restored.
  - `divider_restoring` dan `divider_non_restoring` untuk operand
    unsigned.
  - Lebar power-of-two 1..=32; quotient dan remainder masing-masing
    width bit.
  - Pembagian oleh nol tidak didefinisikan.
- [x] CELL-01-05 Tambahkan comparator, priority encoder, one-hot
  encoder, dan leading-zero count.
  - `comparator_unsigned` mengembalikan less, equal, dan greater untuk
    width 1..=64.
  - `priority_encoder` memilih index terendah yang aktif.
  - `one_hot_encoder` mengubah vektor one-hot menjadi index biner.
  - `leading_zero_count` menghitung nol di depan untuk width
    power-of-two 1..=64.
- [x] CELL-01-06 Tambahkan bit reverse, byte swap, popcount, dan
  saturating arithmetic.
  - `bit_reverse` dan `byte_swap` mengubah urutan bit atau byte tanpa
    gate tambahan.
  - `popcount` menghitung bit satu untuk width power-of-two 1..=64.
  - `saturating_add_unsigned` dan `saturating_sub_unsigned` membatasi
    hasil pada nilai maksimum atau nol.
- [ ] CELL-01-07 Generalisasi LUT dan FSM dengan width dan jumlah state
  yang dapat dikonfigurasi.
- [x] CELL-01-08 Tambahkan crossbar dan bus mux parameterized.
  - `bus_mux` memilih satu dari 1..=64 bus dengan lebar sama.
  - `crossbar` merutekan satu bus per output secara independen.
  - Jumlah bus dan lebar select divalidasi.
- [ ] CELL-01-09 Tambahkan equivalence test antara model fungsional dan
  structural untuk ALU, shifter, multiplier, dan divider.
- [x] CELL-02-01 Tambahkan register dengan enable, clear, preset, dan
  byte enable.
  - `register_extended` memakai kontrol load, clear, dan preset.
  - Byte mask opsional; slice kosong berarti semua byte aktif.
  - Prioritas clear, lalu preset, lalu load, lalu hold.
- [x] CELL-02-02 Tambahkan counter dengan load, up/down, compare,
  capture, dan prescaler generic.
  - `counter_loadable` mendukung load, enable, up/down, dan width
    1..=64.
  - `counter_match` menghasilkan sinyal equality.
  - `prescaler` menghasilkan tick satu cycle setiap divisor.
  - Capture dapat dibangun dari `register_extended` dengan sinyal
    tick atau match sebagai load.
- [ ] CELL-02-03 Tambahkan FIFO sinkron dan async FIFO.
- [~] CELL-02-04 Buat memory compiler parameterized: depth, width,
  byte enable, initialization, latency, dan read-during-write.
- [ ] CELL-02-05 Tambahkan dual-port dan dual-clock RAM.
- [ ] CELL-02-06 Tambahkan cache tag/data array dan metadata valid,
  dirty, dan replacement.
- [ ] CELL-02-07 Tambahkan parity, ECC, error injection, dan scrubber.
- [ ] CELL-02-08 Tambahkan pipeline register, valid/ready handshake,
  dan skid buffer.
- [ ] CELL-02-09 Tambahkan model flash/EEPROM dengan erase/write timing
  behavioral.
- [x] CELL-03-01 Buat register file 32 x 32 dengan byte enable dan
  multi-port.
  - `register_file_masked` mendukung 2 read port, 1 write port, dan
    byte write mask.
  - Lebar data 1..=64 dan jumlah register 2..=32.
  - Multi-write port belum ada.
- [ ] CELL-03-02 Tambahkan pipeline, hazard unit, forwarding, dan
  branch unit.
- [ ] CELL-03-03 Tambahkan CSR file, trap controller, dan interrupt
  entry.
- [ ] CELL-03-04 Buat decoder ISA table-driven untuk RISC-V/ARM-like
  core.
- [ ] CELL-03-05 Tambahkan cache interface dan memory wait state.
- [ ] CELL-03-06 Tambahkan MMU/MPU/TLB jika target berikutnya
  membutuhkan.
- [ ] CELL-03-07 Tambahkan debug unit, breakpoint, dan single-step.
- [ ] CELL-03-08 Buat satu contoh core RV32I atau Cortex-M-like dari
  spec.

Exit P1:

- ALU/shifter/multiplier/divider 32-bit lulus equivalence test.
- Register file 32 x 32 dan memory compiler dapat diparameterisasi.
- Satu core 32-bit sederhana dapat menjalankan program uji.

## P2 - Bus dan Infrastruktur MCU

- [ ] CELL-04-01 Buat bus internal sederhana dengan valid/ready.
- [ ] CELL-04-02 Buat APB bridge dan APB peripheral interface.
- [ ] CELL-04-03 Buat AHB-Lite master/slave interface.
- [ ] CELL-04-04 Buat AXI-Lite atau Wishbone jika target memilih.
- [ ] CELL-04-05 Buat address decoder dengan region dan alignment.
- [ ] CELL-04-06 Buat arbiter round-robin dan priority.
- [ ] CELL-04-07 Buat bridge width/protocol dan wait state.
- [ ] CELL-04-08 Buat error response, timeout, dan retry.
- [ ] CELL-04-09 Buat interrupt controller dengan mask, pending,
  priority, vector, dan nesting.
- [ ] CELL-04-10 Buat DMA controller dengan channel dan descriptor.
- [ ] CELL-04-11 Buat crossbar multi-master multi-slave.
- [ ] CELL-04-12 Buat register map generator dari daftar SFR.
- [ ] CELL-05-01 Buat PLL, clock divider, dan clock mux.
- [ ] CELL-05-02 Buat clock gate dan clock tree metadata.
- [ ] CELL-05-03 Buat CDC synchronizer, handshake, dan async FIFO.
- [ ] CELL-05-04 Buat power domain, isolation, retention, dan wakeup.
- [ ] CELL-05-05 Buat POR, BOD, dan watchdog reset.
- [ ] CELL-06-01 Buat timer generic multi-channel dengan prescaler,
  compare, capture, PWM, dan dead-time.
- [ ] CELL-06-02 Buat RTC dan watchdog generic.
- [ ] CELL-06-03 Buat UART generic dengan FIFO, parity, 9-bit, dan DMA
  hook.
- [ ] CELL-06-04 Buat SPI generic dengan FIFO, multi-mode, dan banyak
  chip select.
- [ ] CELL-06-05 Buat I2C generic dengan multi-master dan clock
  stretching.
- [ ] CELL-06-06 Buat GPIO bank, pin mux, dan alternate function matrix.
- [ ] CELL-06-07 Buat ADC/DAC controller dengan channel, trigger, dan
  calibration.
- [ ] CELL-06-08 Buat USB device/host behavioral dan Ethernet MAC
  behavioral.
- [ ] CELL-06-09 Buat CAN/CAN FD behavioral.
- [ ] CELL-06-10 Buat wireless behavioral stub untuk ESP32.

Exit P2:

- Satu MCU sintetis memiliki bus APB, GPIO, UART, timer, dan interrupt.
- Address map dan IRQ map tervalidasi dari spec.

## P3 - SoC Lanjutan

- [ ] CELL-07-01 Buat pad input, output, dan bidirectional.
- [ ] CELL-07-02 Buat open-drain, pull-up, pull-down, dan wired-AND.
- [ ] CELL-07-03 Buat pin mux matrix dan alternate function.
- [ ] CELL-07-04 Buat drive strength, slew rate, Schmitt trigger, dan
  level shifter.
- [ ] CELL-07-05 Buat pad digital ke analog melalui `device`.
- [ ] CELL-08-01 Tambahkan delay propagasi dan unit delay.
- [ ] CELL-08-02 Tambahkan setup/hold dan clock-to-Q.
- [ ] CELL-08-03 Tambahkan timestamp per net pada scheduler
  multi-clock. Bergantung pada CELL-00-04, bukan duplikat scheduler.
- [ ] CELL-08-04 Tambahkan state X/Z dan resolusi konflik.
- [ ] CELL-08-05 Tambahkan area, timing, dan power estimate per cell.
- [ ] CELL-08-06 Tambahkan waveform trace per net.
- [ ] CELL-08-07 Tambahkan hook equivalence dan property checking.
- [ ] CELL-09-01 Buat contoh `McuSpec` AVR.
- [ ] CELL-09-02 Buat contoh `McuSpec` STM32F1-like.
- [ ] CELL-09-03 Buat contoh `McuSpec` ESP32 behavioral.
- [ ] CELL-09-04 Buat test validator untuk spec valid dan invalid.
- [ ] CELL-09-05 Buat laporan fidelity per blok.
- [ ] CELL-09-06 Buat benchmark build dan simulasi per spec.

Exit P3:

- Satu MCU 32-bit dengan bus, timer, UART, GPIO, dan interrupt dapat
  dibangun dari spec.
- Fidelity setiap blok terlapor.
- Tidak ada perubahan pada entry katalog yang sudah stabil tanpa test.

## Urutan Pengerjaan yang Disarankan

1. CELL-00-01, CELL-00-02, CELL-00-03, CELL-00-06.
2. CELL-00-07, CELL-00-08, CELL-00-09.
3. CELL-01-01, CELL-01-02, CELL-01-03, CELL-02-04.
4. CELL-03-01, CELL-03-02, CELL-03-03, CELL-03-04.
5. CELL-04-01, CELL-04-02, CELL-04-05, CELL-04-09.
6. CELL-06-01, CELL-06-03, CELL-06-06, CELL-09-02.

## Vertikal Slice Pertama

Target yang disarankan: STM32F1-like atau RV32I sederhana.

Isi minimal:

- Core 32-bit dengan ALU, register file, decoder, dan trap.
- Bus APB dan address decoder.
- SRAM parameterized.
- GPIO, timer, UART.
- Interrupt controller.
- Spec dan address map.

Setelah slice ini stabil, baru pertimbangkan ESP32 dengan dual-core,
Wi-Fi/Bluetooth behavioral, dan bus yang lebih kompleks.

## Definition of Done

- [ ] Semua item P0 selesai.
- [ ] Semua item P1 selesai.
- [ ] P2 minimal APB, GPIO, UART, timer, dan interrupt selesai.
- [ ] Satu MCU 32-bit dapat dibangun dari spec.
- [ ] Spec invalid ditolak dengan error jelas.
- [ ] Fidelity setiap blok tercatat.
- [ ] Benchmark build dan simulasi tersedia.
- [ ] Tidak ada duplikasi dengan TODO-COMPONENTS atau TODO-SIMULATION.
- [ ] `fmt`, clippy, test, dan ASCII lulus.

## Catatan Konflik

Jika nanti ada item baru yang ternyata sudah tercakup di TODO lain:

- Jangan membuat item baru.
- Tambahkan rujukan silang.
- Pindahkan status ke TODO yang menjadi sumber kebenaran.
- TODO ini hanya memiliki item untuk library, generator, dan engine
  yang belum ada di TODO lain.
