# TODO - Simulasi Semua Komponen

Status: S1-S4 selesai; S0, S5, dan S8 sebagian; S6-S7 terblokir oleh
model gate-level dan benchmark memory.

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
| Solver-backed | 379 | Ada model device |
| Behavioral | 121 | Belum punya model solver |
| Device model | 24 `DeviceKind` | Sebagian sudah ada model |
| Analisis solver | DC, transient, AC, noise, distorsi, sensitivitas | Ada |
| Netlist GUI | 1 | Ekstraksi dan DC check |
| Simulasi GUI | AVR board + analog DC | Ada |
| Mixed-signal AVR + analog | 0 | Belum ada |

## 3. Keputusan Pengguna

| ID | Keputusan |
|---|---|
| Q01 | 500 entri bertahap; milestone pertama 379 solver-backed |
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
| Q28 | Passive dan source menjadi prioritas pertama setelah 379 |
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

- [x] S0-01 Definisikan `PinRole` untuk peran listrik.
  - `PinRole` dan `device_pin_roles` ada di `sim-device`.
  - Semua device solver-backed punya role dan jumlahnya cocok
    dengan pin katalog.
  - Inspector menampilkan role per pin.
- [~] S0-02 Definisikan `ParameterSpec`.
  - Field name, default, unit, min, max, dan kind sudah ada.
  - Field required dan effect belum.
  - Effect test baru untuk resistor dan diode `n`.
- [x] S0-03 Definisikan `SimulationBinding`.
  - `SimulationBinding` menghubungkan catalog ID, `DeviceKind`, role
    pin, schema parameter, dan label fidelity.
  - `simulation_binding` dan `simulation_bindings` tersedia di
    `sim-device`.
- [x] S0-04 Definisikan aturan ground.
  - Satu net ground eksplisit; ground ganda tidak terhubung ditolak.
- [x] S0-05 Definisikan aturan supply.
  - VCC/GND adalah penanda net; sumber tegangan tetap komponen.
- [x] S0-06 Definisikan label fidelity.
  - `Fidelity`: `ideal`, `compact`, `behavioral`, `calibrated`,
    `unsupported`.
  - Setiap entry solver-backed punya label; inspector menampilkannya.
- [~] S0-07 Audit 60 model class.
  - Audit ada di `docs/63-fidelity.md` dan
    `docs/data/class-fidelity.csv`.
  - Setiap class punya family, principle, role, schema, dan label.
  - Reference vector dan status `calibrated` belum.
- [x] S0-08 Tambahkan test metadata.
  - Test binding, role unik, parameter utama, fidelity, dan klasifikasi.
- [x] S0-09 Benchmark core AVR pada 16 MHz di desktop.
  - `avr_bench` mencapai 2.1x sampai 4.8x realtime pada sampel.
  - Core masih hybrid, bukan full-chip gate-level.
- [x] S0-10 Benchmark memory macro gate-level.
  - `memory_bench` mengukur register file 32x8, 32x16, dan 32x64.
  - Hasil ada di `docs/63-fidelity.md`.
- [~] S0-11 Jalankan GPIO blink gate-level dan putuskan fallback Q55.
  - Program GPIO pada `avr_bench` berjalan 3.58x realtime.
  - Memory macro gate-level jauh di atas budget siklus; fallback Q55
    dipakai sampai model memory lebih cepat.
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
- [x] S1-05 Validasi topologi.
  - Pin floating, ground hilang, dan ground ganda ditolak.
  - Pin bertumpuk, wire stub, dan island tanpa driver ditolak.
  - Label net duplikat digabung; label berbeda pada satu net ditolak.
- [x] S1-06 Buat `Circuit` memakai `CircuitBuilder`.
- [x] S1-07 Test ekstraksi.
  - Divider, pin floating, ground hilang, ground ganda, dan unsupported lulus.
  - Paralel, junction 3 arah, label duplikat, pin bertumpuk, wire stub,
    dan island tanpa driver lulus.

**Acceptance S1:** rangkaian R-C sederhana menghasilkan node dan device
yang sama dengan referensi manual.

## 6. Fase S2 - Integrasi Solver 379 Entri

- [x] S2-01 Tambahkan dependency `sim-analysis` ke `sim-gui`.
- [x] S2-02 Tambahkan domain simulasi: `Avr`, `Analog`, `Mixed`.
  - Domain disimpan di JSON proyek dan ditampilkan di panel.
  - Analog menjalankan solver; AVR diarahkan ke board runner; Mixed
    ditolak dengan pesan belum diimplementasikan.
