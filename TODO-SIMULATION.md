# TODO - Simulasi Semua Komponen

Status: analisis selesai, implementasi belum dimulai.

Tujuan: membuat komponen di skematik GUI dapat disimulasikan dengan model
fisika atau compact model yang terdokumentasi, bukan hanya AVR board.

## 1. Prinsip

- Compact model yang menjelaskan persamaan, parameter, dan batas valid
  dihitung sebagai simulasi fisik.
- Model behavioral harus diberi label fidelity, bukan diklaim sebagai
  fisika penuh.
- Tidak ada hasil simulasi yang dikarang.
- Komponen tanpa model harus ditolak atau ditandai, bukan diberi angka
  palsu.
- Semua hasil harus deterministik dan dapat diuji.
- GUI tetap adapter; solver tetap di `sim-analysis` dan model device tetap
  di `sim-device`.

## 2. Baseline

| Item | Jumlah | Status |
|---|---:|---|
| Entri katalog | 500 | Ada |
| Solver-backed | 215 | Ada model device |
| Behavioral | 285 | Belum punya model solver |
| Device model | 24 `DeviceKind` | Sebagian sudah ada model |
| Analisis solver | DC, transient, AC, noise, distorsi, sensitivitas | Ada |
| Netlist GUI | 1 | Ekstraksi dan DC check |
| Simulasi GUI | AVR board + analog DC | Ada |
| Mixed-signal AVR + analog | 0 | Belum ada |

## 3. Keputusan Pengguna

| ID | Keputusan |
|---|---|
| Q01 | 500 entri bertahap; milestone pertama 215 solver-backed |
| Q02 | Compact model dengan parameter default per kelas |
| Q03 | Model behavioral boleh dipakai dengan label fidelity |
| Q04 | Co-sim AVR dan analog dikerjakan paling akhir |
| Q05 | Analisis pertama: DC operating point dan transient |
| Q06 | Ground wajib memakai komponen GND |
| Q07 | Supply memakai voltage source eksplisit; VCC hanya penanda net |
| Q08 | Peran pin berasal dari metadata katalog |
| Q09 | Schema parameter per kelas dan nilai numerik per instance |
| Q10 | Komponen tanpa model diblokir dengan error jelas |
| Q11 | Sensor dan actuator ditunda ke fase behavioral |
| Q12 | RF block ditunda ke fase AC/RF |
| Q13 | Memory dan MCU memakai structural penuh |
| Q14 | Switch dan relay memakai model ideal + delay sederhana |
| Q15 | Adapter netlist ditempatkan di modul `sim-gui` |
| Q16 | Parameter dan pilihan analisis disimpan di JSON proyek |
| Q17 | Transient memakai adaptive step dengan batas aman |
| Q18 | Validasi wajib menambahkan golden ngspice |
| Q19 | Probe tersedia untuk semua net |
| Q20 | Semua keluarga masuk roadmap; urutan eksekusi perlu ditentukan |
| Q21 | Input lingkungan didefinisikan per komponen |
| Q22 | Kontrol simulasi ditempatkan di panel bawah |
| Q23 | Prioritas utama adalah akurasi, bukan target 60 FPS |
| Q24 | Error simulasi memakai fail fast |
| Q25 | Hasil simulasi wajib deterministik |
| Q26 | Structural penuh berarti gate-level untuk semua MCU dan memory |
| Q27 | ngspice diinstal lokal |
| Q28 | Passive dan source menjadi prioritas pertama setelah 215 |
| Q29 | Board, enclosure, mounting, dan module boleh unsupported |
| Q30 | Semua net diprobe pada full rate |
| Q31 | AVR memakai structural penuh |
| Q32 | Gate-level MCU dan memory ditulis dari nol |
| Q33 | AVR full chip termasuk ADC, SPI, TWI, dan USB |
| Q34 | Probe semua net full rate dibatasi 10 detik simulasi |
| Q35 | Simulasi gate-level semua MCU harus real-time |
| Q36 | Semua 11 MCU wajib gate-level; tidak boleh subset |
| Q37 | ngspice dijalankan lewat CI container |
| Q38 | Sumber pihak ketiga harus open-source |
| Q39 | Prioritas MCU: AVR, PIC/8051, ARM, RISC-V, ESP32, RP2040 |
| Q40 | Prioritas memory: SRAM/DRAM, ROM/PROM, EEPROM/Flash, FRAM/MRAM, NVRAM, controller |
| Q41 | Hasil transient disimpan di JSON + file biner |
| Q42 | Real-time semua MCU wajib dipertahankan |
| Q43 | Gate-level ditulis dari nol sebagai komponen Rust, tanpa HDL |
| Q44 | 11 MCU dikerjakan bertahap, subset dulu |
| Q45 | AVR full chip pada milestone gate-level pertama |
| Q46 | Semua 15 memory wajib pada milestone pertama |
| Q47 | Real-time berlaku untuk semua MCU |
| Q48 | Pekerjaan dipecah menjadi beberapa milestone tahunan |
| Q49 | Hanya implementasi sendiri, tanpa HDL pihak ketiga |
| Q50 | Gate-level berarti gate-level logika, bukan transistor |
| Q51 | Hardware target real-time adalah desktop CPU |
| Q52 | MCU subset pertama adalah AVR |
| Q53 | Memory pertama adalah SRAM |
| Q54 | Vertical slice pertama adalah GPIO blink |
| Q55 | Jika benchmark gagal, real-time boleh diturunkan |
| Q56 | Verifikasi memakai instruction, peripheral, dan timing |
| Q57 | Perbaikan data dan mapping dikerjakan sebelum harness |
| Q58 | Unit test model di sim-device; circuit test di sim-analysis |
| Q59 | Reference vector memakai CSV sumber dan Rust const |
| Q60 | 500 model generik ditambah 50 part-specific prioritas |
| Q61 | Target sebelum simulasi adalah C2 verified |
| Q62 | Setiap parameter numerik wajib mengubah hasil sesuai fungsi |

