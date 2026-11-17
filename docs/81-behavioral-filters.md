# 81 - Behavioral Filter

Dokumen ini mencatat model behavioral untuk 5 entry filter 4-pin:
integrator, differentiator, low-pass, high-pass, dan state-variable.

## 1. Pin

| Pin | Fungsi |
|---:|---|
| 0 | input |
| 1 | output utama |
| 2 | ground |
| 3 | band-pass output untuk state-variable, ground untuk filter lain |

Semua output memakai Thevenin dengan `output_resistance` 1000 ohm dan
batas state `output_limit` 15 V.

## 2. Integrator

```
state += input * dt / time_constant
Vout = gain * state
```

Pada DC, output 0. Integrator bersifat non-inverting dan state dibatasi
oleh `output_limit`.

## 3. Differentiator

```
Vout = gain * time_constant * (input - previous_input) / dt
```

Pada DC, output 0. Derivative dihitung dari selisih antar time step.

## 4. Low-pass dan High-pass

Low-pass memakai backward Euler:

```
state += (input - state) * dt / (time_constant + dt)
Vout = gain * state
```

High-pass memakai state low-pass internal:

```
low += (input - low) * dt / (time_constant + dt)
Vout = gain * (input - low)
```

Pada DC, low-pass meneruskan `gain * input` dan high-pass menghasilkan 0.

## 5. State-variable Filter

```
low += band * dt / time_constant
band += (input - low - damping * band) * dt / time_constant
```

Output utama adalah low-pass dan pin 3 adalah band-pass. Default
`time_constant` 1 ms dan `damping` 0.7.

## 6. Referensi

- Low-pass filter, respons frekuensi dan time constant:
  https://en.wikipedia.org/wiki/Low-pass_filter
- High-pass filter, respons terhadap DC dan frekuensi rendah:
  https://en.wikipedia.org/wiki/High-pass_filter
- Integrator, akumulasi sinyal terhadap waktu:
  https://en.wikipedia.org/wiki/Integrator
- Differentiator, turunan sinyal terhadap waktu:
  https://en.wikipedia.org/wiki/Differentiator
- State-variable filter, low-pass dan band-pass dari state yang sama:
  https://en.wikipedia.org/wiki/State_variable_filter

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 7. Batas

- Tidak ada noise, offset, atau nonlinearitas.
- Tidak ada frekuensi cutoff eksplisit; cutoff diturunkan dari
  `time_constant`.
- Integrator dapat drift dan dibatasi oleh `output_limit`.
- Belum ada reference vector atau status `calibrated`.
