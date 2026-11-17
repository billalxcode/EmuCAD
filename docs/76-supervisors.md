# 76 - Reference dan Supervisor

Dokumen ini mencatat model behavioral untuk 5 entry reference dan
supply supervisor. Semua model memakai pin VCC, GND, dan OUT.

## 1. Voltage Reference

Output adalah sumber tegangan tetap terhadap ground:

```
Vout = threshold
Isupply = quiescent_current
```

Default `threshold` 2.5 V dan `quiescent_current` 1 mA. Arus quiescent
diambil dari pin VCC sehingga supply mengalami drop jika ada
resistansi seri.

Batas: tidak ada drift suhu, noise, load regulation, atau current
limit. Output ideal tidak memiliki resistansi output.

## 2. Current Reference

Output adalah sumber arus tetap dari pin OUT ke ground:

```
Iout = output_current
Isupply = quiescent_current
```

Default `output_current` 1 mA dan `quiescent_current` 1 mA. Pada beban
1 kOhm, output 1 V.

Batas: tidak ada compliance voltage, drift, noise, atau current limit.
Jika beban membutuhkan tegangan di luar rentang supply, output tetap
dipaksa oleh sumber arus ideal.

## 3. Voltage Supervisor, Reset Supervisor, Power Monitor

Model membandingkan tegangan supply dengan threshold dan hysteresis:

```
on  jika Vsupply > threshold
off jika Vsupply < threshold - hysteresis
Vout = on ? (active_low ? 0 : Vsupply) : (active_low ? Vsupply : 0)
```

Default `threshold` 2.5 V, `hysteresis` 0.1 V, dan
`output_resistance` 1000 ohm. Voltage supervisor dan power monitor
memakai `active_low` 0. Reset supervisor memakai `active_low` 1.

Output high mengikuti tegangan supply aktual, bukan nilai tetap 5 V.
Pada supply 2 V, reset supervisor yang aktif menghasilkan output 2 V.

Batas: tidak ada delay, glitch filter, open-drain, atau fault latch.
Ambang hanya bergantung pada tegangan supply, bukan pada riak atau
suhu.

## 4. Referensi

- Voltage reference, tegangan tetap terhadap beban dan supply:
  https://en.wikipedia.org/wiki/Voltage_reference
- Current source, arus independen terhadap tegangan:
  https://en.wikipedia.org/wiki/Current_source
- Power management IC, konteks distribusi dan kontrol daya:
  https://en.wikipedia.org/wiki/Power_management_integrated_circuit
- Reset, keadaan awal yang terkontrol:
  https://en.wikipedia.org/wiki/Reset_(computing)

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 5. Batas Umum

- Tidak ada noise, drift suhu, atau long-term stability.
- Tidak ada load regulation atau line regulation.
- Tidak ada delay, soft start, atau fault protection.
- Belum ada reference vector atau status `calibrated`.
