# 65 - Sensor Suhu

Dokumen ini mencatat model behavioral untuk 6 entry sensor suhu 3-pin.
Semua model memakai pin VCC, GND, dan OUT. Output dibatasi 0 sampai
tegangan VCC.

## 1. NTC

Persamaan beta:

```
R(T) = R25 * exp(B * (1 / (T + 273.15) - 1 / 298.15))
Vout = VCC * R(T) / (R(T) + Rseries)
```

Parameter:

| Parameter | Default | Arti |
|---|---:|---|
| `resistance` | 10 kOhm | R pada 25 C |
| `beta` | 3950 K | koefisien beta |
| `series_resistance` | 10 kOhm | resistor divider |
| `temperature` | 25 C | input lingkungan |

Batas: tanpa self-heating, tanpa Steinhart-Hart, dan tanpa toleransi
per part.

## 2. PTC

```
R(T) = R25 * (1 + alpha * (T - 25))
Vout = VCC * R(T) / (R(T) + Rseries)
```

Parameter: `resistance`, `temperature_coefficient`, `series_resistance`,
dan `temperature`. Batas: hanya PTC linier, bukan PTC switching.

## 3. RTD

Callendar-Van Dusen:

```
T >= 0: R(T) = R0 * (1 + A*T + B*T^2)
T < 0:  R(T) = R0 * (1 + A*T + B*T^2 + C*T^3*(T - 100))
```

Default PT100: `resistance` 100 ohm, `alpha` 3.9083e-3,
`beta` -5.775e-7, `c` -4.183e-12, `series_resistance` 1000 ohm.
Batas: tanpa resistansi kabel, self-heating, atau kalibrasi per probe.

## 4. Thermocouple

```
Vout = offset + sensitivity * (T - Tcold)
```

Default: `sensitivity` 41 uV/C untuk tipe K, `cold_junction` 25 C.
Batas: linearisasi lokal, bukan polinomial NIST penuh dan bukan
kompensasi cold junction otomatis.

## 5. Sensor Semikonduktor

```
Vout = offset + sensitivity * T
```

Default: `sensitivity` 10 mV/C dan `offset` 0 V, mengikuti skala
LM35. Batas: tanpa akurasi, self-heating, atau nonlinier.

## 6. Thermal Switch

Output VCC saat `temperature > threshold`, 0 saat
`temperature < threshold - hysteresis`, dan state terakhir di antara
keduanya. Default threshold 75 C dan hysteresis 5 C.

## 7. Referensi

- Resistance thermometer, Callendar-Van Dusen dan PT100:
  https://en.wikipedia.org/wiki/Resistance_thermometer
- Thermocouple, sensitivitas tipe K sekitar 41 uV/C:
  https://en.wikipedia.org/wiki/Thermocouple
- Thermistor, persamaan beta:
  https://en.wikipedia.org/wiki/Thermistor
- LM35, gain 10 mV/C:
  https://www.ti.com/product/LM35

Untuk produksi, referensi primer yang harus dipakai adalah IEC 60751
untuk RTD, tabel NIST ITS-90 untuk thermocouple, dan datasheet TI untuk
LM35. Wikipedia dipakai sebagai verifikasi awal, bukan sumber akhir.

## 8. Batas Umum

- `temperature` adalah parameter lingkungan, bukan simulasi termal.
- Tidak ada self-heating, waktu respons, atau kopling termal.
- Tidak ada toleransi, drift, atau kalibrasi per part.
- Output adalah Thevenin ideal dengan resistor output.
- Model belum `calibrated`; belum ada reference vector.
