# 88 - Domain Digital

Dokumen ini mencatat domain digital minimal di `sim-cells` dan perintah
`emucad digital`.

## 1. Cakupan

Domain digital memakai `DigitalCircuit` dua-state dengan input dan
output bernama. Saat ini 13 entry katalog sudah dapat dijalankan:

| Kelompok | Entry |
|---|---|
| Multiplexer | 2:1, 4:1 |
| Adder | half adder, full adder, 4-bit adder |
| Flip-flop | D, JK, T, SR |
| Counter | 4-bit counter, decade counter, Johnson counter |
| Shift register | 4-bit shift register |

## 2. Perintah

```
emucad digital <id> [nama=0|1]... [--ticks N]
```

Contoh:

```
emucad digital logic.adder.4bit_adder a0=1 a1=1 b0=1 b2=1
emucad digital logic.counter.4bit_counter --ticks 3
emucad digital logic.flipflop.d_flipflop d=1 clk=1 --ticks 1
```

Tanpa `--ticks`, rangkaian hanya di-settle satu kali. Dengan
`--ticks N`, clock global dijalankan N kali.

## 3. Port

Port ditentukan per entry, bukan dari jumlah pin katalog. Contoh:

- `logic.adder.4bit_adder`: input `a0..a3`, `b0..b3`, `cin`; output
  `s0..s3`, `cout`.
- `logic.counter.4bit_counter`: input `reset`; output `q0..q3`.
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

Masih ada 50 entry digital di luar domain ini:

- 3 special gate (open-drain, tri-state, transmission gate).
- 6 flip-flop tambahan (latch, edge-triggered, master-slave, multi-bit).
- 6 counter/shift tambahan (8-bit, up/down, ripple, SIPO, PISO).
- 6 mux/decoder/encoder tambahan.
- 5 adder/comparator/parity/buffer tambahan.
- 6 level shifter/interface.
- 6 programmable logic.
- 6 serial interface.
- 3 timer peripheral.
- 3 digital source.

## 6. Verifikasi

Test domain digital mencakup:

- Select mux 2:1.
- Penjumlahan 3 + 5 pada 4-bit adder.
- Capture D flip-flop pada tepi naik.
- Counter 4-bit setelah tiga tick.
- Decade counter kembali ke nol setelah sepuluh tick.
- Shift register memindahkan data.

## 7. Referensi

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
