# 70 - Sensor Kelembapan dan Kimia

Dokumen ini mencatat model behavioral untuk 4 entry sensor kelembapan
dan kimia 3-pin. Semua model memakai pin VCC, GND, dan OUT. Output
dibatasi 0 sampai tegangan VCC.

## 1. Humidity Sensor

Model memakai respons linier terhadap kelembapan relatif:

```
Vout = clamp(offset + sensitivity * relative_humidity, 0, VCC)
```

Default `sensitivity` 0.03 V/%RH dan `offset` 0.8 V. Pada 50 %RH,
output 2.3 V pada supply 5 V. Rentang input yang disarankan 0 sampai
100 %RH.

Batas: model tidak mengoreksi suhu, hysteresis, drift, kondensasi,
atau kontaminasi. Sensor kapasitif dan resistif nyata memerlukan
kalibrasi pada beberapa titik.

## 2. Gas Sensor

Model chemiresistor metal-oxide:

```
R(C) = max(R0 * (1 + relative_sensitivity * C), 1e-9)
Vout = VCC * R(C) / (R(C) + series_resistance)
```

Default `resistance` 10000 ohm, `relative_sensitivity` -0.0001 per
ppm, dan `series_resistance` 10000 ohm. Sensitivitas negatif
merepresentasikan penurunan resistansi saat gas reduktor terdeteksi.
Pada 1000 ppm, R menjadi 9000 ohm dan output sekitar 2.368 V.

Batas: respons gas nyata umumnya tidak linier dan bergantung pada
heater, kelembapan, suhu, serta cross-sensitivity. Model ini hanya
linierisasi pada rentang kecil.

## 3. pH Sensor

Model memakai kemiringan Nernst di sekitar pH netral:

```
Vout = clamp(offset + sensitivity * (7 - ph), 0, VCC)
```

Default `sensitivity` 0.05916 V/pH (sekitar 59.16 mV/pH pada 25 C),
`offset` 2.5 V, dan `ph` 7.0. Pada pH 4, output 2.67748 V.

Batas: tidak ada kompensasi suhu, penuaan elektroda, impedansi
elektroda, atau kalibrasi buffer. Elektroda referensi dianggap
ideal.

## 4. Electrochemical Sensor

Model amperometrik dengan output linier terhadap konsentrasi:

```
Vout = clamp(offset + sensitivity * concentration, 0, VCC)
```

Default `sensitivity` 0.01 V/ppm dan `offset` 0.0 V. Pada 100 ppm,
output 1.0 V pada supply 5 V.

Batas: tidak ada baseline, cross-sensitivity, waktu respons, atau
ketergantungan suhu. Model menganggap konversi arus ke tegangan
sudah dilakukan di dalam modul.

## 5. Referensi

- Hygrometer, sensor kapasitif dan resistif untuk kelembapan relatif:
  https://en.wikipedia.org/wiki/Hygrometer
- Gas detector, sensor metal-oxide dan electrochemical:
  https://en.wikipedia.org/wiki/Gas_detector
- pH meter, hubungan potensial linier terhadap pH melalui persamaan
  Nernst dan sekitar 0.06 V per unit pH:
  https://en.wikipedia.org/wiki/PH_meter
- Electrochemical gas sensor, output arus linier terhadap
  konsentrasi gas:
  https://en.wikipedia.org/wiki/Electrochemical_gas_sensor

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 6. Batas Umum

- Tidak ada noise, drift suhu, atau cross-sensitivity.
- Tidak ada waktu respons, histeresis, atau penuaan.
- Tidak ada model kimia atau elektroda fisik.
- Output adalah Thevenin ideal dengan resistor output 1000 ohm.
- Belum ada reference vector atau status `calibrated`.
