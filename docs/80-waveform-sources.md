# 80 - Waveform dan Noise Source

Dokumen ini mencatat model behavioral untuk pulse current, exponential
voltage, PWL voltage, dan dua noise source deterministik. Semua model
memakai dua pin.

## 1. Pulse Current Source

Memakai model pulse yang sama dengan pulse voltage source, tetapi
menghasilkan arus:

```
I(t) = v1 sebelum delay
I(t) = v2 selama width
I(t) = v1 setelah fall
```

Parameter `v1` dan `v2` dalam ampere. Default `v2` 1 mA, period 2 us,
width 1 us, rise dan fall 1 ns.

## 2. Exponential Voltage Source

```
t < td1: v1
td1 <= t < td2: v1 + (v2 - v1) * (1 - exp(-(t - td1) / tau1))
t >= td2: peak + (v1 - v2) * (1 - exp(-(t - td2) / tau2))
```

Default `v1` 0, `v2` 5 V, `td1` 0, `tau1` 1 ms, `td2` 1 ms, dan
`tau2` 1 ms. `td2` tidak boleh lebih awal dari `td1`.

## 3. PWL Voltage Source

Model mendukung empat titik `(t0, v0)` sampai `(t3, v3)` dengan
interpolasi linier. Sebelum titik pertama output `v0`; setelah titik
terakhir output `v3`. Waktu harus tidak menurun.

Default: `(0, 0)`, `(1 ms, 5 V)`, `(2 ms, 5 V)`, dan `(3 ms, 0 V)`.

Batas: entry PWL generik dengan lebih dari empat titik belum
didukung.

## 4. Noise Voltage dan Current Source

Model memakai xorshift64 dengan seed eksplisit:

```
state = seed
state = xorshift64(state)
V = amplitude * (2 * state / 2^53 - 1)
```

Nilai dipertahankan konstan selama satu time step dan diperbarui
setelah update. Seed yang sama menghasilkan urutan yang sama; seed
berbeda menghasilkan urutan berbeda. Default amplitude 1 mV untuk
voltage dan 1 uA untuk current.

Batas: noise bersifat white dan uniform, bukan Gaussian, dan tidak
memiliki bandwidth atau spectral density.

## 5. Referensi

- Exponential decay, dasar perubahan eksponensial:
  https://en.wikipedia.org/wiki/Exponential_decay
- Piecewise linear function, interpolasi antar titik:
  https://en.wikipedia.org/wiki/Piecewise_linear_function
- Noise in electronics, fluktuasi acak pada sinyal:
  https://en.wikipedia.org/wiki/Noise_(electronics)
- Pseudorandom number generator, seed dan reproducibility:
  https://en.wikipedia.org/wiki/Pseudorandom_number_generator

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 6. Batas Umum

- PWL hanya empat titik.
- Noise bukan model fisik dan tidak memiliki spektrum terkalibrasi.
- Tidak ada behavioral expression, polynomial, atau table source.
- Belum ada reference vector atau status `calibrated`.
