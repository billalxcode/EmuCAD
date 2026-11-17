# 84 - Behavioral Oscillator

Dokumen ini mencatat model behavioral untuk 4 entry oscillator 4-pin:
VCO, amplitude modulator, frequency modulator, dan PWM. PLL belum
didukung.

## 1. Pin

| Pin | Fungsi |
|---:|---|
| 0 | input kontrol atau modulasi |
| 1 | output |
| 2 | supply |
| 3 | ground |

Output adalah Thevenin dengan `output_resistance` 1000 ohm dan dibatasi
0 sampai tegangan supply.

## 2. VCO

```
frequency = frequency + sensitivity * (Vctrl - control_center)
Vout = square(frequency, duty 0.5)
```

Default `frequency` 1 kHz, `sensitivity` 1000 Hz/V, dan
`control_center` 2.5 V. Frekuensi dibatasi minimal nol.

## 3. Amplitude Modulator

```
envelope = 1 + sensitivity * (Vmod - control_center)
Vout = offset + amplitude * envelope * sin(2*pi*frequency*t)
```

Default `frequency` 1 kHz, `sensitivity` 1 per V, `amplitude` 1 V, dan
`offset` 0 V. Output di-clamp ke rentang supply.

## 4. Frequency Modulator

```
frequency(t) = frequency + sensitivity * (Vmod - control_center)
Vout = offset + amplitude * sin(phase(t))
```

Phase diakumulasi dari frekuensi sesaat. Default `sensitivity`
1000 Hz/V dan `control_center` 2.5 V.

## 5. PWM

```
duty = 0.5 + sensitivity * (Vctrl - control_center)
Vout = high jika phase < duty, selain itu low
```

Default `sensitivity` 0.1 per V dan `control_center` 2.5 V. Duty
dibatasi 0.01 sampai 0.99.

## 6. Referensi

- Voltage-controlled oscillator, frekuensi oleh tegangan kontrol:
  https://en.wikipedia.org/wiki/Voltage-controlled_oscillator
- Amplitude modulation, envelope dan carrier:
  https://en.wikipedia.org/wiki/Amplitude_modulation
- Frequency modulation, phase dari frekuensi sesaat:
  https://en.wikipedia.org/wiki/Frequency_modulation
- Pulse-width modulation, duty cycle:
  https://en.wikipedia.org/wiki/Pulse-width_modulation

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 7. Batas

- Tidak ada phase noise, jitter, atau harmonic.
- Modulator memakai satu tone dan kontrol analog.
- PLL belum didukung karena membutuhkan phase detector dan loop filter.
- Belum ada reference vector atau status `calibrated`.
