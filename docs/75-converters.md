# 75 - Konverter Daya

Dokumen ini mencatat model behavioral untuk 8 entry DC-DC converter dan
5 entry charge pump/charger. Model memakai output teregulasi dan input
current yang menjaga efisiensi.

## 1. Persamaan

```
Vout = output_voltage
Iin = Iout * output_voltage / (rated_input_voltage * efficiency)
Pin = Vin_actual * Iin
Pout = Vout * Iout
```

Output dimodelkan sebagai sumber tegangan ideal. Input dimodelkan
sebagai current-controlled current source yang mengambil arus dari
sisi input sebanding dengan arus output. Efisiensi harus lebih besar
dari 0 dan maksimal 1.

## 2. DC-DC Converter

Empat pin: input positif, input negatif, output positif, dan output
negatif. Default `rated_input_voltage` 12 V, `output_voltage` 5 V, dan
`efficiency` 0.85.

Semua topologi buck, boost, buck-boost, Cuk, SEPIC, flyback, forward,
dan push-pull memakai model yang sama. Topologi hanya memengaruhi
nilai output dan efisiensi yang diberikan.

## 3. Charge Pump dan Charger

Tiga pin: input, output, dan ground. Default `rated_input_voltage`
5 V, `output_voltage` 5 V, dan `efficiency` 0.8. Output negatif dapat
diberikan untuk inverting charge pump.

Model yang sama dipakai untuk charge pump, inverting charge pump,
linear charger, switching charger, dan solar charger. Untuk charger,
`output_voltage` adalah tegangan baterai yang diatur.

## 4. Batas

- Tidak ada duty cycle, switching frequency, atau ripple.
- Tidak ada induktor, kapasitor output, atau respons transien.
- Tidak ada isolasi, rectifier, atau snubber.
- Tidak ada current limit, soft start, atau proteksi.
- Tidak ada model termal atau degradasi baterai.
- Efisiensi dianggap konstan pada rentang operasi.

## 5. Referensi

- DC-to-DC converter, konversi level tegangan DC:
  https://en.wikipedia.org/wiki/DC-to-DC_converter
- Buck converter, step-down dan efisiensi switching:
  https://en.wikipedia.org/wiki/Buck_converter
- Boost converter, step-up:
  https://en.wikipedia.org/wiki/Boost_converter
- Charge pump, konversi dengan kapasitor:
  https://en.wikipedia.org/wiki/Charge_pump
- Battery charger, protokol pengisian baterai:
  https://en.wikipedia.org/wiki/Battery_charger
- Solar charger, pengisian dari energi surya:
  https://en.wikipedia.org/wiki/Solar_charger

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 6. Batas Umum

- Output ideal tidak memiliki output resistance atau droop.
- Input current bergantung pada arus output, bukan pada topologi.
- Belum ada reference vector atau status `calibrated`.
- Model hanya valid untuk analisis daya rata-rata.