- [x] S2-03 Bangun circuit di worker, bukan di UI thread.
  - DC dan transient berjalan di worker analog.
  - Hasil basi diabaikan lewat request ID.
  - Solve yang sedang berjalan belum bisa diinterupsi paksa.
- [x] S2-04 Jalankan DC operating point.
- [x] S2-05 Petakan error solver ke pesan GUI.
- [~] S2-06 Tampilkan node voltage dan branch current.
  - Node voltage tampil di toolbar; branch current dan panel hasil belum.
- [x] S2-07 Tandai komponen unsupported sebelum run.
- [x] S2-08 Test rangkaian solver-backed.
  - Divider, diode clamp/rectifier, BJT, op-amp follower, relay,
    logic gate, dan transmission line lulus.
  - Relay dan logic diuji lewat `CircuitBuilder` langsung.

**Acceptance S2:** minimal 379 entri solver-backed dapat masuk netlist
dan menghasilkan DC operating point atau error yang jelas.

## 7. Fase S3 - Transient dan Probe

- [x] S3-01 Tambahkan konfigurasi transient: dt, steps, gmin, adaptive.
  - dt, steps, gmin, use_dc_op, adaptive, min/max dt, tolerance,
    dan max samples tersedia di panel.
- [x] S3-02 Tambahkan sampling hasil solver ke event worker.
  - Trace transient dikirim dari worker ke UI.
- [x] S3-03 Tambahkan probe untuk semua net voltage pada full rate.
  - Trace menyimpan semua node pada setiap langkah.
  - Grafik membatasi tampilan ke 8 series.
- [x] S3-04 Tambahkan probe branch current.
  - API terminal current tersedia untuk R, C, L, source, switch,
    relay coil, dan diode.
  - Nilai arus tampil di tabel probe pada waktu cursor.
  - Grafik arus terpisah belum.
- [x] S3-05 Tambahkan decimation min/max untuk grafik.
  - Bucket min/max menjaga spike tetap terlihat.
- [x] S3-06 Tambahkan cursor waktu dan cycle.
  - Cursor transient menampilkan waktu dan nilai net terpilih.
- [x] S3-07 Batasi buffer 10 detik simulasi untuk semua net full rate.
  - Durasi dibatasi 10 detik dan sampel dibatasi max samples.
- [x] S3-08 Test transient.
  - RC, RL, C, L, diode `cj0`, pulse width, dan rise lulus.
  - Arus resistor, source, induktor, kapasitor, dan diode lulus.
  - Diode rectifier, transmission line delay, dan BJT switching lulus.

**Acceptance S3:** pengguna dapat memilih net atau pin, menjalankan
transient, dan melihat grafik yang deterministik.

## 8. Fase S4 - UI dan Persistensi

- [x] S4-01 Panel simulasi dengan analisis, run, pause, step, reset.
  - Panel menampilkan domain, DC/Transient, run, pause, step, dan reset.
  - Transient fixed-step memakai `TransientSession` bertahap.
  - Transient adaptive masih berjalan penuh tanpa pause/step.
- [x] S4-02 Tampilkan status domain dan komponen unsupported.
  - Panel simulasi menampilkan status netlist dan error unsupported.
- [x] S4-03 Inspector parameter numerik per komponen.
  - Semua parameter schema tampil dengan min, max, dan unit.
  - Parameter boolean tampil sebagai checkbox.
- [x] S4-04 Simpan parameter dan pilihan analisis di JSON.
  - Parameter numerik, kind analisis, dan setting transient tersimpan.
  - Hasil simulasi tidak disimpan.
  - Perubahan dokumen menghapus hasil dan mengabaikan event worker lama.
- [x] S4-05 Migrasi proyek lama tanpa merusak nilai string.
  - Proyek lama memakai analysis default dan parameter kosong.
- [x] S4-06 Tampilkan nama net dan pin role.
  - Nama net tampil di hasil DC dan grafik transient.
  - Pin role tampil di inspector.
- [x] S4-07 Tampilkan error sebelum simulasi dimulai.
  - Panel simulasi menampilkan status netlist sebelum run.
- [x] S4-08 Test UI headless.
  - Panel simulasi, setting transient, dan graph dirender.
  - Pilihan analisis, domain, reload proyek, dan komponen unsupported
    di panel sudah diuji.