### 3.1 Target dan Fallback

Target maksimum:

- Semua 11 MCU gate-level logika.
- AVR full chip termasuk ADC/SPI/TWI/USB.
- Semua 15 memory gate-level.
- Gate-level ditulis dari nol sebagai komponen Rust.
- Real-time di desktop CPU.
- Beberapa milestone tahunan.

Urutan awal:

1. AVR sebagai MCU pertama.
2. SRAM sebagai memory pertama.
3. GPIO blink sebagai vertical slice pertama.
4. Instruction, peripheral, dan timing test sebagai verifikasi.

Catatan konflik:

- Q42 dan Q47 menetapkan real-time semua MCU sebagai target.
- Q55 mengizinkan real-time diturunkan jika benchmark gagal.
- Keputusan yang dipakai: real-time adalah target, bukan jaminan sebelum
  benchmark. Jika benchmark gagal, real-time diturunkan dan hasilnya
  dicatat.

Gerbang kelayakan sebelum implementasi besar:

1. Benchmark AVR gate-level pada clock 16 MHz di desktop.
2. Benchmark SRAM gate-level.
3. Jalankan GPIO blink gate-level.
4. Ukur event simulation Rust.
5. Jika real-time tidak tercapai, aktifkan fallback Q55.
6. Jangan mengklaim semua MCU real-time sebelum benchmark.

## 4. Fase S0 - Kontrak dan Metadata

- [ ] S0-01 Definisikan `PinRole` untuk peran listrik.
  - Contoh: anode, cathode, base, collector, emitter, gate, drain,
    source, body, inverting, non-inverting, supply, output, control,
    common, normally-open, normally-closed.
  - Setiap definisi komponen wajib memiliki role per pin.
- [~] S0-02 Definisikan `ParameterSpec`.
  - Field name, default, unit, min, max, dan kind sudah ada.
  - Field required dan effect belum.
  - Effect test baru untuk resistor dan diode `n`.
- [ ] S0-03 Definisikan `SimulationBinding`.
  - Menghubungkan definition ID ke catalog ID, `DeviceKind`, role pin,
    dan parameter default.
- [ ] S0-04 Definisikan aturan ground.
  - Satu net ground yang eksplisit.
  - Deteksi ground ganda yang tidak terhubung.
- [ ] S0-05 Definisikan aturan supply.
  - VCC/GND symbol adalah penanda net.
  - Sumber tegangan tetap komponen terpisah.
- [ ] S0-06 Definisikan label fidelity.
  - `ideal`, `compact`, `behavioral`, `calibrated`, `unsupported`.
