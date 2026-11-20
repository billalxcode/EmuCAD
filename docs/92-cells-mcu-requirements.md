# 92 - Kebutuhan Cells untuk Multi-MCU

Tanggal: 2026-11-20
Status: analisis kebutuhan, belum implementasi.

## 1. Tujuan

`cells` perlu menjadi library digital yang cukup lengkap sehingga MCU
baru seperti AVR, 8051, PIC, STM32, ESP32, atau CPU lain dapat dibangun
dari spesifikasi, bukan dari kode khusus per chip.

Targetnya bukan langsung membuat semua MCU full gate-level. Targetnya:

1. Semua primitive digital memakai `cells`.
2. Blok yang dapat diparameterisasi memiliki spesifikasi yang jelas.
3. Model behavioral dan gate-level dapat dipilih per blok.
4. MCU baru cukup mendefinisikan spec, address map, pin map, dan
   peripheral yang dipakai.
5. Tidak ada duplikasi implementasi antara CPU, peripheral, dan
   katalog.

## 2. Ringkasan Status

`cells` saat ini sudah kuat untuk eksperimen CPU 8-bit sederhana dan
blok digital dasar, tetapi belum menjadi library SoC/MCU yang lengkap.

Yang sudah ada:

- Gate dasar, mux, buffer, transmission gate, tri-state inverter.
- D flip-flop, D latch, SRAM 6T, ring oscillator.
- Adder ripple-carry, ALU 8-bit, register, counter, shift register.
- Register file 32 x 8 dengan dua read port dan satu write port.
- Program counter dan stack pointer 16-bit.
- CPU mainan 16-bit dengan 32 register 8-bit dan ISA kecil.
- Digital circuit dua keadaan, zero-delay, satu clock ideal.
- Katalog digital untuk timer, UART, SPI, I2C, CAN, USB token,
  Ethernet, dan beberapa memory macro.

Yang belum memadai untuk STM32/ESP32:

- ALU 32-bit, barrel shifter, multiplier, divider, dan unit bit-manip.
- Bus fabric, address decoder, arbiter, bridge, wait state, DMA.
- Interrupt controller, CSR, exception, privilege, MMU/MPU.
- Multi-clock, PLL, clock divider, clock gating, reset synchronizer,
  CDC.
- FIFO, async FIFO, pipeline register, scoreboard, hazard unit.
- Memory compiler parameterized, byte enable, cache, parity/ECC.
- Pin mux, pad model, open-drain resolution, high-Z bus, pull-up/down.
- Spec MCU, validator address map, IRQ map, dan pin map.
- Model timing, propagation delay, setup/hold, dan multi-clock.

## 3. Lapisan yang Dibutuhkan

### 3.1 Primitive Cell

Sudah:

- Inverter, NAND2, NOR2, AND2, OR2, XOR2, MUX2.
- Buffer, transmission gate, tri-state inverter.
- D latch, D flip-flop, SRAM 6T, ring oscillator.

Sebagian:

- Drive strength dan ukuran transistor baru tersedia di level
  `CellBuilder`, belum menjadi varian cell standar.
- Tri-state inverter ada, tetapi `DigitalCircuit` belum memiliki
  resolusi high-Z atau bus contention.

Belum:

- Multi-input gate standar: NAND3/4, NOR3/4, AOI/OAI.
- Clock buffer, clock inverter, clock gate.
- Level shifter, isolation cell, retention flip-flop.
- Scan flip-flop, test cell, boundary scan.
- Open-drain buffer dengan resolusi wired-AND.
- Pad input/output, ESD, drive strength, slew rate.
- SRAM multi-port, dual-port, byte-write mask.

### 3.2 Combinational Block

Sudah:

- `and_all`, `or_all`, `mux_tree`, `decoder`, `read_bits`.
- Half adder, full adder, ripple-carry adder.
- Increment, decrement, ALU 8-bit.
- LUT dan finite state machine kecil di katalog digital.

Sebagian:

- Encoder, priority encoder, comparator, parity, dan beberapa blok
  lain hanya ada sebagai entry katalog, bukan builder library.
