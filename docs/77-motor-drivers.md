# 77 - Motor Driver

Dokumen ini mencatat model behavioral untuk 3 entry motor driver
empat pin. Model memakai satu kanal high-side switch dengan hysteresis
dan resistansi on/off.

## 1. Pin dan Persamaan

Pin 0 adalah VCC, pin 1 ground, pin 2 input kontrol, dan pin 3 output.

```
on  jika Vin > threshold
off jika Vin < threshold - hysteresis
Vout = on ? VCC : 0
Rout = on ? on_resistance : off_resistance
```

Default `threshold` 2.5 V, `hysteresis` 0.1 V, `on_resistance` 0.5
ohm, dan `off_resistance` 1e6 ohm. Output adalah Thevenin terhadap
ground.

Pada supply 5 V dan beban 10 ohm, output on sekitar 4.762 V dan output
off mendekati 0 V.

## 2. Cakupan Entry

H-bridge, half-bridge, dan stepper driver memakai model satu kanal yang
sama. Ini adalah subset behavioral, bukan model empat saklar H-bridge
atau sekuens step penuh.

## 3. Batas

- Tidak ada arah putaran, PWM, atau dead time.
- Tidak ada dua output komplementer.
- Tidak ada current limit, shoot-through, atau proteksi termal.
- Tidak ada flyback diode atau regenerasi.
- Tidak ada model motor mekanik di dalam driver.
- Stepper driver tidak menjalankan sekuens winding.

## 4. Referensi

- H-bridge, rangkaian yang membalik polaritas beban:
  https://en.wikipedia.org/wiki/H-bridge
- Motor controller, pengaturan start, arah, kecepatan, dan proteksi:
  https://en.wikipedia.org/wiki/Motor_controller
- Stepper motor, putaran diskret dan step position:
  https://en.wikipedia.org/wiki/Stepper_motor
- Brushed DC electric motor, komutasi internal motor DC:
  https://en.wikipedia.org/wiki/Brushed_DC_electric_motor

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 5. Batas Umum

- Output high mengikuti supply aktual.
- Belum ada reference vector atau status `calibrated`.
- Model hanya valid untuk analisis daya dan level sinyal.