- [ ] S0-07 Audit 61 model class.
  - Setiap class harus punya role pin, parameter schema, dan status
    simulasi.
- [ ] S0-08 Tambahkan test metadata.
- [ ] S0-09 Benchmark core AVR gate-level pada 16 MHz di desktop.
- [ ] S0-10 Benchmark SRAM macro gate-level.
- [ ] S0-11 Jalankan GPIO blink gate-level dan putuskan fallback Q55.
  - Semua definisi solver-backed punya binding.
  - Tidak ada duplicate role pada satu definition.
  - Semua parameter default valid.

**Acceptance S0:** setiap komponen solver-backed punya binding lengkap,
dan komponen unsupported punya alasan yang dapat ditampilkan.

## 5. Fase S1 - Ekstraksi Netlist

- [x] S1-01 Buat union-find untuk pin, wire vertex, dan junction.
- [x] S1-02 Petakan setiap pin ke node listrik.
- [x] S1-03 Beri nama net stabil.
  - Prioritaskan label wire.
  - Jika tidak ada label, gunakan class dan index.
- [x] S1-04 Deteksi ground dan supply dari komponen atau net class.
- [~] S1-05 Validasi topologi.
  - Pin floating sudah ditolak.
  - Ground hilang dan ground ganda sudah ditolak.
  - Net tanpa driver, pin bertumpuk, dan wire stub belum.
- [x] S1-06 Buat `Circuit` memakai `CircuitBuilder`.
- [~] S1-07 Test ekstraksi.
  - Divider, pin floating, ground hilang, ground ganda, dan unsupported lulus.
  - Paralel, junction 3 arah, dan label net duplikat belum.

**Acceptance S1:** rangkaian R-C sederhana menghasilkan node dan device
yang sama dengan referensi manual.

## 6. Fase S2 - Integrasi Solver 215 Entri

- [x] S2-01 Tambahkan dependency `sim-analysis` ke `sim-gui`.
- [ ] S2-02 Tambahkan domain simulasi: `Avr`, `Analog`, `Mixed`.
- [ ] S2-03 Bangun circuit di worker, bukan di UI thread.
- [x] S2-04 Jalankan DC operating point.
- [x] S2-05 Petakan error solver ke pesan GUI.
- [~] S2-06 Tampilkan node voltage dan branch current.
  - Node voltage tampil di toolbar; branch current dan panel hasil belum.
- [x] S2-07 Tandai komponen unsupported sebelum run.
- [~] S2-08 Test rangkaian solver-backed.
  - Resistor divider DC lulus.
  - Diode clamp, BJT, op-amp, relay, logic, dan transmission line belum.

**Acceptance S2:** minimal 215 entri solver-backed dapat masuk netlist
dan menghasilkan DC operating point atau error yang jelas.

## 7. Fase S3 - Transient dan Probe

- [ ] S3-01 Tambahkan konfigurasi transient: dt, steps, gmin, adaptive.
- [ ] S3-02 Tambahkan sampling hasil solver ke event worker.
- [ ] S3-03 Tambahkan probe untuk semua net voltage pada full rate.
- [ ] S3-04 Tambahkan probe branch current.
- [ ] S3-05 Tambahkan decimation min/max untuk grafik.
- [ ] S3-06 Tambahkan cursor waktu dan cycle.
- [ ] S3-07 Batasi buffer 10 detik simulasi untuk semua net full rate.
- [ ] S3-08 Test transient.
  - RC charge/discharge.
  - Diode rectifier.
  - Pulse source.
  - BJT switch delay.
  - Transmission line delay.

**Acceptance S3:** pengguna dapat memilih net atau pin, menjalankan
transient, dan melihat grafik yang deterministik.

## 8. Fase S4 - UI dan Persistensi

- [ ] S4-01 Panel simulasi dengan analisis, run, pause, step, reset.
- [ ] S4-02 Tampilkan status domain dan komponen unsupported.
- [~] S4-03 Inspector parameter numerik per komponen.
  - Semua parameter schema tampil dengan min, max, dan unit.
  - Parameter boolean tampil sebagai checkbox.
  - Effect test baru untuk resistor dan diode `n`.
- [~] S4-04 Simpan parameter dan pilihan analisis di JSON.
  - Nilai parameter numerik sudah tersimpan per instance.
  - Pilihan analisis dan hasil terstruktur belum.
