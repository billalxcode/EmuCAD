# 72 - Sensor Proximity

Dokumen ini mencatat model behavioral untuk 3 entry sensor proximity
3-pin. Semua model memakai pin VCC, GND, dan OUT. Output dibatasi
0 sampai tegangan VCC.

## 1. Inductive Proximity Sensor

Sensor induktif mendeteksi target logam tanpa kontak. Model bekerja
sebagai switch dengan hysteresis:

```
on  jika distance < threshold
off jika distance > threshold + hysteresis
```

Default `threshold` 5 mm dan `hysteresis` 0.5 mm. Input `distance`
dalam mm. Output adalah 0 V saat off dan VCC saat on.

Batas: hanya target logam yang dianggap valid. Bentuk medan, ukuran
target, dan bahan target tidak dimodelkan.

## 2. Capacitive Touch Sensor

Sensor kapasitif mendeteksi objek konduktif atau dielektrik melalui
perubahan kapasitansi. Model bekerja sebagai switch dengan hysteresis:

```
on  jika capacitance > threshold
off jika capacitance < threshold - hysteresis
```

Default `threshold` 10 pF dan `hysteresis` 2 pF. Input `capacitance`
dalam pF. Output adalah 0 V saat off dan VCC saat on.

Batas: stray capacitance, luas elektroda, lapisan pelindung, dan
kelembapan tidak dimodelkan. Nilai stray capacitance nyata dapat
berkisar dari sekitar 10 pF sampai 10 nF.

## 3. Ultrasonic Distance Sensor

Sensor ultrasonik mengukur jarak dari waktu tempuh pulsa. Model
memakai output analog linier terhadap jarak:

```
Vout = clamp(offset + sensitivity * distance, 0, VCC)
```

Default `sensitivity` 1.0 V/m dan `offset` 0.5 V. Pada 2 m, output
2.5 V pada supply 5 V. Waktu tempuh tidak dihitung ulang; input
`distance` sudah merupakan hasil pengukuran.

Batas: kecepatan suara terhadap suhu, sudut pancaran, blind zone,
pantulan target, dan waktu respons tidak dimodelkan.

## 4. Referensi

- Proximity sensor, tipe induktif, kapasitif, dan ultrasonik:
  https://en.wikipedia.org/wiki/Proximity_sensor
- Capacitive sensing, deteksi objek konduktif atau dielektrik dan
  stray capacitance:
  https://en.wikipedia.org/wiki/Capacitive_sensing
- Ultrasonic transducer, pengukuran jarak dari waktu tempuh dan
  output analog yang proporsional terhadap jarak:
  https://en.wikipedia.org/wiki/Ultrasonic_transducer

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 5. Batas Umum

- Tidak ada noise, drift suhu, atau cross-sensitivity.
- Tidak ada waktu respons, hysteresis mekanik, atau saturasi.
- Tidak ada model medan elektromagnetik atau akustik.
- Output switch adalah push-pull 0/VCC, bukan open-drain.
- Output analog adalah Thevenin ideal dengan resistor output 1000 ohm.
- Belum ada reference vector atau status `calibrated`.
