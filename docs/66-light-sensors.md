# 66 - Sensor Cahaya

Dokumen ini mencatat model behavioral untuk 5 entry sensor cahaya 3-pin.
Semua model memakai pin VCC, GND, dan OUT. Output dibatasi 0 sampai
tegangan VCC.

## 1. Photoresistor

Model gamma empiris:

```
R(E) = R10 * (E / 10)^(-gamma)
Vout = VCC * R(E) / (R(E) + Rseries)
```

Parameter:

| Parameter | Default | Arti |
|---|---:|---|
| `resistance_10lux` | 10 kOhm | resistansi pada 10 lux |
| `gamma` | 0.8 | kemiringan log-log |
| `series_resistance` | 10 kOhm | resistor divider |
| `illuminance` | 10 lux | input lingkungan |

Batas: model empiris, bukan data spektral atau datasheet part tertentu.

## 2. Photodiode

Photocurrent linear terhadap irradiance:

```
Iph = responsivity * area * irradiance
Vout = offset + Iph * Rfeedback
```

Default: `responsivity` 0.5 A/W, `area` 1e-6 m^2,
`feedback_resistance` 1e6 ohm, `irradiance` 1 W/m^2, sehingga output
0.5 V. Batas: tanpa dark current, spectral response, atau response time.

## 3. Linear Light Sensor

```
Vout = offset + sensitivity * illuminance
```

Dipakai untuk phototransistor, ambient light sensor, dan color sensor.
Default sensitivity 10 mV/lux untuk phototransistor dan 1 mV/lux untuk
ambient/color. Batas: color sensor hanya satu kanal, bukan RGB.

## 4. Referensi

- Photoresistor, perubahan resistansi terhadap iluminasi:
  https://en.wikipedia.org/wiki/Photoresistor
- Photodiode, photocurrent linear terhadap irradiance dan responsivity
  A/W:
  https://en.wikipedia.org/wiki/Photodiode

Model gamma LDR adalah fitting empiris umum; untuk produksi, gunakan
datasheet part dan kalibrasi.

## 5. Batas Umum

- Tidak ada spectral response atau filter warna.
- Tidak ada dark current, noise, atau response time.
- Tidak ada self-heating atau ketergantungan suhu.
- Output adalah Thevenin ideal dengan resistor output.
- Belum ada reference vector atau status `calibrated`.