- [ ] S4-05 Migrasi proyek lama tanpa merusak nilai string.
- [ ] S4-06 Tampilkan nama net dan pin role.
- [ ] S4-07 Tampilkan error sebelum simulasi dimulai.
- [ ] S4-08 Test UI headless.
  - Panel render.
  - Pilihan analisis.
  - Komponen unsupported.
  - Reload proyek.
- [~] S4-09 Test parameter effect di GUI.
  - Test membuktikan perubahan nilai menandai proyek dirty.
  - Test DC membuktikan resistor dan diode `n` mengubah hasil.
  - Parameter transient dan AC belum diuji.

**Acceptance S4:** setiap parameter yang dapat diedit mengubah hasil
simulasi sesuai fungsi; parameter tanpa efek tidak ditampilkan.

## 9. Fase S5 - Model Compact dan Behavioral Non-MCU

- [ ] S5-01 Klasifikasikan 285 entri.
  - Dapat dimodelkan dengan compact model.
  - Perlu model digital.
  - Perlu model multidomain.
  - Tidak dapat disimulasikan dengan data saat ini.
- [ ] S5-02 Gunakan `sim-cells` untuk logic, flip-flop, counter, dan
  multiplexer bila cocok.
- [ ] S5-03 Tunda MCU dan memory ke fase S6; jangan membuat model
  sementara yang diklaim structural.
- [ ] S5-04 Buat model sensor dengan input lingkungan.
- [ ] S5-05 Buat model actuator dengan output fisik.
- [ ] S5-06 Buat model display dan RF hanya jika ada data referensi.
- [ ] S5-07 Tambahkan label fidelity pada setiap model.
- [ ] S5-08 Tambahkan test per keluarga.
- [ ] S5-09 Dokumentasikan batas valid setiap model.

**Acceptance S5:** setiap entri non-MCU yang diklaim simulatable punya
model, input, output, dan referensi yang jelas.

## 10. Fase S6 - Gate-Level Memory dan MCU

- [ ] S6-01 Definisikan arti gate-level untuk 11 MCU dan 15 memory.
  - Netlist gate-level logika.
  - Memory macro dan peripheral.
  - Interface dan timing.
- [ ] S6-02 Tulis gate-level dari nol sebagai komponen Rust.
  - AVR menjadi MCU pertama.
  - PIC, 8051, ARM Cortex-M, RISC-V, ESP32, STM32, RP2040 menyusul.
  - Semua 11 MCU wajib pada akhir roadmap.
  - Tidak memakai HDL pihak ketiga.
- [ ] S6-03 Implementasikan memory gate-level.
  - SRAM menjadi memory pertama.
  - DRAM, ROM, PROM, EEPROM, Flash, FRAM, MRAM, NVRAM, controller,
    dan module menyusul.
  - Semua 15 memory wajib pada akhir roadmap.
- [ ] S6-04 Gunakan `sim-cells` untuk sel memory dan logic.
- [ ] S6-05 Tambahkan model interface dan timing.
- [ ] S6-06 Tambahkan instruction suite, peripheral suite, dan timing
  test per keluarga.
- [ ] S6-07 Dokumentasikan batas structural dan performa.
- [ ] S6-08 Ukur waktu build dan waktu simulasi per MCU dan memory.

**Acceptance S6:** semua MCU dan memory yang dipilih memiliki model
gate-level Rust yang dapat diaudit, dengan test dan batas performa.

## 11. Fase S7 - AVR Structural dan Mixed-Signal

- [ ] S7-01 Definisikan cakupan AVR structural full chip.
  - Core, register file, ALU, control, flash, SRAM.
  - Timer, UART, SPI, TWI, ADC, GPIO, dan USB.
  - Semua peripheral wajib structural.
- [ ] S7-02 Siapkan model AVR gate-level dari implementasi Rust sendiri.
  - Tidak memakai HDL pihak ketiga.
  - Gunakan datasheet, ISA manual, dan referensi arsitektur.