- Lebar operand masih banyak yang hardcoded 8-bit.
- Shift dan rotate baru berupa operasi byte, belum barrel shifter
  parameterized.

Belum:

- Barrel shifter 8/16/32/64-bit.
- Multiplier array, Booth multiplier, divider restored/non-restored.
- Leading-zero count, bit reverse, byte swap, saturating arithmetic.
- Comparator parameterized dengan flag less/equal/greater.
- Carry-lookahead adder dan prefix adder untuk jalur kritis.
- One-hot/priority encoder generic.
- Crossbar mux dan bus mux parameterized.
- Formal equivalence block untuk ALU dan shifter.

### 3.3 Sequential dan Memory

Sudah:

- Register width 1..64.
- Shift register width 1..64.
- Counter width 1..16.
- Register file parameterized `count x width` dengan 2 read dan 1 write.
- Program counter dan stack pointer 16-bit.
- SRAM 6T dan array 128 x 8 di katalog.

Sebagian:

- Register file belum memiliki byte enable, clear, preset, parity, atau
  multi-write port.
- Counter belum memiliki load, up/down, compare, capture, atau
  prescaler generic.
- Memory array masih fixed 128 x 8 dan berada di katalog, bukan
  generator.
- ROM, flash, EEPROM, FRAM, MRAM, dan NVRAM masih entry katalog.

Belum:

- FIFO sinkron dan async FIFO.
- Dual-clock RAM dan dual-port RAM.
- Memory compiler: word count, data width, byte enable, latency,
  initialization, read-during-write behavior.
- Cache tag/data array, valid/dirty bit, replacement metadata.
- Parity, ECC, scrubber, dan error injection.
- ROM/flash programming model dan erase/write timing.
- Cache line refill dan memory protection metadata.

### 3.4 CPU Core

Sudah:

- Register file 32 x 8.
- ALU 8-bit dengan operasi ADD/SUB/AND/OR/XOR/pass.
- PC 16-bit, instruction register 16-bit.
- CPU kecil dengan fetch/execute dua fase dan ISA mainan.
- Decoder opcode dan branch sederhana.

Sebagian:

- CPU ini adalah model demonstrasi, bukan implementasi ISA nyata.
- Belum ada pipeline, hazard unit, forwarding, atau branch prediction.
- Belum ada CSR, interrupt entry, exception, privilege, atau sleep.
- Belum ada 32-bit datapath atau register file 32 x 32.

Belum:

- ALU 32/64-bit dengan flag lengkap.
- Barrel shifter dan multiplier/divider 32-bit.
- Decoder ISA table-driven yang dapat dikonfigurasi per spec.
- Pipeline register dan hazard control.
- Instruction cache dan data cache interface.
- MMU/MPU, TLB, dan akses memori terproteksi.
- CSR file, trap controller, interrupt controller.
- Multi-core, coherent bus, atomic operation, dan memory ordering.
- Debug unit, breakpoint, trace, dan single-step.

### 3.5 Bus dan Interconnect

Sudah:

- Decoder one-hot.
- Mux tree untuk memilih data.
- Beberapa entry katalog yang memiliki address dan data bus.

Belum:

- Protokol bus: Wishbone, APB, AHB, AXI, TileLink, atau bus internal
  sederhana.
- Address decoder parameterized dengan region dan alignment.
- Bus arbiter, round-robin, priority, dan fairness.
- Bus bridge antar protokol dan lebar data.
- Wait state, error response, timeout, dan retry.
- DMA controller, descriptor engine, dan channel arbitration.
- Interrupt controller: mask, pending, priority, vector, nesting.
- Crossbar multi-master multi-slave.
- Register map generator untuk peripheral dan SFR.

### 3.6 Clock, Reset, dan Power

Sudah:

- Ring oscillator.
- Clock source digital di katalog.
- D flip-flop dengan clock ideal global.
- Reset global untuk seluruh `DigitalCircuit`.

Belum:

