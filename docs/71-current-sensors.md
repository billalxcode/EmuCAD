# 71 - Sensor Arus dan Tegangan

Dokumen ini mencatat model behavioral untuk 3 entry sensor arus dan
tegangan 3-pin. Semua model memakai pin VCC, GND, dan OUT. Output
dibatasi 0 sampai tegangan VCC.

## 1. Current Sense Resistor

Shunt resistor mengubah arus menjadi tegangan menurut Hukum Ohm:

```
Vshunt = current * shunt_resistance
Vout = clamp(offset + gain * Vshunt, 0, VCC)
```

Default `shunt_resistance` 0.1 ohm, `gain` 20 V/V, dan `offset` 0.0.
Pada 1 A, Vshunt 0.1 V dan output 2.0 V pada supply 5 V.

Batas: disipasi daya shunt, induktansi parasit, dan isolasi tidak
dimodelkan. Model menganggap amplifier penguat sudah berada di dalam
modul.

## 2. Hall Current Sensor

Model memakai respons linier terhadap arus:

```
Vout = clamp(offset + sensitivity * current, 0, VCC)
```

Default `sensitivity` 0.1 V/A dan `offset` 2.5 V. Pada 10 A, output
3.5 V pada supply 5 V. Sensor Hall memberi isolasi elektrik antara
konduktor primer dan output.

Batas: saturasi inti, bandwidth, noise, drift suhu, dan medan
eksternal tidak dimodelkan. Model hanya satu arah arus.

## 3. Voltage Divider Sensor

Pembagi tegangan menghasilkan sebagian dari tegangan input:

```
Vout = clamp(offset + sensitivity * voltage, 0, VCC)
```

Dengan pembagi resistor, `sensitivity` sama dengan
R2 / (R1 + R2). Default 0.2 V/V, sehingga 10 V menghasilkan 2.0 V
pada supply 5 V. Batas atas sensitivitas 1.0 V/V.

Batas: efek pembebanan output tidak dihitung sebagai fungsi beban;
model memakai resistor output Thevenin 1000 ohm. Toleransi resistor,
koefisien suhu, dan respons frekuensi tidak dimodelkan.

## 4. Referensi

- Current sensing, shunt resistor dan sensor Hall untuk pengukuran
  arus:
  https://en.wikipedia.org/wiki/Current_sensing
- Hall-effect sensor, output proporsional terhadap medan dan arus:
  https://en.wikipedia.org/wiki/Hall-effect_sensor
- Voltage divider, rasio Vout / Vin dan efek pembebanan:
  https://en.wikipedia.org/wiki/Voltage_divider

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 5. Batas Umum

- Tidak ada noise, drift suhu, atau nonlinearitas.
- Tidak ada bandwidth, waktu respons, atau saturasi.
- Tidak ada isolasi galvanik fisik di netlist.
- Output adalah Thevenin ideal dengan resistor output 1000 ohm.
- Belum ada reference vector atau status `calibrated`.
