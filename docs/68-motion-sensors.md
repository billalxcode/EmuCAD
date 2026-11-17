# 68 - Sensor Gerak

Dokumen ini mencatat model behavioral untuk 5 entry sensor gerak 3-pin.
Semua model memakai pin VCC, GND, dan OUT. Output dibatasi 0 sampai
tegangan VCC. Setiap entry memiliki satu input lingkungan eksplisit
dan hanya memodelkan satu sumbu.

## 1. Persamaan Umum

```
Vout = clamp(offset + sensitivity * input, 0, VCC)
```

Default `offset` adalah 2.5 V dan `output_resistance` 1000 ohm, kecuali
dinyatakan lain. Input default 0.0.

| Entry | Input | Sensitivitas default |
|---|---|---:|
| Accelerometer | `acceleration` (m/s2) | 0.050986 V/(m/s2) |
| Gyroscope | `angular_rate` (deg/s) | 0.01 V/(deg/s) |
| IMU | `acceleration` (m/s2) | 0.050986 V/(m/s2) |
| Tilt sensor | `angle` (deg) | 0.025 V/deg |
| Vibration sensor | `vibration` (m/s2) | 0.050986 V/(m/s2) |

Sensitivitas 0.050986 V/(m/s2) setara dengan sekitar 0.5 V/g, dengan
g = 9.80665 m/s2. Pada supply 5 V, input 1 g menghasilkan 3.0 V.

## 2. Accelerometer

Accelerometer mengukur proper acceleration, termasuk percepatan akibat
gaya yang menahan benda di permukaan bumi. Model memakai input
`acceleration` dalam m/s2 dan respons linier.

Batas: hanya satu sumbu, tanpa gravitasi otomatis, cross-axis, bias,
drift, noise, atau bandwidth. Pengguna harus memasukkan percepatan
yang ingin disimulasikan.

## 3. Gyroscope

Gyroscope mengukur kecepatan sudut. Model memakai input `angular_rate`
dalam deg/s dan respons linier. Default 10 mV per deg/s, sehingga
100 deg/s menghasilkan 3.5 V pada supply 5 V.

Batas: hanya satu sumbu dan tanpa integrasi sudut. Bias, drift,
noise, dan ketergantungan suhu tidak dimodelkan.

## 4. IMU

Entry IMU memakai model accelerometer satu sumbu. Ini adalah subset
behavioral, bukan representasi enam atau sembilan sumbu.

Batas: tidak ada accelerometer dan gyroscope gabungan, fusion, atau
orientasi tiga sumbu.

## 5. Tilt Sensor

Model memakai input `angle` dalam derajat dan respons linier:

```
Vout = clamp(offset + sensitivity * angle, 0, VCC)
```

Default 0.025 V/deg dan offset 2.5 V. Pada supply 5 V, rentang linier
yang tidak terpotong adalah sekitar -90 sampai +90 derajat.

Batas: hubungan sebenarnya adalah fungsi sinus terhadap gravitasi.
Model linier hanya cocok untuk sudut kecil atau rentang yang sudah
dikalibrasi.

## 6. Vibration Sensor

Model memakai input `vibration` dalam m/s2 sebagai amplitudo
percepatan sesaat. Default sensitivitas sama dengan accelerometer,
yaitu sekitar 0.5 V/g, dan offset 2.5 V agar sinyal dua arah dapat
direpresentasikan.

Batas: input bukan nilai RMS, bukan spektrum frekuensi, dan tidak ada
resonansi mekanik atau filter high-pass.

## 7. Referensi

- Accelerometer, proper acceleration dan pemakaian untuk vibration
  monitoring:
  https://en.wikipedia.org/wiki/Accelerometer
- Gyroscope, pengukuran orientasi dan angular velocity:
  https://en.wikipedia.org/wiki/Gyroscope
- Inclinometer, pengukuran sudut terhadap arah gravitasi:
  https://en.wikipedia.org/wiki/Inclinometer

Persamaan di dokumen ini adalah model behavioral proyek. Default
sensitivitas dipilih agar mudah dibaca, bukan berasal dari datasheet
part tertentu.

## 8. Batas Umum

- Semua entry hanya satu sumbu.
- Tidak ada cross-axis sensitivity atau misalignment.
- Tidak ada noise, bias, drift suhu, atau nonlinearitas.
- Tidak ada respons frekuensi atau waktu tunda.
- Output adalah Thevenin ideal dengan resistor output.
- Belum ada reference vector atau status `calibrated`.
