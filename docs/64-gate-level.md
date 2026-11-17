# 64 - Gate-Level Scope dan Status

Dokumen ini mendefinisikan arti gate-level, mencatat komponen structural
yang sudah ada, dan menetapkan gap untuk S6 dan S7.

## 1. Definisi

Gate-level berarti:

- Logika dua keadaan dari gerbang Boolean.
- Flip-flop dan latch sebagai sel digital.
- Memory macro dari sel digital, bukan transistor.
- Interface dan timing digital.
- Peripheral digital yang dapat diaudit sebagai rangkaian gerbang.

Gate-level bukan transistor-level dan bukan TCAD. Model transistor
seperti `sram6t` di `sim-cells` adalah primitif analog, bukan target
utama S6.

## 2. Yang Sudah Ada

- `sim-cells` menyediakan gerbang dasar, adder, mux, decoder, counter,
  shift register, register file, dan CPU digital.
- `sim-cells::digital` menyediakan `DigitalCircuit` dengan evaluasi
  kombinasional dan clock DFF.
- `sim-cells::sequential::sram6t` menyediakan sel SRAM 6T level
  transistor sebagai referensi.
- `sim-cells::catalog_memory` menyediakan sel SRAM digital 1-bit dan
  array 128x8 (1024 bit) untuk domain digital.
- `sim-avr` memakai `sim-cells` untuk datapath, register bank, instruksi,
  ALU, shift, multiply, dan beberapa GPIO.
- `sim-board` menyediakan board AVR + LED + resistor + button + UART.

## 3. Batas AVR Saat Ini

AVR belum full-chip gate-level. Yang masih hybrid:

- Kontrol instruksi dan decode.
- Timer, UART, SPI, TWI, ADC, dan USB.
- Integrasi analog-digital.
- Flash dan SRAM sebagai memory structural penuh.

Karena itu klaim "AVR full chip gate-level" belum boleh dipakai.

## 4. Benchmark

Contoh hasil `avr_bench` pada desktop x86_64:

| Program | Realtime |
|---|---:|
| ALU | 3.185x |
| Mixed | 2.921x |
| Memory | 4.850x |
| Timer | 2.538x |
| GPIO | 3.584x |

Register file gate-level 32x8:

- 992 gate, 256 DFF.
- 165407 operasi/detik.
- 6046 ns per tulis + clock + baca + settle.

Pada 16 MHz, satu siklus adalah 62.5 ns. Memory macro gate-level masih
jauh di atas budget jika setiap akses disimulasikan penuh.

## 5. Gap Matrix

| Item | Target | Status |
|---|---|---|
| MCU gate-level | 11 | 1 hybrid (AVR) |
| Memory gate-level | 15 | 1 register file dan 7 macro memory |
| AVR full chip | Semua peripheral | Sebagian |
| Mixed-signal scheduler | AVR + analog | Belum |
| Real-time semua MCU | Desktop | Belum terbukti |

MCU yang belum dimulai: PIC, 8051, ARM Cortex-M, RISC-V, ESP32, STM32,
RP2040, dan MCU 32-bit generik.

Memory yang belum dimulai: DRAM, ROM/PROM, EEPROM, Flash, FRAM, MRAM,
NVRAM, controller, module, dan sel memory lain.

## 6. Keputusan

- Q55 dipakai: target real-time dipertahankan sebagai target, bukan
  jaminan.
- S6 dan S7 tidak diklaim selesai sampai model gate-level dan
  benchmark tersedia.
- Pekerjaan berikutnya harus dimulai dari satu vertical slice:
  AVR gate-level penuh + SRAM macro + GPIO blink.
- Setelah vertical slice stabil, baru menambah MCU dan memory lain.

## 7. Kriteria Selesai S6/S7

- Netlist gate-level dapat diaudit untuk setiap MCU dan memory.
- Ada instruction suite, peripheral suite, dan timing test.
- Ada benchmark build dan simulasi.
- Mixed-signal scheduler deterministik.
- Semua klaim real-time disertai angka dan batas hardware.
