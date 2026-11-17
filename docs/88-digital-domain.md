# 88 - Domain Digital

Dokumen ini mencatat domain digital minimal di `sim-cells` dan perintah
`emucad digital`.

## 1. Cakupan

Domain digital memakai `DigitalCircuit` dua-state dengan input dan
output bernama. Saat ini 56 entry katalog sudah dapat dijalankan:

| Kelompok | Entry |
|---|---|
| Multiplexer | 2:1, 4:1, 8:1 |
| Demux/decoder/encoder | 1:2 demux, 2:4 decoder, 3:8 decoder, 8:3 encoder, priority encoder |
| Adder/comparator | half adder, full adder, 4-bit adder, magnitude comparator |
| Parity/buffer | parity generator, parity checker, bus buffer |
| Flip-flop | D, JK, T, SR, D latch, edge D, master-slave JK, dual/quad D, octal latch |
| Counter | 4-bit, 8-bit, decade, up/down, ripple, Johnson |
| Shift register | 4-bit, 8-bit, SIPO, PISO |
| Level | level shifter, line driver, line receiver, optocoupler interface |
| Timer | timer, RTC, watchdog |
| Digital source | clock, PRBS, pattern |
| Programmable | PAL, GAL, CPLD, FPGA logic block, lookup table (LUT 4-input) |
| High-Z | open-drain, tri-state, transmission gate, bidirectional buffer, bidirectional/open-drain level |

## 2. Perintah

```
emucad digital <id> [nama=0|1]... [--ticks N] [--pattern V] [--mask V]
```

Contoh:

```
emucad digital logic.adder.4bit_adder a0=1 a1=1 b0=1 b2=1
emucad digital logic.counter.4bit_counter --ticks 3
emucad digital logic.flipflop.d_flipflop d=1 clk=1 --ticks 1
emucad digital logic.programmable.lookup_table a0=1 a1=1 --mask 8
emucad digital source.digital.digital_pattern_source reset=1 --pattern 1 --ticks 1
```

`--pattern` dan `--mask` menerima desimal atau hex `0x`. LUT memakai
empat input `a0..a3` dan mask 16 bit; bit ke-i adalah output untuk
kombinasi input i.

Tanpa `--ticks`, rangkaian hanya di-settle satu kali. Dengan
`--ticks N`, clock global dijalankan N kali.

## 3. Port

Port ditentukan per entry, bukan dari jumlah pin katalog. Contoh:

- `logic.adder.4bit_adder`: input `a0..a3`, `b0..b3`, `cin`; output
  `s0..s3`, `cout`.
- `logic.counter.4bit_counter`: input `reset`; output `q0..q3`.
- `logic.mux.8_to_1_multiplexer`: input `in0..in7`, `sel0..sel2`;
  output `y`.
- `logic.adder.magnitude_comparator`: input `a0..a3`, `b0..b3`;
  output `eq`, `gt`, `lt`.
- `logic.flipflop.d_flipflop`: input `d`, `clk`; output `q`, `qn`.

## 4. Batas

- Hanya dua-state 0/1; tidak ada high-Z, drive strength, atau delay.
- Clock flip-flop adalah clock global dari `tick`, bukan clock net.
- D flip-flop memakai deteksi tepi naik dengan register clock internal.
- Counter dan shift register aktif pada setiap tick.
- Tidak ada timing, setup/hold, atau propagation delay.
- Serial interface, programmable logic, level shifter, dan digital
  source belum didukung.

## 5. Entry Digital yang Belum Didukung

Masih ada 7 entry digital di luar domain ini:

- 1 programmable logic (finite state machine).
- 6 serial interface (UART, SPI, I2C, CAN, USB, Ethernet).

## 7. Verifikasi

Test domain digital mencakup:

- Select mux 2:1 dan mux 8:1.
- Decoder 3:8 dan priority encoder.
- Penjumlahan 3 + 5 pada 4-bit adder.
- Comparator 5 > 3.
- Parity checker mendeteksi error.
- Bus buffer mengikuti enable.
- D flip-flop dan dual D flip-flop menangkap pada tepi naik.
- D latch menahan nilai saat enable rendah.
- Up/down counter menghitung dua arah.
- PISO menggeser data parallel ke serial.
- Watchdog timeout setelah 255 tick.
- Clock source toggle setiap tick.
- Open-drain dan tri-state memakai sinyal `z` untuk kondisi released.
- Bidirectional buffer mengarahkan satu sisi dan melepas sisi lain.
- PRBS source bergeser setelah reset.
- Pattern source memuat pola saat reset.
- LUT 4-input dengan mask AND menghasilkan output yang benar.
- Counter 4-bit setelah tiga tick.
- Decade counter kembali ke nol setelah sepuluh tick.
- Shift register memindahkan data.

## 8. Referensi

- Logic gate, dasar gerbang dua-state:
  https://en.wikipedia.org/wiki/Logic_gate
- Flip-flop, latch dan edge triggering:
  https://en.wikipedia.org/wiki/Flip-flop_(electronics)
- Counter, rangkaian pencacah:
  https://en.wikipedia.org/wiki/Counter_(digital)
- Multiplexer, pemilihan input:
  https://en.wikipedia.org/wiki/Multiplexer
- Adder, penjumlahan biner:
  https://en.wikipedia.org/wiki/Adder_(electronics)
