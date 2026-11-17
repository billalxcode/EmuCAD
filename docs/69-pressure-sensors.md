# 69 - Sensor Tekanan dan Gaya

Dokumen ini mencatat model behavioral untuk 4 entry sensor tekanan,
gaya, dan strain. Semua model memakai pin VCC, GND, dan OUT. Output
dibatasi 0 sampai tegangan VCC.

## 1. Strain Gauge Sensor

Strain gauge mengubah resistansi ketika benda mengalami regangan:

```
dR / R = gauge_factor * strain
```

Model memakai output jembatan dengan satu gauge aktif:

```
Vout = clamp(offset + VCC * gauge_factor * strain / 4, 0, VCC)
```

Default `gauge_factor` 2.1, `strain` 0.0, dan `offset` 0.0. Pada
supply 5 V dan strain 0.001 (1000 microstrain), output sekitar
2.625 mV.

Batas: model menganggap jembatan seimbang dan hanya satu gauge aktif.
Resistansi nominal gauge, self-heating, dan efek suhu tidak
dimodelkan.

## 2. Load Cell

Load cell adalah transduser gaya yang umumnya memakai strain gauge
dalam jembatan Wheatstone. Output rated dinyatakan dalam mV/V dari
tegangan eksitasi. Model:

```
Vout = clamp(offset + VCC * rated_output * load / rated_load, 0, VCC)
```

Default `rated_output` 0.002 V/V (2 mV/V), `rated_load` 100 N, dan
`load` 0.0. Pada 50 N dan supply 5 V, output sekitar 5 mV.

Batas: model tidak memodelkan empat kabel jembatan, ringing mekanik,
creep, hysteresis, atau overload. Input `load` adalah gaya aksial
satu arah.

## 3. Pressure Sensor

Model memakai respons linier:

```
Vout = clamp(offset + sensitivity * pressure, 0, VCC)
```

Default `sensitivity` 1e-5 V/Pa (10 mV/kPa) dan `offset` 0.5 V.
Pada 100 kPa, output 1.5 V pada supply 5 V.

Batas: referensi tekanan absolut, gauge, atau diferensial tidak
dimodelkan. Input `pressure` harus diberikan sesuai referensi yang
diinginkan. Efek media, suhu, dan respons dinamis tidak ada.

## 4. Force Sensor

Model memakai respons linier:

```
Vout = clamp(offset + sensitivity * force, 0, VCC)
```

Default `sensitivity` 0.01 V/N dan `offset` 0.0. Pada 100 N, output
1.0 V pada supply 5 V. Model ini generik, bukan model khusus
force-sensing resistor.

Batas: satu sumbu, tanpa hysterisis mekanik, overload, atau
ketergantungan pada luas kontak.

## 5. Referensi

- Strain gauge, definisi gauge factor dan persamaan jembatan dengan
  satu gauge aktif:
  https://en.wikipedia.org/wiki/Strain_gauge
- Load cell, strain gauge bridge dan rated output mV/V:
  https://en.wikipedia.org/wiki/Load_cell
- Pressure measurement, definisi tekanan dan referensi absolut,
  gauge, serta diferensial:
  https://en.wikipedia.org/wiki/Pressure_measurement

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 6. Batas Umum

- Semua entry hanya satu sumbu.
- Tidak ada noise, drift suhu, hysteresis, atau nonlinearitas.
- Tidak ada respons frekuensi, ringing, atau waktu tunda.
- Tidak ada jembatan empat kabel atau kompensasi suhu.
- Output adalah Thevenin ideal dengan resistor output 1000 ohm.
- Belum ada reference vector atau status `calibrated`.
