# 62 - Klasifikasi Entry Behavioral

Dokumen ini mengklasifikasikan 214 entry katalog yang belum memiliki
model solver. Tujuannya menentukan model apa yang harus dibuat pada S5
dan mana yang harus menunggu fase lain.

Data per entry ada di
[`data/behavioral-classification.csv`](data/behavioral-classification.csv).

## 1. Kategori

| Kategori | Arti |
|---|---|
| `compact-analog` | Model listrik compact dengan persamaan dan parameter fisik. |
| `behavioral-electrical` | Blok behavioral dengan input-output listrik dan label fidelity. |
| `digital` | Model digital memakai `sim-cells` atau blok peripheral. |
| `mixed-signal` | Membutuhkan konversi analog-digital dan timing. |
| `structural` | Gate-level penuh pada S6. |
| `multidomain` | Membutuhkan input lingkungan atau output fisik. |
| `rf` | Membutuhkan AC, S-parameter, atau model RF. |
| `unsupported` | Tidak disimulasikan; alasan harus ditampilkan. |

## 2. Jumlah

| Kategori | Jumlah |
|---|---:|
| `digital` | 63 |
| `behavioral-electrical` | 62 |
| `multidomain` | 26 |
| `compact-analog` | 0 |
| `structural` | 26 |
| `rf` | 18 |
| `unsupported` | 14 |
| `mixed-signal` | 5 |
| **Total** | **214** |

## 3. Prioritas

| Prioritas | Fase | Isi |
|---|---|---|
| P1 | S5-1 | Transformer, crystal, diode array, protection, thyristor, power transistor. |
| P2 | S5-2 | Timer, optocoupler, DC-DC, charge pump, supervisor, motor driver, behavioral block. |
| P3 | S5-3 | Logic special, flip-flop, counter, mux, adder, level shifter, programmable logic, digital source, peripheral digital. |
| P4 | S5-4 | ADC dan konversi mixed-signal. |
| P5 | S5-5 | Sensor dan actuator multidomain. |
| P6 | S6 | Memory dan MCU gate-level. |
| P7 | S5-6 | RF block setelah jalur AC tersedia. |
| P0 | none | Board, enclosure, dan mechanical. |

## 4. Keluarga Pertama

Urutan implementasi yang disarankan:

1. Transformer dan crystal. Selesai.
   - Model mutual inductance dan BVD.
   - Dibutuhkan oleh pasif dan osilator.
2. Protection dan thyristor. Selesai.
   - Fuse dan varistor memakai model protection.
   - SCR/LASCR, TRIAC, DIAC, GTO/MCT, dan SCS memakai model thyristor.
   - PUT thyristor memakai model programmable unijunction.
3. Power transistor. Selesai pada level model.
   - IGBT memakai model IGBT dengan body diode.
   - GaN, SiC, MESFET, dan HEMT memakai JFET atau MOSFET generik.
   - UJT dan PUT memakai model unijunction latching.
4. Behavioral electrical. Sebagian.
   - Optocoupler selesai dengan model CTR dan output saturation.
   - Timer, DC-DC, charge pump, supervisor, dan motor driver belum.
5. Digital.
   - Gunakan `sim-cells` untuk flip-flop, counter, mux, adder.
6. Sensor dan actuator. Sebagian.
   - 29 sensor sudah memakai model behavioral: 6 sensor suhu,
     5 sensor cahaya, 5 sensor magnetik, 5 sensor gerak,
     4 sensor tekanan/gaya, dan 4 sensor kelembapan/kimia.
   - 6 sensor lain dan actuator/display belum.
7. RF.
   - Tunda sampai AC dan S-parameter tersedia.

P1 compact-analog selesai pada level model. Tidak ada entry
`compact-analog` yang tersisa di klasifikasi.

## 5. Aturan

- Setiap model wajib punya label fidelity.
- Model behavioral tidak boleh diklaim sebagai fisika penuh.
- Parameter yang tampil wajib punya effect test.
- Entry yang belum punya model tetap unsupported dan tidak diberi
  hasil palsu.
- Model digital harus deterministik dan tidak memakai wall-clock.

## 6. Acceptance S5-01

- CSV mencakup tepat 214 entry behavioral.
- Setiap entry punya satu kategori yang valid.
- Setiap kategori punya fase dan prioritas.
- Tidak ada entry yang hilang atau duplikat.
