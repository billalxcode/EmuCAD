# 38 - Profiling dan Optimasi

Dokumen ini menjelaskan benchmark solver dan optimasi hot path.

---

## 1. Status

P10-22 dan P10-23 diimplementasikan:

- Command benchmark di CLI.
- Pengukuran DC dan transient.
- Tiga optimasi pada jalur solver.

Belum ada profiler eksternal, pengukuran release, atau analisis memori.

---

## 2. Benchmark

Command:

```
emucad bench [iterations] [nodes]
```

Default `iterations = 1000` dan `nodes = 10`.

Benchmark memakai rangkaian RC:

- Voltage source `5 V`.
- Resistor `1 kOhm`.
- Capacitor `1 uF`.

Setiap iterasi menjalankan:

- Satu DC operating point.
- Satu transient 100 langkah.

Benchmark kedua memakai resistor ladder dengan jumlah node yang dapat
 dipilih. Setiap node dihubungkan oleh resistor seri `1 kOhm` dan resistor
 shunt `2 kOhm` ke ground. Ladder mengukur biaya per node.

Output:

```
dc: 1000 runs, 0.037 s, 37.1 us/run, ...
transient: 1000 runs, 0.290 s, 290.4 us/run, 2.90 ns/step
ladder-10: 200 runs, 0.091 s, 454.1 us/run, 45.41 us/node
```

Angka bergantung mesin dan build.

---

## 3. Optimasi

### 3.1 Direct DC untuk Rangkaian Linear

Rangkaian tanpa device nonlinear langsung diselesaikan pada `gmin` final.
Gmin stepping hanya dipakai jika ada diode, MOSFET, BJT, JFET, atau device
nonlinear lain.

Alasan: gmin stepping dibutuhkan untuk konvergensi nonlinear, tetapi hanya
menambah biaya pada sistem linear.

### 3.2 Solve Tanpa Clone Matriks

`DenseMatrix::solve_owned` memakai matriks yang sudah dimiliki pemanggil.
`MnaContext::solve` memakai jalur ini sehingga tidak menyalin matriks setiap
solve.

`DenseMatrix::solve` tetap tersedia dan mendelegasikan ke `solve_owned`
setelah clone.

### 3.3 Shared Mapping

`MnaContext` dan `SimulationSystem` berbagi `Arc<Vec<Option<usize>>>` untuk
node dan branch. Sebelumnya kedua mapping di-clone pada setiap solve.

### 3.4 Fast Path Fungsional AVR

CPU AVR memakai jalur fungsional untuk instruksi panas:

- Register disimpan sebagai array `[u8; 32]`, bukan register bank gate-level.
- ADD/ADC/SUB/SBC/CP/CPC/CPI memakai helper flag murni.
- Logic, INC/DEC/NEG, shift, dan multiply dihitung langsung di Rust.
- Hasil decode instruksi di-cache per alamat program.
- Trace hanya dibuat pada API `step`; `step_untraced` tidak menyalin snapshot.
- Peripheral serial dilewati saat tidak ada transfer aktif dan interrupt UART mati.

Unit struktural ALU, shift, multiply, dan register bank tetap tersedia di
`sim-cells` dan `sim-avr` untuk test dan pengembangan mode struktural.
Hot path `Cpu` tidak lagi membangun unit-unit itu.

Hasil final, median tiga run, 1.000.000 instruksi, release build:

| Program | mode | instructions/detik | cycles/detik | real-time |
|---|---|---:|---:|---:|
| ALU | traced | 34.385.807 | 42.982.259 | 2,686x |
| ALU | untraced | 40.184.314 | 50.230.393 | 3,139x |
| Mixed | traced | 35.348.628 | 41.775.680 | 2,611x |
| Mixed | untraced | 38.913.534 | 45.988.754 | 2,874x |
| Memory | traced | 35.832.351 | 62.706.614 | 3,919x |
| Memory | untraced | 41.166.762 | 72.041.833 | 4,503x |
| Timer0 prescaler 1 | traced | 27.368.581 | 34.210.726 | 2,138x |
| Timer0 prescaler 1 | untraced | 29.366.545 | 36.708.181 | 2,294x |
| Timer0 prescaler 64 | traced | 29.420.540 | 36.775.675 | 2,298x |
| Timer0 prescaler 64 | untraced | 32.874.932 | 41.093.665 | 2,568x |

Reproduksi:

```bash
bash scripts/bench-avr.sh 1000000
```

Real-time factor dihitung dari cycles/detik dibagi 16 MHz. Nilai di atas 1
berarti simulasi lebih cepat dari AVR 16 MHz.

Timer0/1 memakai counter integer fungsional dengan pembagian prescaler
bulk. Timer0 prescaler 1 naik dari 0,126x menjadi 2,14x-2,29x real-time.

### 3.5 Pacing 16 MHz

CLI menyediakan `emucad run <file.hex> [steps] --realtime`. GUI worker
memakai pacing yang sama secara default. Pacer memakai target cycle
absolut, sleep untuk sisa di atas 1 ms, lalu spin untuk sisa terakhir.

Verifikasi 10.000.000 instruksi, lima run, toleransi 1%:

| Program | median error | worst error |
|---|---:|---:|
| ALU | +0,301% | +0,898% |
| Mixed | +0,228% | +0,759% |
| Memory | +0,195% | +0,549% |
| Timer0 prescaler 1 | +0,509% | +0,646% |

GUI worker memakai algoritma yang sama. Release test menjalankan
16.000.000 cycle dan selesai dalam sekitar 1,00 detik dengan toleransi 1%.

Reproduksi:

```bash
bash scripts/check-avr-realtime.sh
```

Setelah optimasi, circuit CPU berisi 120 gate, 3 komponen, 48 DFF, dengan
komponen terbesar 40 gate. Timer1/Timer2 belum terhubung ke register IO
CPU, sleep wake-up belum menambah cycle, dan watchdog belum dimodelkan;
firmware yang memakai peripheral tersebut belum bisa diklaim cycle-exact.

---

## 4. Hasil

Pengukuran pada mesin pengembangan Linux, debug build, 1000 iterasi:

| Jalur | Sebelum | Sesudah |
|---|---|---|
| DC | 106.7 us/run | 36.8 - 37.1 us/run |
| Transient | 3.11 us/step | 2.90 - 2.92 us/step |

Penurunan DC sekitar 65 persen. Penurunan transient sekitar 6 persen.

Angka ini bukan jaminan untuk mesin lain.

---

## 5. Test

- `DenseMatrix::solve_owned` menghasilkan solusi yang sama.
- CLI memvalidasi jumlah iterasi.
- Format laporan benchmark diuji.
- Komponen digital independen tetap settle setelah input berubah.
- Sketsa Arduino `Serial.println("Hello World")` diverifikasi lewat
  bridge PTY setelah optimasi.
- Seluruh test workspace lulus.

Total test `sim-core`: 62.
Total test `sim-cli`: 26.
Total test `sim-gui`: 151.
Total workspace: 751.

---

## 6. Batas

- Benchmark memakai debug build.
- Tidak ada CPU pinning atau pengukuran statistik.
- Tidak ada profiling memori.
- Tidak ada perbandingan release build.
- Benchmark hanya memakai rangkaian RC linear.
- Belum ada optimasi sparse untuk transient.