- PLL, clock divider, clock mux, clock gate.
- Multi-clock domain dan penjadwalan per domain.
- Reset synchronizer dan async assert/sync deassert.
- CDC synchronizer, handshake, dan async FIFO.
- Power domain, isolation, retention, dan wakeup.
- Brown-out detector, power-on reset, dan watchdog reset.
- Clock tree latency dan skew.

### 3.7 Peripheral

Sudah sebagai entry katalog:

- Timer, RTC, watchdog.
- UART 8-bit.
- SPI 8-bit.
- I2C master 7-bit address.
- CAN 2.0A transmitter.
- USB full-speed token transmitter.
- Ethernet entry.
- Memory macro SRAM/ROM/EEPROM/Flash/FRAM/MRAM/NVRAM.

Sebagian:

- Entry katalog fixed-width, fixed-feature, dan tidak memiliki API
  builder yang parameterized.
- UART belum punya FIFO, flow control, 9-bit, atau DMA.
- SPI belum punya multi-mode, FIFO, chip select banyak, atau DMA.
- I2C belum punya multi-master, clock stretching, atau FIFO.
- CAN belum punya receiver, filter, dan error handling penuh.
- USB baru token packet, belum device/host controller.
- Ethernet baru entry, belum MAC, DMA, atau PHY interface.
- Timer belum punya banyak channel, capture, compare, PWM, dan prescaler
  generic.

Belum:

- GPIO bank dengan pin mux dan alternate function matrix.
- ADC/DAC digital controller dengan channel, trigger, dan calibration.
- PWM multi-channel dengan dead-time dan complementary output.
- RTC calendrical, alarm, dan backup domain.
- CRC, RNG, AES, SHA, dan secure boot.
- USB device/host, Ethernet MAC, CAN FD, SDIO, dan camera interface.
- Wireless baseband Wi-Fi/Bluetooth untuk ESP32.
- Peripheral register file generator dengan SFR field metadata.

### 3.8 Pin, Pad, dan Analog Interface

Sudah:

- `cells::Cell` dengan port bernama, subcircuit, dan powered copy.
- Tri-state inverter.
- `avr::pin` punya model electrical analitik.

Sebagian:

- `DigitalCircuit` hanya dua keadaan: 0 dan 1. Belum ada high-Z, pull-up,
  pull-down, wired-AND, atau resolusi konflik.
- Belum ada pad model digital ke analog.

Belum:

- Pad input/output/bidirectional.
- Pin mux matrix.
- Alternate function mapping.
- Open-drain dan pull-up resolution di digital engine.
- Drive strength, slew rate, Schmitt trigger, dan ESD.
- Analog pin model yang memakai `device`.

### 3.9 Engine Simulasi Digital

Sudah:

- `DigitalBuilder` dan `DigitalCircuit`.
- Evaluasi topologis, komponen siklik, dirty tracking.
- `settle`, `tick`, `reset`, `stats`.
- Circuit dua keadaan, zero-delay, satu clock global.

Sebagian:

- `DigitalBuilder` datar; belum ada hierarchy module.
- Belum ada parameterized elaboration.
- Belum ada multi-clock dan event queue.

Belum:

- Delay propagation, setup/hold, clock-to-Q.
- Multi-clock, clock domain, dan phase.
- Asynchronous reset dan CDC.
- X/Z state dan bus contention.
- Power/ground dan level conversion.
- Waveform trace per net dengan timestamp.
- Formal equivalence dan property checking.
- Area/timing/power estimation per cell.

### 3.10 Spec dan Build

Belum:

- Schema `McuSpec`.
- Validator address map, IRQ map, pin map, dan memory map.
- Generator register file peripheral dari deskripsi SFR.
- Generator top-level interconnect dari daftar master/slave.
- Pemilihan fidelity per blok: behavioral, cycle-accurate, atau
  gate-level.
- Dependency graph dan urutan elaborasi.
- Snapshot/reproducible build untuk spec tertentu.

## 4. Matriks Status

