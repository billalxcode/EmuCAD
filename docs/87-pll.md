# 87 - PLL

Dokumen ini mencatat model behavioral untuk timer PLL 8-pin dan
behavioral PLL 4-pin. Model bekerja sebagai frequency multiplier
dengan phase reset pada tepi input.

## 1. Pin

Timer PLL 8-pin:

| Pin | Fungsi |
|---:|---|
| 0 | VCC |
| 1 | GND |
| 2 | output |
| 4 | input referensi |

Behavioral PLL 4-pin:

| Pin | Fungsi |
|---:|---|
| 0 | input referensi |
| 1 | output |
| 2 | supply |
| 3 | ground |

Pin lain pada timer PLL tidak dipakai.

## 2. Persamaan

```
periode_input = waktu antara dua tepi naik
fout = multiplier / periode_input
```

Jika belum ada tepi input, output memakai `frequency` sebagai free-run.
Pada setiap tepi naik input, phase output di-reset ke nol dan output
high sampai setengah periode output.

Default `multiplier` 2, `frequency` 1 kHz, dan `threshold` 2.5 V.
Output adalah gelombang kotak 50 persen dengan `output_resistance`
1000 ohm.

## 3. Batas

- Tidak ada phase detector, charge pump, atau loop filter.
- Tidak ada lock range, capture range, atau jitter.
- Input harus berupa sinyal kotak yang melewati threshold.
- Output hanya frequency multiplier dengan phase reset.
- Belum ada reference vector atau status `calibrated`.

## 4. Referensi

- Phase-locked loop, konsep loop dan phase:
  https://en.wikipedia.org/wiki/Phase-locked_loop
- Frequency multiplier, kelipatan frekuensi referensi:
  https://en.wikipedia.org/wiki/Frequency_multiplier

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.