- [~] S4-09 Test parameter effect di GUI.
  - Test membuktikan perubahan nilai menandai proyek dirty.
  - Test DC membuktikan resistor dan diode `n` mengubah hasil.
  - Test transient membuktikan C, L, `cj0`, pulse width, dan rise.
  - Parameter AC belum diuji.

**Acceptance S4:** setiap parameter yang dapat diedit mengubah hasil
simulasi sesuai fungsi; parameter tanpa efek tidak ditampilkan.

## 9. Fase S5 - Model Compact dan Behavioral Non-MCU

- [x] S5-01 Klasifikasikan 121 entri.
  - Lihat `docs/62-behavioral-classification.md`.
  - Data per entry ada di `docs/data/behavioral-classification.csv`.
  - Kategori: compact, behavioral, digital, mixed-signal,
    structural, multidomain, RF, dan unsupported.
- [~] S5-02 Gunakan `sim-cells` untuk logic, flip-flop, counter, dan
  multiplexer bila cocok.
  - 61 entry logic dan peripheral sudah dapat dijalankan lewat
    `emucad digital`.
  - Cakupan: 8 mux/decoder/encoder, 8 adder/comparator/parity/buffer,
    10 flip-flop, 10 counter/shift, 6 level/high-Z, 3 timer, 3 digital
    source, 5 programmable logic/LUT, serta UART, SPI, I2C, dan CAN.
  - 2 entry digital lain belum: USB dan Ethernet.
- [x] S5-03 Tunda MCU dan memory ke fase S6; jangan membuat model
  sementara yang diklaim structural.
- [x] S5-04 Buat model sensor dengan input lingkungan.
  - 35 sensor sudah dimodelkan: 6 sensor suhu, 5 sensor cahaya,
    5 sensor magnetik, 5 sensor gerak, 4 sensor tekanan/gaya,
    4 sensor kelembapan/kimia, 3 sensor arus/tegangan, dan
    3 sensor proximity.
  - Seluruh entry kelas Sensor sudah solver-backed.
- [~] S5-05 Buat model actuator dengan output fisik.
  - 20 entry actuator dan display sudah solver-backed.
  - LED, lampu, buzzer, speaker, motor, solenoid, heater, Peltier,
    TEG, dan display memakai model elektrik behavioral.
  - Output fisik belum menjadi kanal simulasi terpisah.
- [~] S5-06 Buat model display dan RF hanya jika ada data referensi.
  - Display sudah memakai model beban empat pin.
  - 4 RF amplifier sudah memakai model VCVS tiga pin.
  - 13 entry DC-DC converter dan charge pump sudah solver-backed.
  - 5 entry reference dan supervisor sudah solver-backed.
  - 3 entry motor driver sudah solver-backed.
  - 2 entry sine source sudah solver-backed.
  - 4 entry timer/oscillator sudah solver-backed; PLL belum.
  - 5 entry waveform/noise source sudah solver-backed.
  - 4 entry MCU converter memakai behavioral converter.
  - 5 entry behavioral filter sudah solver-backed.
  - 5 entry behavioral math sudah solver-backed.
  - 5 entry behavioral detector sudah solver-backed.
  - 4 entry behavioral oscillator sudah solver-backed; PLL belum.
  - 4 entry behavioral converter sudah solver-backed; R-2R DAC belum.
  - Polynomial dan table voltage source sudah solver-backed.
  - Timer PLL dan behavioral PLL sudah solver-backed.
  - 14 entry mixer, filter, antenna, dan coupler menunggu
    S-parameter dan solver dua-port.
- [~] S5-07 Tambahkan label fidelity pada setiap model.
  - Setiap entry solver-backed punya label dari `SimulationBinding`.
  - Inspector menampilkan label fidelity.
  - Label `calibrated` belum dipakai karena reference vector belum ada.
- [~] S5-08 Tambahkan test per keluarga.
  - Transformer, crystal, fuse, varistor, IGBT, UJT, PUT, SCR,
    TRIAC, DIAC, GTO, MCT, SCS, bridge, dual diode, optocoupler, dan
    6 sensor suhu, 5 sensor cahaya, 5 sensor magnetik,
    5 sensor gerak, 4 sensor tekanan/gaya, 4 sensor
    kelembapan/kimia, 3 sensor arus/tegangan, 3 sensor
    proximity, 20 actuator/display, 4 RF amplifier, 13
    konverter daya, 5 reference/supervisor, 3 motor driver,
    2 sine source, 4 timer/oscillator, 5 waveform/noise source,
    5 behavioral filter, 5 behavioral math, 5 behavioral detector,
    4 behavioral oscillator, 4 behavioral converter, polynomial/table
    source, 2 PLL, dan 13 entry domain digital sudah punya test.
  - Keluarga behavioral lain belum.
