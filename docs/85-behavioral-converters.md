# 85 - Behavioral Converter

Dokumen ini mencatat model behavioral untuk 4 entry converter 4-pin:
sample-and-hold, SAR ADC, delta-sigma ADC, dan PWM DAC. R-2R DAC
belum didukung karena membutuhkan bus multi-bit.

## 1. Pin

Sample-and-hold dan ADC:

| Pin | Fungsi |
|---:|---|
| 0 | input analog |
| 1 | control atau clock |
| 2 | output |
| 3 | ground |

PWM DAC:

| Pin | Fungsi |
|---:|---|
| 0 | input PWM |
| 1 | output |
| 2 | supply |
| 3 | ground |

## 2. Sample-and-Hold

Jika control melewati `threshold`, output mengikuti input. Jika tidak,
output menahan nilai terakhir.

```
Vout = gain * held
```

Default `threshold` 2.5 V dan `gain` 1.

## 3. SAR ADC

Pada tepi naik clock, input dikuantisasi:

```
code = round(input / full_scale * (2^resolution - 1))
Vout = code * full_scale / (2^resolution - 1)
```

Default `resolution` 8 bit dan `full_scale` 5 V. Output berupa tegangan
analog yang merepresentasikan kode, bukan bus digital.

## 4. Delta-sigma ADC

Model memakai filter orde satu sebelum kuantisasi:

```
filtered += (input - filtered) * 0.1
Vout = quantize(filtered)
```

Kuantisasi dilakukan pada tepi naik clock. Default `resolution` 8 bit
dan `full_scale` 5 V.

## 5. PWM DAC

Model adalah low-pass filter dari input PWM:

```
state += (input - state) * dt / (time_constant + dt)
Vout = gain * state
```

Pada DC, output sama dengan `gain * input`. Default `time_constant`
1 ms dan `gain` 1.

## 6. R-2R DAC

Entry R-2R DAC tetap unsupported. DAC resistor ladder membutuhkan
banyak pin digital atau bus, sedangkan katalog hanya menyediakan 4 pin.
Model 1-bit akan menyesatkan.

## 7. Referensi

- Sample and hold, dasar penyimpanan nilai analog:
  https://en.wikipedia.org/wiki/Sample_and_hold
- Analog-to-digital converter, sampling dan kuantisasi:
  https://en.wikipedia.org/wiki/Analog-to-digital_converter
- Delta-sigma modulation, oversampling dan noise shaping:
  https://en.wikipedia.org/wiki/Delta-sigma_modulation
- Resistor ladder, dasar DAC R-2R:
  https://en.wikipedia.org/wiki/Resistor_ladder
- Pulse-width modulation, dasar PWM DAC:
  https://en.wikipedia.org/wiki/Pulse-width_modulation

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 8. Batas

- ADC mengeluarkan tegangan analog, bukan bus digital.
- Tidak ada sample aperture, jitter, atau noise.
- Delta-sigma bukan modulator bitstream penuh.
- PWM DAC menganggap input sudah berupa tegangan PWM.
- Belum ada reference vector atau status `calibrated`.
