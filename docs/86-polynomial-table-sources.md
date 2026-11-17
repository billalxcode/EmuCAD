# 86 - Polynomial dan Table Source

Dokumen ini mencatat model untuk polynomial voltage source dan table
voltage source 2-pin.

## 1. Polynomial Voltage Source

Output adalah polinomial waktu:

```
V(t) = c0 + c1*t + c2*t^2 + c3*t^3
```

Default `c0` 0, `c1` 1 V/s, `c2` 0, dan `c3` 0. Waktu maju mengikuti
`dt` pada transient; pada DC nilai t = 0.

## 2. Table Voltage Source

Table source memakai model PWL empat titik:

```
(t0, v0), (t1, v1), (t2, v2), (t3, v3)
```

Interpolasi linier antar titik. Sebelum titik pertama output `v0`;
setelah titik terakhir output `v3`. Default `(0, 0)`, `(1 ms, 5 V)`,
`(2 ms, 5 V)`, dan `(3 ms, 0 V)`.

Batas: table generik dengan lebih dari empat titik belum didukung.

## 3. Referensi

- Polynomial, ekspresi pangkat dan koefisien:
  https://en.wikipedia.org/wiki/Polynomial
- Lookup table, table nilai untuk interpolasi:
  https://en.wikipedia.org/wiki/Lookup_table
- Piecewise linear function, interpolasi antar titik:
  https://en.wikipedia.org/wiki/Piecewise_linear_function

Persamaan di dokumen ini adalah model behavioral proyek. Default
dipilih agar mudah dibaca dan bukan berasal dari datasheet part
tertentu.

## 4. Batas

- Polynomial memakai waktu sebagai variabel, bukan tegangan kontrol.
- Table hanya empat titik.
- Tidak ada extrapolation atau smoothing.
- Behavioral voltage/current source dengan ekspresi bebas belum
  didukung.
- Belum ada reference vector atau status `calibrated`.
