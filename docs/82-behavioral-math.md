# 82 - Behavioral Math

Dokumen ini mencatat model behavioral untuk 5 entry math 4-pin:
multiplier, divider, log amplifier, antilog amplifier, dan RMS
converter.

## 1. Pin

| Pin | Fungsi |
|---:|---|
| 0 | input 1 |
| 1 | input 2 |
| 2 | output |
| 3 | ground |

Output adalah Thevenin dengan `output_resistance` 1000 ohm dan batas
`output_limit` 15 V.

## 2. Persamaan

```
multiplier: Vout = gain * V1 * V2
divider:    Vout = gain * V1 / V2
log:        Vout = gain * ln(V1 / reference)
antilog:    Vout = gain * reference * exp(V1 / reference)
RMS:        Vout = gain * sqrt(mean(V1^2))
```

- Divider mengembalikan batas output jika V2 mendekati nol.
- Log mengembalikan batas negatif jika V1 tidak positif.
- Antilog membatasi eksponen agar tidak overflow.
- RMS memakai state `mean_square` dengan `time_constant`.

Default `gain` 1, `reference` 1 V, `time_constant` 1 ms,
`output_limit` 15 V, dan `output_resistance` 1000 ohm.

## 3. RMS pada DC dan Transient

Pada DC, RMS mengembalikan `gain * |V1|`. Pada transient, model
memperbarui mean square:

```
mean_square += (V1^2 - mean_square) * dt / (time_constant + dt)
```

Nilai awal mean square adalah nol.

## 4. Referensi

- Multiplier, perkalian sinyal analog:
  https://en.wikipedia.org/wiki/Analog_multiplier
- Divider, pembagian sinyal:
  https://en.wikipedia.org/wiki/Voltage_divider
- Logarithmic amplifier, output logaritmik:
  https://en.wikipedia.org/wiki/Log_amplifier
- Log amplifier, termasuk konfigurasi antilog:
  https://en.wikipedia.org/wiki/Log_amplifier
- Root mean square, nilai efektif:
  https://en.wikipedia.org/wiki/Root_mean_square

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 5. Batas

- Tidak ada noise, offset, bandwidth, atau suhu.
- Divider dan log memakai proteksi numerik, bukan proteksi rangkaian.
- RMS adalah low-pass dari kuadrat sinyal, bukan true RMS converter.
- Belum ada reference vector atau status `calibrated`.
