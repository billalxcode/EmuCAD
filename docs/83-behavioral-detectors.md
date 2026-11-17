# 83 - Behavioral Detector

Dokumen ini mencatat model behavioral untuk 5 entry detector 4-pin:
comparator, Schmitt trigger, peak detector, track-and-hold, dan
precision rectifier.

## 1. Pin

| Pin | Fungsi |
|---:|---|
| 0 | input |
| 1 | reference atau control |
| 2 | output |
| 3 | ground |

Output adalah Thevenin dengan `output_resistance` 1000 ohm.

## 2. Comparator

```
Vout = input > reference ? output_high : output_low
```

Default `output_high` 5 V dan `output_low` 0 V. Comparator tidak
memiliki hysteresis.

## 3. Schmitt Trigger

```
on  jika input > upper_threshold
off jika input < lower_threshold
```

Default `upper_threshold` 3 V dan `lower_threshold` 2 V. Output
memakai `output_high` dan `output_low`.

## 4. Peak Detector

State `peak` menyimpan nilai input tertinggi. Jika pin reference
melewati `reset_threshold`, state kembali ke nol.

```
Vout = gain * peak
```

Default `reset_threshold` 2.5 V dan `output_limit` 15 V.

## 5. Track-and-Hold

Jika pin reference melewati `threshold`, output mengikuti input. Jika
tidak, output menahan nilai terakhir.

```
Vout = gain * held
```

Default `threshold` 2.5 V.

## 6. Precision Rectifier

```
Vout = gain * max(input, 0)
```

Input negatif menghasilkan 0. Default `gain` 1 dan `output_limit`
15 V.

## 7. Referensi

- Comparator, pembanding dua tegangan:
  https://en.wikipedia.org/wiki/Comparator
- Schmitt trigger, comparator dengan hysteresis:
  https://en.wikipedia.org/wiki/Schmitt_trigger
- Envelope detector, dasar deteksi puncak:
  https://en.wikipedia.org/wiki/Envelope_detector
- Sample and hold, dasar track-and-hold:
  https://en.wikipedia.org/wiki/Sample_and_hold
- Precision rectifier, penyearah presisi:
  https://en.wikipedia.org/wiki/Precision_rectifier

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 8. Batas

- Comparator dan Schmitt ideal tanpa delay atau slew rate.
- Peak detector tidak memiliki droop atau reset timing.
- Track-and-hold tidak memiliki aperture time atau droop.
- Precision rectifier tidak memiliki offset atau bandwidth.
- Belum ada reference vector atau status `calibrated`.
