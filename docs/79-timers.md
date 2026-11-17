# 79 - Timer dan Oscillator

Dokumen ini mencatat model behavioral untuk 4 entry timer 8-pin:
555 timer, monostable, astable, dan VCO. PLL belum didukung.

## 1. Pemetaan Pin

Model memakai pin berikut:

| Pin | Fungsi |
|---:|---|
| 0 | VCC |
| 1 | GND |
| 2 | OUT |
| 3 | RESET, high impedance |
| 4 | TRIG untuk monostable |
| 5 | CTRL untuk VCO |
| 6 | THR, high impedance |
| 7 | DIS, high impedance |

Hanya pin 0, 1, dan 2 yang aktif untuk astable. Pin lain tidak
di-drive dan harus memiliki jalur DC jika diperlukan.

## 2. Astable dan 555 Timer

```
phase = (t mod period) / period
Vout = high jika phase < duty, selain itu 0
```

Default `period` 1 ms dan `duty` 0.5. Output high mengikuti VCC dan
rendah 0 V. `output_resistance` default 1000 ohm.

555 timer memakai model astable yang sama.

## 3. Monostable

Output low sampai trigger pada pin 4 melewati `trigger_threshold`.
Saat tepi naik terdeteksi, output high selama `period`, lalu kembali
low. Trigger yang datang saat pulsa berjalan tidak memperpanjang pulsa.

Default `period` 1 ms dan `trigger_threshold` 2.5 V.

## 4. VCO

```
frequency = 1 / period + sensitivity * (Vctrl - control_center)
```

Frekuensi dibatasi minimal 0. Default `period` 1 ms, `sensitivity`
1000 Hz/V, dan `control_center` 2.5 V. Output adalah gelombang kotak
dengan duty 50 persen.

Batas: phase dan frekuensi dihitung dari control voltage pada saat
update; tidak ada phase noise atau jitter.

## 5. PLL

PLL tetap unsupported. Model PLL membutuhkan phase detector, loop
filter, divider, dan umpan balik yang stabil. Tanpa model itu, PLL
akan menyesatkan.

## 6. Referensi

- 555 timer IC, aplikasi timer dan oscillator:
  https://en.wikipedia.org/wiki/555_timer_IC
- Multivibrator, astable dan monostable:
  https://en.wikipedia.org/wiki/Multivibrator
- Voltage-controlled oscillator, frekuensi oleh tegangan kontrol:
  https://en.wikipedia.org/wiki/Voltage-controlled_oscillator
- Phase-locked loop, dasar loop dan divider:
  https://en.wikipedia.org/wiki/Phase-locked_loop

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 7. Batas Umum

- Tidak ada reset, threshold, discharge, atau control internal 555.
- Tidak ada propagation delay atau jitter.
- Tidak ada model komponen eksternal RC.
- Belum ada reference vector atau status `calibrated`.