- [ ] S7-03 Definisikan scheduler digital-analog.
- [ ] S7-04 Buat wrapper AVR structural sebagai device digital.
- [ ] S7-05 Petakan pin AVR ke node netlist.
- [ ] S7-06 Hubungkan pin electrical sebagai Thevenin.
- [ ] S7-07 Tambahkan breakpoint saat pin berubah.
- [ ] S7-08 Hubungkan ADC, PWM, timer, dan UART.
- [ ] S7-09 Jaga determinisme urutan event.
- [ ] S7-10 Test AVR + LED + resistor + voltage source.
- [ ] S7-11 Jalankan GPIO blink gate-level sebagai vertical slice.
- [ ] S7-12 Ukur performa dan latency pada desktop CPU.
- [ ] S7-13 Verifikasi real-time semua MCU atau aktifkan fallback Q55.

**Acceptance S7:** AVR structural dan analog berjalan dalam satu
simulasi dengan sumber gate-level yang dapat diaudit.

## 12. Fase S8 - Verifikasi dan Rilis

- [ ] S8-01 Buat reference circuit untuk setiap keluarga.
- [ ] S8-02 Bandingkan dengan solusi analitik.
- [ ] S8-03 Sediakan CI container dengan ngspice dan golden test wajib.
- [ ] S8-04 Tambahkan instruction suite, peripheral suite, dan timing test.
- [ ] S8-05 Ukur performa real-time semua MCU di desktop dan akurasi 500 komponen.
- [ ] S8-06 Perbarui docs, README, dan TODO.
- [ ] S8-07 Simpan hasil transient di JSON + file biner.
- [ ] S8-08 Jalankan fmt, clippy, test, ASCII, package, dan CI ngspice.

**Acceptance S8:** semua test lulus, hasil dapat direproduksi, dan
dokumentasi menyebut batas fidelity.

## 13. Risiko

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Pin role salah | Hasil terlihat benar tetapi salah | Audit per class, test referensi |
| Parameter default generik | Semua part dalam kelas identik | Tambah parameter per part |
| Model behavioral dianggap fisika | Klaim berlebihan | Label fidelity |
| Ground ambigu | Solver singular | Validasi ground eksplisit |
| Floating node | Convergence failure | Validasi sebelum run |
| AVR + analog scheduler salah | Deadlock atau drift waktu | Event queue deterministik |
| Multidomain sensor/actuator | Model tidak lengkap | Input eksplisit dan batas valid |
| Performa transient | UI tidak responsif | Worker, adaptive step, batas sample |
| Gate-level semua MCU | Tidak ada netlist gate-level di repo | Tulis dari nol adalah proyek besar |
| Real-time semua MCU | Hardware target belum ditentukan | Benchmark sebelum implementasi |
| Semua 15 memory | Beban verifikasi besar | Urutan dan test suite |
| AVR full chip | ADC/SPI/TWI/USB structural menambah beban besar | Tentukan subset atau milestone |
| Semua net full rate | RAM dan waktu render besar | Buffer 10 detik dan batas memori |
| ngspice tidak tersedia | Golden test tidak berjalan | CI container dengan ngspice |
| Semua net diprobe | Memori dan grafik berat | Sampling selektif dan decimation |

## 14. Non-Goals

- TCAD atau simulasi level transistor silikon.
- Semua 500 entri langsung menjadi model datasheet.
- Mixed-signal penuh pada fase pertama.
- RF S-parameter penuh pada fase pertama.
- Sensor dan actuator multidomain penuh pada fase pertama.

## 15. Definition of Done

- [ ] Minimal 215 entri solver-backed dapat disimulasikan pada milestone
  pertama.
- [ ] Seluruh 500 entri memiliki status simulasi yang jelas.
- [ ] Netlist GUI tervalidasi.
- [ ] DC dan transient berjalan di worker.
- [ ] Semua net dapat diprobe dengan buffer 10 detik simulasi.
- [ ] Parameter dapat diedit dan disimpan di JSON.
- [ ] Setiap parameter numerik punya effect test dan mengubah hasil.
- [ ] Tidak ada parameter editable yang tidak dipakai model.
- [ ] Semua 11 MCU gate-level logika selesai.
- [ ] Semua 15 memory gate-level selesai.
- [ ] AVR full chip termasuk ADC/SPI/TWI/USB selesai.
- [ ] GPIO blink gate-level menjadi vertical slice pertama.
- [ ] Instruction, peripheral, dan timing test lulus.
- [ ] Target real-time diukur di desktop; fallback dicatat bila gagal.
- [ ] Golden ngspice berjalan di CI container.
- [ ] Hasil transient disimpan di JSON + file biner.
- [ ] Dokumentasi fidelity diperbarui.
