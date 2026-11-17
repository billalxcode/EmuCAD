# 73 - Actuator dan Display

Dokumen ini mencatat model behavioral untuk 20 entry actuator dan
display. Model merepresentasikan beban elektrik dan konversi energi
utama. Output fisik seperti cahaya, suara, torsi, dan piksel belum
dirender sebagai kanal terpisah; pengguna membaca arus dan daya
terminal dari hasil simulasi.

## 1. LED dan Lampu

LED dan RGB LED memakai junction eksponensial:

```
I = is * (exp(Vj / (n * Vt)) - 1)
V = Vj + rs * I
```

Default `is` 1e-15 A, `n` 2.0, dan `rs` 10 ohm. Pada 20 mA, tegangan
maju berada di kisaran LED merah. RGB LED adalah subset satu kanal.

Incandescent lamp memakai model resistor 100 ohm. Neon lamp memakai
model DIAC dengan `vbo` 90 V sebagai pendekatan gas discharge dua
arah.

Batas: warna, fluks cahaya, efisiensi, dan thermal filament tidak
dimodelkan. Neon lamp tidak memiliki maintaining voltage atau
polaritas.

## 2. Buzzer, Speaker, dan Piezo

Buzzer dan speaker memakai beban resistor. Speaker default 8 ohm dan
buzzer 100 ohm. Piezo transducer dan piezo buzzer memakai kapasitor
dengan default 100 nF dan 20 nF.

Batas: impedansi akustik, resonansi, SPL, dan respons frekuensi tidak
dimodelkan. Model hanya valid untuk beban listrik pada level sinyal.

## 3. Motor dan Solenoid

Motor DC, brushless DC, servo, dan stepper memakai model seri:

```
I = (V - back_emf_constant * speed) / resistance
```

Default resistance 10, 5, 50, dan 20 ohm, dengan
`back_emf_constant` 0.01 V/(rad/s). Semua parameter memengaruhi arus
terminal. Solenoid memakai resistor 20 ohm.

Batas: hanya satu winding. Komutasi BLDC, kontrol posisi servo,
sekuens step, torsi beban, inersia, dan induktansi solenoid tidak
dimodelkan.

## 4. Display

Empat entry display memakai model beban empat pin:

```
G_supply = supply_current * brightness / rated_voltage
```

Pin VCC dan GND dibebani conductance tersebut. Dua pin input memiliki
`input_leakage` ke ground. Default `rated_voltage` 5 V,
`input_leakage` 1e-6 S, dan `supply_current` 1, 2, 5, atau 20 mA
sesuai entry.

Batas: isi piksel, protokol data, backlight, kontras, dan waktu refresh
tidak dimodelkan. Pin input hanya berupa beban kebocoran.

## 5. Heater, Peltier, dan TEG

Resistive heater memakai resistor 10 ohm. Peltier cooler dan
thermoelectric generator memakai model seri:

```
I = (V - seebeck_constant * delta_temperature) / resistance
```

Peltier default resistance 2 ohm dan Seebeck 0.05 V/K. TEG default
resistance 5 ohm dan Seebeck 0.05 V/K, sehingga berfungsi sebagai
sumber Thevenin ketika perbedaan suhu diberikan.

Batas: perpindahan panas, COP, efisiensi, dan aliran panas tidak
dimodelkan. Peltier dan TEG hanya memodelkan kopel elektrik-termal
pada level terminal.

## 6. Referensi

- LED, emisi cahaya saat arus mengalir:
  https://en.wikipedia.org/wiki/Light-emitting_diode
- Incandescent light bulb, pemanasan Joule pada filamen:
  https://en.wikipedia.org/wiki/Incandescent_light_bulb
- Neon lamp, gas discharge dan tegangan penyalaan:
  https://en.wikipedia.org/wiki/Neon_lamp
- Buzzer, perangkat sinyal audio:
  https://en.wikipedia.org/wiki/Buzzer
- Loudspeaker, transduser elektroakustik:
  https://en.wikipedia.org/wiki/Loudspeaker
- DC motor, konversi arus listrik menjadi gaya mekanik:
  https://en.wikipedia.org/wiki/DC_motor
- Thermoelectric effect, konversi beda suhu dan tegangan:
  https://en.wikipedia.org/wiki/Thermoelectric_effect
- Thermoelectric generator, sumber listrik dari beda suhu:
  https://en.wikipedia.org/wiki/Thermoelectric_generator
- Liquid-crystal display, modulasi cahaya oleh kristal cair:
  https://en.wikipedia.org/wiki/Liquid-crystal_display
- OLED, dioda organik yang memancarkan cahaya:
  https://en.wikipedia.org/wiki/OLED
- E Ink, teknologi electronic paper:
  https://en.wikipedia.org/wiki/E_Ink
- Solenoid, kumparan elektromagnetik:
  https://en.wikipedia.org/wiki/Solenoid

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 7. Batas Umum

- Output fisik belum menjadi kanal simulasi terpisah.
- Tidak ada noise, drift suhu, umur, atau efisiensi terkalibrasi.
- Tidak ada model akustik, optik, mekanik, atau termal penuh.
- Model motor hanya satu winding dan satu sumbu.
- Model display tidak menjalankan protokol data atau piksel.
- Belum ada reference vector atau status `calibrated`.
