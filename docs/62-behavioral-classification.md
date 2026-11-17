# 62 - Klasifikasi Entry Behavioral

Dokumen ini mengklasifikasikan 121 entry katalog yang belum memiliki
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
| `behavioral-electrical` | 3 |
| `compact-analog` | 0 |
| `structural` | 26 |
| `rf` | 14 |
| `unsupported` | 14 |
| `mixed-signal` | 1 |
| **Total** | **121** |

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
   - 35 sensor dan 20 actuator/display sudah solver-backed.
   - Actuator/display memakai model elektrik behavioral; output
     fisik belum dirender sebagai kanal terpisah.
   - 4 RF amplifier memakai model VCVS tiga pin.
   - 13 entry DC-DC converter dan charge pump memakai model
     teregulasi dengan efisiensi.
   - 5 entry reference dan supervisor memakai sumber atau
     threshold tiga pin.
   - 3 entry motor driver memakai switch satu kanal.
   - 2 entry sine source memakai model sinusoidal.
   - 4 entry timer/oscillator memakai model 8 pin; PLL belum.
   - 5 entry waveform/noise source memakai model waktu atau PRNG.
   - 5 entry behavioral filter memakai state dan time constant.
   - 5 entry behavioral math memakai operasi nonlinier.
   - 5 entry behavioral detector memakai threshold dan state.
   - 4 entry behavioral oscillator memakai phase dan modulasi; PLL belum.
   - 4 entry behavioral converter memakai sampling dan kuantisasi.
   - 4 entry MCU converter memakai model behavioral converter.
   - 75 entry dapat dijalankan di domain digital: 61 logic/peripheral
     dan 14 memory gate-level.
   - Polynomial dan table source memakai model waktu.
   - Dua PLL memakai frequency multiplier dengan phase reset.
   - 14 entry RF lain menunggu S-parameter dan solver dua-port.
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

- CSV mencakup tepat 121 entry behavioral.
- Setiap entry punya satu kategori yang valid.
- Setiap kategori punya fase dan prioritas.
- Tidak ada entry yang hilang atau duplikat.