| Kebutuhan | Sudah | Sebagian | Belum |
|---|---|---|---|
| Gate dasar | Ya | - | Varian drive strength |
| Arithmetic | Adder, ALU 8-bit | Comparator, encoder, LUT | 32/64-bit, divider, barrel shifter |
| Sequential | DFF, register, counter | Counter 1..16, PC/SP 16-bit | FIFO, CDC, pipeline |
| Memory | SRAM6T, array 128x8 | Register file 2R1W | Memory compiler, cache, ECC |
| CPU | CPU mainan 16-bit | ISA decoder kecil | Pipeline, CSR, exception, 32-bit |
| Bus | Decoder, mux | Entry katalog ber-address | APB/AHB/AXI, arbiter, DMA |
| Clock/reset | Ring oscillator | Clock source katalog | PLL, clock domain, reset sync |
| Peripheral | UART/SPI/I2C/CAN/USB token | Timer, memory macro | Pin mux, DMA, ADC/DAC, WiFi |
| Pin/pad | Tri-state inverter | Cell port | High-Z, pad, open-drain, mux |
| Engine | Zero-delay 2-state | Dirty tracking, stats | Timing, multi-clock, X/Z |
| Spec/build | - | - | McuSpec, validator, generator |

## 5. Prioritas Implementasi

### P0 - Fondasi

1. `DigitalVector` atau `Bus` dengan width, slice, concat, zero/sign
   extension.
2. Hierarchy module dan parameterized elaboration di
   `DigitalBuilder`.
3. High-Z/tristate resolution dan bus contention.
4. Multi-clock domain, clock gate, reset synchronizer.
5. Spec dasar: `McuSpec`, `BusSpec`, `MemorySpec`, `PeripheralSpec`,
   `PinSpec`, `ClockSpec`.
6. Central address map dan IRQ map.

### P1 - CPU dan Memory

1. ALU 32-bit dan 64-bit.
2. Barrel shifter, multiplier, divider.
3. Register file dengan byte enable dan multi-port.
4. Memory compiler parameterized.
5. Pipeline register, hazard unit, branch unit.
6. CSR/trap/interrupt untuk RISC-V/ARM-like core.

### P2 - MCU Infrastructure

1. Bus APB/AHB sederhana dan address decoder.
2. Interrupt controller, timer generic, GPIO bank, pin mux.
3. UART/SPI/I2C generic dengan FIFO dan DMA hook.
4. DMA controller.
5. RTC, watchdog, clock/power manager.
6. Peripheral register map generator.

### P3 - Advanced SoC

1. Cache, MPU/MMU, TLB.
2. USB, Ethernet, CAN FD, SDIO.
3. Multi-core, coherency, atomic.
4. Wireless behavioral model untuk ESP32.
5. Power domain dan security block.

## 6. Fidelity yang Disarankan

Jangan memaksa semua blok menjadi gate-level.

| Blok | Fidelity awal |
|---|---|
| CPU core | Cycle-accurate structural untuk blok inti |
| Register file, ALU, shifter | Gate-level `cells` |
| SRAM/flash kecil | Gate-level macro |
| Bus fabric | Cycle-accurate structural |
| Timer/GPIO/UART/SPI/I2C | Behavioral dengan register interface |
| USB/Ethernet/CAN | Transaction-level behavioral |
| WiFi/Bluetooth ESP32 | Behavioral atau black-box dengan event |
| Analog pin/ADC | `device` + solver analog |

Dengan pembagian ini, MCU baru dapat dibangun dari spec tanpa harus
membuat seluruh SoC menjadi netlist transistor.

## 7. Keputusan yang Dibutuhkan

1. Apakah target pertama adalah STM32/ESP32 full gate-level, atau
   CPU core structural dengan peripheral behavioral?
2. Apakah `cells` perlu memiliki `McuSpec` dan generator, atau spec
   builder ditempatkan di crate baru seperti `soc`?
3. Apakah high-Z dan multi-clock harus masuk ke `DigitalCircuit` yang
   sama, atau dibuat engine terpisah?
4. Apakah `cells` boleh bergantung pada `device` untuk pad/analog,
   atau batasnya harus tetap murni digital?

Rekomendasi: mulai dari P0 dan P1 dengan fidelity campuran. Full
gate-level ESP32/STM32 bukan target realistis untuk fase berikutnya.