- [~] S5-09 Dokumentasikan batas valid setiap model.
  - Audit kelas dan fidelity ada di `docs/63-fidelity.md`.
  - Batas model compact sudah dicatat di `docs/12-device-models.md`.
  - Batas behavioral per keluarga belum lengkap.

### 9.1 Sisa Pekerjaan Setelah S5

| Kelompok | Entri | Blocker |
|---|---:|---|
| Digital logic | 2 | 61 sudah di domain digital; 2 belum, yaitu USB dan Ethernet. |
| Structural MCU dan memory | 19 | 7 dari 15 memory sudah gate-level; butuh 11 MCU dan 8 memory lain. Scope ada di `docs/64-gate-level.md`. |
| RF | 14 | Mixer, filter, antenna, dan coupler butuh S-parameter dan solver dua-port. |
| Board module | 14 | Board-level module belum punya batas dan referensi yang cukup. |
| Mixed-signal ADC | 1 | R-2R DAC membutuhkan bus multi-bit; 4 converter lain sudah dipetakan. |
| Behavioral source | 2 | Behavioral voltage/current membutuhkan parser ekspresi bebas. |
| R-2R DAC | 1 | Butuh bus multi-bit; katalog hanya menyediakan 4 pin. |
| **Total** | **53** | |

Dari 121 entry yang belum analog solver-backed, 68 sudah dapat
dijalankan melalui domain digital. Sisa yang belum punya model sama
sekali adalah 53 entry.

**Acceptance S5:** setiap entri non-MCU yang diklaim simulatable punya
model, input, output, dan referensi yang jelas.

## 10. Fase S6 - Gate-Level Memory dan MCU

- [x] S6-01 Definisikan arti gate-level untuk 11 MCU dan 15 memory.
  - Lihat `docs/64-gate-level.md`.
  - Gate-level berarti gerbang Boolean, DFF, memory macro digital,
    interface, dan timing.
  - Bukan transistor-level atau HDL pihak ketiga.
- [~] S6-02 Tulis gate-level dari nol sebagai komponen Rust.
  - AVR memakai `sim-cells` untuk datapath dan register, tetapi kontrol
    dan peripheral masih hybrid.
  - PIC, 8051, ARM Cortex-M, RISC-V, ESP32, STM32, dan RP2040 belum.
  - Tidak memakai HDL pihak ketiga.
- [~] S6-03 Implementasikan memory gate-level.
  - Register file digital 32x8/16/64 sudah ada dan diukur.
  - Sel SRAM 6T transistor-level ada sebagai referensi.
  - Sel SRAM digital 1-bit dan array 128x8 (1024 bit) sudah ada.
  - Array non-volatile untuk EEPROM, Flash, FRAM, MRAM, dan NVRAM sudah
    ada dengan retensi setelah reset.
  - DRAM, ROM, PROM, controller, module, SPI flash, I2C EEPROM, dan
    SD card belum.
- [~] S6-04 Gunakan `sim-cells` untuk sel memory dan logic.
  - AVR datapath, register bank, ALU, dan register file memakai
    `sim-cells`.
  - Sel SRAM dan array katalog (SRAM serta non-volatile) memakai
    `sim-cells::catalog_memory`.
  - Sel memory digital lain belum.
- [~] S6-05 Tambahkan model interface dan timing.
  - Timing digital AVR dan peripheral sudah diuji sebagian.
  - Interface dan timing 11 MCU belum.
- [~] S6-06 Tambahkan instruction suite, peripheral suite, dan timing
  test per keluarga.
  - Suite AVR sudah ada.
  - Suite MCU lain belum.
- [x] S6-07 Dokumentasikan batas structural dan performa.
  - Lihat `docs/64-gate-level.md` dan `docs/63-fidelity.md`.
- [~] S6-08 Ukur waktu build dan waktu simulasi per MCU dan memory.
  - AVR dan register file sudah diukur.
  - Array SRAM 128x8 sudah diuji fungsi, belum diukur performanya.
  - MCU dan memory lain belum ada.

**Acceptance S6:** semua MCU dan memory yang dipilih memiliki model
gate-level Rust yang dapat diaudit, dengan test dan batas performa.

## 11. Fase S7 - AVR Structural dan Mixed-Signal

