# 78 - Sine Source

Dokumen ini mencatat model behavioral untuk sine voltage source dan
sine current source. Keduanya memakai dua pin.

## 1. Persamaan

```
V(t) = offset + amplitude * sin(2*pi*frequency*t + phase)
I(t) = offset + amplitude * sin(2*pi*frequency*t + phase)
```

`phase` dalam derajat. Default `amplitude` 5 V untuk voltage source
dan 1 mA untuk current source, `frequency` 1 kHz, `phase` 0, dan
`offset` 0.

Pada transient, waktu maju mengikuti `dt`. Pada DC operating point,
nilai yang dipakai adalah nilai pada t = 0.

## 2. Implementasi

- Voltage source memakai branch tegangan sehingga output dapat
  bernilai negatif.
- Current source memakai current source MNA.
- `terminal_current` tersedia untuk probe.

## 3. Batas

- Tidak ada distortion, noise, atau frekuensi non-sinus.
- Tidak ada amplitude drift atau phase noise.
- Tidak ada parameter rise/fall atau modulasi.
- Current source ideal tidak memiliki compliance voltage.
- Model hanya mendukung satu tone.

## 4. Referensi

- Sine wave, bentuk gelombang sinus dan phase:
  https://en.wikipedia.org/wiki/Sine_wave
- Alternating current, arus yang berubah arah secara periodik:
  https://en.wikipedia.org/wiki/Alternating_current
- Root mean square, nilai efektif gelombang sinus:
  https://en.wikipedia.org/wiki/Root_mean_square

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 5. Batas Umum

- Belum ada reference vector atau status `calibrated`.
- Model valid untuk analisis transient dan AC satu tone.
- Entry PWL, exponential, noise, dan behavioral source belum
  didukung.
