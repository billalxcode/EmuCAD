# 67 - Sensor Magnetik

Dokumen ini mencatat model behavioral untuk 5 entry sensor magnetik 3-pin.
Semua model memakai pin VCC, GND, dan OUT. Output dibatasi 0 sampai
tegangan VCC. Medan magnet diberikan eksplisit lewat parameter
`magnetic_field` dalam satuan tesla (T).

## 1. Hall Switch

Model switch dengan hysteresis:

```
on  jika B > threshold
off jika B < threshold - hysteresis
```

Output adalah 0 V saat off dan VCC saat on, dengan resistansi output
1000 ohm. Default `threshold` 5 mT dan `hysteresis` 1 mT.

Batas: model tidak memodelkan open-drain, active-low, atau tegangan
saturasi rendah. Polaritas medan dianggap positif untuk arah aktif.

## 2. Linear Hall Sensor

```
Vout = clamp(offset + sensitivity * B, 0, VCC)
```

Default `sensitivity` 5 V/T dan `offset` 2.5 V. Pada supply 5 V, medan
0 T menghasilkan 2.5 V dan medan 0.2 T menghasilkan 3.5 V.

Batas: offset dalam volt tetap, bukan ratiometric terhadap supply.
Saturasi atas dan bawah mengikuti VCC dan 0 V.

## 3. Reed Switch Sensor

Model memakai persamaan switch yang sama dengan Hall switch:

```
on  jika B > threshold
off jika B < threshold - hysteresis
```

Default `threshold` 5 mT dan `hysteresis` 1 mT. Model ini menghasilkan
sinyal tegangan pada pin OUT, bukan kontak mekanis mengambang.

Batas: pull-in dan drop-out hanya bergantung pada besar medan pada
satu arah. Orientasi, bouncing kontak, dan umur kontak tidak
dimodelkan.

## 4. Magnetoresistive Sensor

Resistansi bergantung pada medan:

```
R(B) = max(R0 * (1 + relative_sensitivity * B), 1e-9)
Vout = VCC * R(B) / (R(B) + series_resistance)
```

Default `resistance` 1000 ohm, `relative_sensitivity` 10 per tesla, dan
`series_resistance` 1000 ohm. Pada 0.05 T, R menjadi 1500 ohm dan
output 3 V pada supply 5 V.

Batas: model satu resistor linear, bukan jembatan Wheatstone.
Anisotropi, hysteresis material, dan saturasi magnetik tidak
dimodelkan.

## 5. Fluxgate Sensor

```
Vout = clamp(offset + sensitivity * B, 0, VCC)
```

Default `sensitivity` 1000 V/T dan `offset` 0 V, sehingga 1 mT
menghasilkan 1 V pada supply yang cukup. Batas: eksitasi AC, inti
saturasi, dan noise tidak dimodelkan; respons hanya linear.

## 6. Referensi

- Hall-effect sensor, output proporsional terhadap medan dan
  hysteresis pada Hall switch:
  https://en.wikipedia.org/wiki/Hall-effect_sensor
- Reed switch, aktuasi kontak oleh medan magnet:
  https://en.wikipedia.org/wiki/Reed_switch
- Magnetometer, tipe magnetoresistive dan fluxgate:
  https://en.wikipedia.org/wiki/Magnetometer

Persamaan di dokumen ini adalah model behavioral proyek, bukan
persamaan material dari referensi. Untuk kalibrasi, gunakan datasheet
part dan pengukuran.

## 7. Batas Umum

- Tidak ada noise, drift suhu, atau offset selain parameter eksplisit.
- Tidak ada respons frekuensi atau waktu tunda.
- Tidak ada medan magnet dari arus di netlist; input harus diberikan
  sebagai parameter.
- Output adalah Thevenin ideal dengan resistor output.
- Belum ada reference vector atau status `calibrated`.