- [x] S7-01 Definisikan cakupan AVR structural full chip.
  - Cakupan ada di `docs/64-gate-level.md`.
  - Core, register, ALU, timer, UART, SPI, TWI, ADC, GPIO, USB, flash,
    dan SRAM tercatat.
  - Status saat ini masih hybrid.
- [~] S7-02 Siapkan model AVR gate-level dari implementasi Rust sendiri.
  - Datapath dan register sudah structural.
  - Kontrol dan peripheral belum full gate-level.
  - Tidak memakai HDL pihak ketiga.
- [ ] S7-03 Definisikan scheduler digital-analog.
  - Belum ada scheduler deterministik AVR + analog.
- [ ] S7-04 Buat wrapper AVR structural sebagai device digital.
  - Belum ada device digital di solver analog.
- [~] S7-05 Petakan pin AVR ke node netlist.
  - Board simulation memetakan PB5 dan pin LED.
  - Pemetaan penuh 32 pin belum.
- [ ] S7-06 Hubungkan pin electrical sebagai Thevenin.
  - Belum diimplementasikan untuk semua pin.
- [ ] S7-07 Tambahkan breakpoint saat pin berubah.
  - Belum ada breakpoint pin di co-sim.
- [~] S7-08 Hubungkan ADC, PWM, timer, dan UART.
  - Peripheral AVR sudah ada di `sim-avr`.
  - Koneksi ke solver analog belum.
- [ ] S7-09 Jaga determinisme urutan event.
  - Belum ada event queue gabungan digital-analog.
- [x] S7-10 Test AVR + LED + resistor + voltage source.
  - Board test dan GUI simulation sudah mencakup LED, resistor, dan PB5.
- [x] S7-11 Jalankan GPIO blink gate-level sebagai vertical slice.
  - Program GPIO pada `avr_bench` berjalan 3.58x realtime.
  - Core masih hybrid; bukan full-chip gate-level.
- [x] S7-12 Ukur performa dan latency pada desktop CPU.
  - Hasil ada di `docs/63-fidelity.md`.
- [~] S7-13 Verifikasi real-time semua MCU atau aktifkan fallback Q55.
  - AVR dan memory macro diukur.
  - Semua MCU belum; fallback Q55 dipakai.

**Acceptance S7:** AVR structural dan analog berjalan dalam satu
simulasi dengan sumber gate-level yang dapat diaudit.

## 12. Fase S8 - Verifikasi dan Rilis

- [~] S8-01 Buat reference circuit untuk setiap keluarga.
  - Divider, RC, dan RL punya reference analitik di
    `crates/sim-analysis/tests/reference.rs`.
  - Keluarga lain memakai test unit, belum semua punya reference
    analitik terpadu.
- [~] S8-02 Bandingkan dengan solusi analitik.
  - Divider, RC, dan RL dibandingkan dengan solusi analitik.
  - Perbandingan lengkap per keluarga belum.
- [~] S8-03 Sediakan CI container dengan ngspice dan golden test wajib.
  - `ci/ngspice.Dockerfile`, `scripts/ngspice_golden.sh`, dan job CI
    sudah ada.
  - Job belum dieksekusi lokal karena akses Docker socket ditolak;
    hasil CI belum diverifikasi.
- [~] S8-04 Tambahkan instruction suite, peripheral suite, dan timing test.
  - Suite AVR sudah ada di `sim-avr`.
  - Timing suite lintas MCU belum.
- [~] S8-05 Ukur performa real-time semua MCU di desktop dan akurasi 500 komponen.
  - AVR dan memory macro sudah diukur.
  - 11 MCU dan 500 komponen belum diukur seluruhnya.
- [~] S8-06 Perbarui docs, README, dan TODO.
  - Fidelity, benchmark, reference circuit, dan persistensi sudah
    diperbarui.
  - Dokumentasi per keluarga behavioral belum lengkap.
  - Fidelity, benchmark, dan batas real-time sudah diperbarui.
  - Dokumentasi per keluarga behavioral belum lengkap.
- [x] S8-07 Simpan hasil transient di JSON + file biner.
  - Metadata JSON dan sampel biner ditulis sebagai sidecar proyek.
  - Trace dipulihkan saat proyek dibuka; sidecar lama dihapus saat
    hasil baru tidak ada.
- [~] S8-08 Jalankan fmt, clippy, test, ASCII, package, dan CI ngspice.
  - fmt, clippy, test, dan ASCII lulus.
  - Package dan CI ngspice belum dijalankan pada mesin ini.

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

- [ ] Minimal 379 entri solver-backed dapat disimulasikan pada milestone
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
