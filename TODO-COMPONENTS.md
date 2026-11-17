# TODO - Komponen 500

Status per 2026-11-16: katalog 500 entri ada, 363 entri dapat
diinstansiasi lewat registry, dan 137 entri belum punya model solver.
Tidak ada entri yang memenuhi seluruh syarat batch `verified`. Sebagian
dari 363 mapping masih generik atau salah topologi, jadi `mapped` tidak
sama dengan `selesai`. Binding Python `sim-python` menyediakan katalog
dan builder topologi; simulasi tetap berjalan di crate Rust.

Tujuan: membangun katalog 500 model komponen yang terverifikasi, dapat
diekspor ke Python builder, dan diuji dengan unit test ketat.

Dokumen desain: `docs/60-component-catalog.md`.

## 1. Rekomendasi

Mulai dari 80 komponen inti analog dan digital, bukan langsung 500.
Setelah pipeline katalog, test, dan referensi stabil, tambah batch 20
sampai 30 komponen sampai mencapai 500.

Urutan yang disarankan:

1. Passives dan sources.
2. Diode, BJT, JFET, MOSFET.
3. Proteksi, switch, relay.
4. Op-amp, comparator, timer.
5. Logic, memory, MCU peripheral.
6. Power, sensor, actuator.
7. RF, behavioral, board module.
8. Part-specific models.
9. Python builder.

## 2. Keputusan yang Harus Disetujui

- [x] C0-01 Setujui bahwa 500 entri berarti 500 definisi model generik,
  bukan 500 part number.
- [x] C0-02 Setujui tingkat model T0 sampai T5 di `docs/60`.
- [x] C0-03 Setujui toleransi referensi per tingkat model.
- [x] C0-04 Setujui schema katalog dan status draft sampai verified.
- [x] C0-05 Setujui kebijakan sumber dan lisensi clean-room tanpa salin GPL.
- [x] C0-06 Pilih analisis pertama: DC operating point dan transient.
- [x] C0-07 Tentukan rasio model generik dan part-specific.
  - 500 model generik, ditambah 50 part-specific prioritas di C10.
- [x] C0-08 Tentukan lingkup Python builder: batch atau interaktif.
- [x] C0-09 Setujui lokasi katalog di `sim-device` tanpa crate baru.
- [x] C0-10 Setujui nama, unit SI, dan aturan versioning.

## 3. Aturan Batch

Setiap batch dinyatakan selesai jika:

- Entri katalog memiliki prinsip kerja satu paragraf.
- Parameter memiliki unit, default, min, dan max.
- Setiap parameter numerik dibaca model dan punya effect test.
- Tidak ada parameter dekoratif yang tidak memengaruhi hasil.
- Model Rust deterministic dan tidak memakai wall-clock.
- Minimal tiga unit test: nominal, batas, dan error.
- Minimal satu conservation atau energy test.
- Minimal satu reference vector dari datasheet atau simulator.
- Referensi dan lisensi dicatat.
- Status berubah menjadi `verified`.
- Dokumentasi dan contoh diperbarui.

Batch yang gagal salah satu syarat tetap berstatus `tested`, bukan
`verified`.

## 3a. Keputusan Pendekatan

| ID | Keputusan |
|---|---|
| D-01 | Perbaikan data dan mapping dikerjakan sebelum harness verifikasi. |
| D-02 | Unit test model di `sim-device`; circuit dan conservation test di `sim-analysis`. |
| D-03 | Reference vector memakai CSV sumber dan Rust const hasil generator. |
| D-04 | Rasio akhir: 500 model generik dan 50 part-specific prioritas. |
| D-05 | Target sebelum kembali ke TODO-SIMULATION adalah C2 verified. |
| D-06 | Setiap parameter numerik wajib berfungsi dan punya effect test. |

## 3b. Aturan Parameter

Parameter yang terlihat dan dapat diedit pengguna wajib memenuhi aturan:

- Nilai numerik dibaca model saat stamping atau update.
- Mengubah nilai harus mengubah hasil simulasi sesuai fungsi fisiknya.
- Default adalah nilai awal, bukan nilai tetap.
- Parameter yang belum berfungsi tidak boleh tampil sebagai editable.
- Parameter yang hanya berlaku pada region tertentu wajib punya test
  yang memasuki region tersebut.
- Konstanta fisik seperti Boltzmann dan elementary charge bukan parameter
  pengguna dan tidak ditampilkan.
- Nilai parameter disimpan di proyek JSON dan menginvalidasi hasil lama
  ketika berubah.

Setiap parameter minimal punya:

- Test nominal pada nilai default.
- Test sensitivity: nilai diubah dalam rentang valid, output berubah
  dengan arah dan besaran yang diharapkan.
- Test boundary: nilai min dan max valid; nilai di luar rentang ditolak.
- Test no-op guard: parameter yang tidak dipakai model harus dihapus
  atau ditandai unsupported.

## 4. Infrastruktur

- [x] C1-01 Buat modul katalog di `crates/sim-device`.
- [~] C1-02 Definisikan struct `ComponentSpec` dan parameter typed.
  - Struct ada, tetapi parameter masih satu nama string plus default.
  - Schema multi-parameter dan tipe belum ada.
- [~] C1-03 Buat validator ID, unit, pin, dan rentang parameter.
  - ID, teks, pin positif, dan default finite sudah divalidasi.
  - Unit, pin per kelas, min, dan max belum divalidasi.
- [x] C1-04 Buat loader katalog deterministik.
- [x] C1-05 Tambah perintah `emucad catalog --json`.
- [ ] C1-06 Buat format reference vector dan parser test.
- [ ] C1-07 Buat harness unit test model.
- [ ] C1-08 Buat harness conservation test KCL, KVL, dan energi.
- [ ] C1-09 Buat harness convergence dan parameter fuzz.
- [ ] C1-10 Buat cross-check opsional terhadap ngspice.
- [x] C1-11 Tambah CI Linux untuk katalog dan test model.
- [~] C1-12 Tambah laporan coverage per kategori dan status.
  - CLI baru melaporkan total; per kategori dan status belum.
- [x] C1-21 Buat crate `sim-python` dengan `pyo3` untuk katalog dan
  builder netlist.
- [x] C1-22 Tambah smoke test Python dan validasi jumlah pin registry.

## 4a. Status Katalog dan Audit

- [x] C1-13 Hasilkan 500 entri katalog deterministik.
- [x] C1-14 Uji jumlah 500, ID unik, slug ASCII, dan family count.
- [x] C1-15 Uji validasi setiap entri dan smoke response finite.
- [x] C1-16 Petakan entry analog ke `DeviceKind` yang sudah didukung.
- [x] C1-19 Buat `ComponentRegistry` dan `ComponentRequest`.
- [x] C1-20 Uji emulasi voltage divider dan saklar ideal melalui registry.
- [ ] C1-17 Implementasikan solver penuh untuk keluarga behavioral.
- [ ] C1-18 Tambah reference vector dan cross-check ngspice per model.

Status per 2026-11-16. Angka `mapped` diambil dari field `device` di
katalog. Angka `verified` diambil dari syarat batch di bagian 3.

Legend:

- `cataloged`: entri ada dan lolos validasi metadata.
- `mapped`: `device` terisi dan registry dapat membuat `DeviceInstance`.
- `tested`: model device punya test, tetapi belum tentu test per entry.
- `verified`: memenuhi seluruh Aturan Batch.

| Level | Jumlah | Arti |
|---|---:|---|
| cataloged | 500 | metadata katalog lengkap |
| mapped | 363 | dapat masuk netlist lewat registry |
| tested per entry | 0 | belum ada test per entry |
| verified | 0 | belum ada yang lulus semua syarat |

Peringatan: 363 `mapped` bukan 363 model yang benar. Audit menemukan
mapping generik, pin count salah, dan parameter yang tidak sesuai model.

### 4a.1 Matriks per Batch

| Batch | Entri | Mapped | Catatan |
|---|---:|---:|---|
| C2-01 | 12 | 12 | 1 ideal benar; 11 butuh model khusus; potensiometer dan digital pot butuh pin kontrol. |
| C2-02 | 13 | 13 | 1 ideal; 12 butuh ESR, ESL, C(V), atau kontrol. |
| C2-03 | 10 | 10 | Ideal dan konstruksi dasar layak; coupled, current transformer, flyback, choke butuh 4 pin dan mutual. |
| C2-04 | 6 | 6 | Model mutual inductance ada; verifikasi batch belum. |
| C2-05 | 5 | 5 | Model BVD ada; verifikasi batch belum. |
| C2-06 | 9 | 9 | 1 ideal; lossy butuh loss; attenuator, splitter, coupler bukan transmission line. |
| C2-07 | 12 | 11 | 4 DC/AC, pulse V/I, sine V/I, PWL, exponential, dan noise V/I; clock/pattern belum. |
| C2-08 | 8 | 4 | 4 controlled benar; 4 behavioral belum. |
| C2-09 | 5 | 2 | Battery layak ideal; solar cell salah; 3 entri digital tidak sesuai kategori. |
| C3-01 | 10 | 10 | 2 rectifier benar; 8 butuh model Schottky, Zener, avalanche, tunnel, PIN, varactor. |
| C3-02 | 8 | 8 | Semua diode generik; tidak ada model optik atau LED. |
| C3-03 | 8 | 8 | 6 optocoupler memakai model optocoupler; 2 photodiode tanpa photocurrent. |
| C3-04 | 10 | 10 | 6 TVS memakai diode generik; 4 bridge/array memakai model bridge atau dual diode; verifikasi batch belum. |
| C3-05 | 4 | 4 | Data `is` salah, default 0.7 A dengan unit V. |
| C3-06 | 12 | 12 | Ebers-Moll ada; parameter masih default kelas. |
| C3-07 | 8 | 8 | Semua 3 pin; dual_pnp menjadi NPN; array butuh topologi sendiri. |
| C3-08 | 8 | 8 | 6 single layak; 2 dual salah pin. |
| C3-09 | 15 | 15 | 9 single layak; 2 dual dan 2 four-terminal salah pin; 2 depletion salah vto. |
| C3-10 | 8 | 8 | IGBT memakai model IGBT; GaN dan SiC memakai JFET atau MOSFET generik; verifikasi batch belum. |
| C3-11 | 4 | 4 | MESFET dan HEMT memakai JFET; UJT dan PUT memakai model unijunction; verifikasi batch belum. |
| C4-01 | 8 | 8 | Model thyristor behavioral; PUT thyristor memakai model PUT; verifikasi batch belum. |
| C4-02 | 6 | 6 | Fuse atau varistor; TVS memakai clamp varistor; verifikasi batch belum. |
| C4-03 | 4 | 4 | Circuit breaker memakai fuse; spark gap, crowbar, surge arrester memakai varistor. |
| C4-04 | 8 | 8 | 5 SPST-like layak; SPDT, DPST, DPDT 2 pin dan tidak bisa changeover. |
| C4-05 | 7 | 7 | Relay 5 pin ada; latching dan time delay belum; SSR memakai switch generik. |
| C4-06 | 5 | 5 | 2 switch layak; 4:1, 8:1, demux 4 pin salah. |
| C4-07 | 5 | 5 | Semua resistor 2 pin; header, terminal, IC socket butuh pin lebih; jumper tanpa state. |
| C5-01 | 15 | 15 | Hanya unit 5 pin yang layak; dual, quad, fully differential, instrumentation pin salah; tanpa noise, slew, bandwidth, offset. |
| C5-02 | 6 | 6 | Comparator generik; hysteresis, open-collector, delay belum. |
| C5-03 | 5 | 5 | Mapped ke opamp; topologi instrumentation tidak ada. |
| C5-04 | 5 | 5 | OTA dan Norton salah model; audio, video, buffer generik. |
| C5-05 | 5 | 4 | 555, monostable, astable, dan VCO solver-backed; PLL belum. |
| C5-06 | 4 | 4 | Voltage source 2 pin; current reference salah; topology shunt dan series belum. |
| C6-01 | 12 | 9 | 8 gate benar; Schmitt tanpa hysteresis; 3 special belum. |
| C6-02 | 10 | 0 | Belum ada model flip-flop dan latch. |
| C6-03 | 10 | 0 | Belum ada model counter dan shift register. |
| C6-04 | 8 | 0 | Belum ada model mux, decoder, encoder. |
| C6-05 | 8 | 0 | Belum ada model adder, comparator, buffer. |
| C6-06 | 6 | 0 | Belum ada model level shifter dan interface. |
| C6-07 | 6 | 0 | Belum ada model programmable logic. |
| C6-08 | 15 | 0 | Belum ada model memory. |
| C6-09 | 11 | 0 | Belum ada model MCU. |
| C6-10 | 5 | 0 | Belum ada model ADC, DAC, PWM. |
| C6-11 | 6 | 0 | Belum ada model serial interface. |
| C6-12 | 3 | 0 | Belum ada model timer, RTC, watchdog. |
| C7-01 | 6 | 6 | Regulator 3 pin generik; negative, current, shunt, adjustable belum. |
| C7-02 | 8 | 8 | Delapan topologi memakai converter teregulasi dengan efisiensi; verifikasi batch belum. |
| C7-03 | 5 | 5 | Charge pump dan charger memakai model tiga pin dengan efisiensi; verifikasi batch belum. |
| C7-04 | 5 | 5 | Voltage/current reference dan tiga supervisor memakai model tiga pin; verifikasi batch belum. |
| C7-05 | 3 | 3 | Ideal switch; ideal diode dan ORing logic belum. |
| C7-06 | 3 | 3 | H-bridge, half-bridge, dan stepper driver memakai switch satu kanal; verifikasi batch belum. |
| C7-07 | 6 | 6 | Model NTC, PTC, RTD, thermocouple, semikonduktor, thermal switch; verifikasi batch belum. |
| C7-08 | 5 | 5 | Photoresistor, photodiode, phototransistor/ambient/color memakai model cahaya; verifikasi batch belum. |
| C7-09 | 5 | 5 | Hall switch, Hall linear, reed switch, magnetoresistive, fluxgate; verifikasi batch belum. |
| C7-10 | 5 | 5 | Accelerometer, gyroscope, IMU, tilt, vibration memakai model satu sumbu; verifikasi batch belum. |
| C7-11 | 4 | 4 | Strain gauge, load cell, pressure, dan force memakai model bridge atau linier; verifikasi batch belum. |
| C7-12 | 4 | 4 | Humidity, gas, pH, dan electrochemical memakai model linier atau chemiresistor; verifikasi batch belum. |
| C7-13 | 3 | 3 | Current sense resistor, Hall current, dan voltage divider; verifikasi batch belum. |
| C7-14 | 3 | 3 | Inductive, capacitive touch, dan ultrasonic memakai model switch atau linier; verifikasi batch belum. |
| C7-15 | 4 | 4 | LED dan RGB memakai junction eksponensial; lampu dan neon memakai resistor atau DIAC; verifikasi batch belum. |
| C7-16 | 4 | 4 | Buzzer/speaker memakai resistor dan piezo memakai kapasitor; verifikasi batch belum. |
| C7-17 | 5 | 5 | Motor memakai resistance dan back EMF; solenoid memakai resistor; verifikasi batch belum. |
| C7-18 | 4 | 4 | Display memakai beban supply dan input leakage; verifikasi batch belum. |
| C7-19 | 3 | 3 | Heater memakai resistor; Peltier/TEG memakai resistance dan Seebeck; verifikasi batch belum. |
| C8-01 | 4 | 4 | RF amplifier, LNA, power amplifier, dan VGA memakai VCVS tiga pin; verifikasi batch belum. |
| C8-02 | 4 | 0 | Belum ada model RF mixer dan modulator. |
| C8-03 | 4 | 0 | Belum ada model RF filter. |
| C8-04 | 3 | 0 | Belum ada model antenna. |
| C8-05 | 3 | 0 | Belum ada model coupler, splitter, isolator. |
| C8-06 | 5 | 5 | Integrator, differentiator, low-pass, high-pass, dan state-variable sudah solver-backed. |
| C8-07 | 5 | 5 | Multiplier, divider, log, antilog, dan RMS sudah solver-backed. |
| C8-08 | 5 | 5 | Comparator, Schmitt, peak, track-hold, dan rectifier sudah solver-backed. |
| C8-09 | 5 | 0 | Belum ada model VCO, PLL, modulator. |
| C8-10 | 5 | 0 | Belum ada model sample-hold, ADC, DAC behavioral. |
| C9-01 | 3 | 0 | Belum ada model board. |
| C9-02 | 3 | 0 | Belum ada model breadboard dan jumper. |
| C9-03 | 3 | 0 | Belum ada model power module dan USB. |
| C9-04 | 3 | 0 | Belum ada model display dan sensor module. |
| C9-05 | 2 | 0 | Belum ada model enclosure dan mechanical. |

### 4a.2 Temuan Audit

1. Parameter diode sudah dikoreksi.
   - Sebelumnya 10 entri memakai `is` dengan unit V: 6 TVS default 6,
     4 diode lain default 0.7.
   - Sekarang semua `is` memakai unit A dan default 1e-12.
   - Breakdown voltage TVS belum menjadi parameter terpisah.
2. Waveform source sudah dikoreksi.
   - Sebelumnya 8 entri sine, pulse, PWL, exponential, dan noise memakai
     `VoltageSource` atau `CurrentSource` DC.
   - Sekarang pulse voltage source memakai `PulseVoltageSource` dan
     2 sine source memakai model sine; 5 waveform lain ditandai
     unsupported sampai model tersedia.
3. Topologi multi-unit dan changeover salah.
   - 8 BJT array memakai 3 pin; dual_pnp dan npn_pnp_pair memakai NPN.
   - 2 dual JFET dan 4 MOSFET ganda atau four-terminal memakai 3 pin.
   - 5 opamp multi-unit memakai 5 pin.
   - 3 mechanical switch changeover memakai 2 pin.
   - 3 analog mux dan demux memakai 4 pin.
4. Model pengganti tidak sesuai entry.
   - 11 resistor non-ideal, 12 kapasitor non-ideal, dan 9 induktor
     non-ideal masih memakai model ideal.
   - 8 LED, 2 photodiode, 6 TVS, dan 4 diode lain belum punya model
     optik, breakdown, atau photocurrent.
   - 4 voltage reference dan 6 regulator masih memakai source atau
     regulator generik.
   - 25 opamp dan OTA memakai satu macromodel opamp; 6 comparator
     memakai satu macromodel comparator.
   - 5 connector masih memakai resistor 2 pin.
5. Validasi katalog hanya memeriksa `pins > 0`.
   - Tidak ada aturan pin per model class.
   - Tidak ada min, max, status, references, atau test per entry.
6. Syarat batch belum terpenuhi.
   - 0 dari 500 punya tiga unit test per entry.
   - 0 dari 500 punya conservation test.
   - 0 dari 500 punya reference vector.
   - 0 dari 500 berstatus `verified`.
7. Infrastruktur verifikasi belum ada.
   - C1-06 sampai C1-10, C1-17, dan C1-18 masih terbuka.
8. Dokumentasi belum selaras.
   - `docs/11-netlist.md` masih menyebut device lama dan belum stamping.
   - `docs/12-device-models.md` belum punya status per entry.
   - Belum ada laporan coverage per kategori dan status.

### 4a.3 Yang Sudah Layak

- 500 entri katalog deterministik, ID unik, dan validasi metadata.
- Perintah `emucad catalog --json` dan binding Python untuk katalog.
- 25 device solver: resistor, capacitor, inductor, voltage source,
  current source, pulse source, VCVS, VCCS, CCCS, CCVS, switch, ideal
  switch, relay, transmission line, diode, BJT, JFET, MOSFET, opamp,
  comparator, regulator, dan logic gate.
- Test circuit di `sim-analysis`: voltage divider, RC dan RLC, diode
  clamp, BJT, JFET, MOSFET, transmission line, switch, pulse source,
  noise, distorsi, sensitivitas, corner, dan determinisme.
- 1054 test workspace lulus pada audit ini.

### 4a.4 Arti untuk Pengerjaan

- `mapped` hanya berarti entry dapat masuk netlist, bukan model benar.
- Batch baru boleh dicentang `verified` setelah C1-06 sampai C1-10
  tersedia dan entry punya reference vector.
- Perbaikan data dan mapping di 4b harus dikerjakan sebelum menambah
  model baru, supaya model baru tidak mewarisi data yang salah.

## 4b. Perbaikan Data dan Mapping

- [x] C1-23 Perbaiki parameter diode TVS dan diode lain.
  - `is` sekarang memakai unit A dan default 1e-12.
  - Tegangan breakdown belum menjadi parameter terpisah; menunggu C1-30.
- [x] C1-24 Perbaiki mapping sumber waveform.
  - Pulse voltage source memakai `PulseVoltageSource`.
  - Sine, PWL, exponential, noise, dan pulse current ditandai
    unsupported sampai model tersedia.
- [ ] C1-25 Perbaiki BJT array dan pasangan.
  - Pin count, polaritas, dan topologi harus sesuai entry.
- [ ] C1-26 Perbaiki pin MOSFET dan JFET ganda serta four-terminal.
- [ ] C1-27 Pisahkan opamp multi-unit atau tandai unsupported.
- [ ] C1-28 Perbaiki switch changeover dan analog multiplexer.
- [ ] C1-29 Tambahkan validasi pin count per model class.
- [~] C1-30 Tambahkan status, min, max, dan references ke schema.
  - `ParameterSpec` dengan min, max, dan kind sudah ada.
  - Status per entry dan references belum.
- [ ] C1-31 Audit semua parameter katalog dan daftar output yang
  dipengaruhinya.
- [~] C1-32 Buat parameter effect test untuk setiap parameter yang
  dipakai model.
  - Resistor dan diode `n` punya effect test DC.
  - Kapasitansi, induktansi, diode `cj0`, pulse width, dan rise punya
    effect test transient.
  - Parameter AC dan model lain belum.
- [x] C1-33 Sembunyikan atau hapus parameter yang belum berfungsi
  dari inspector.
  - Inspector hanya menampilkan parameter dari schema device.

## 5. Passives dan Sources

Target: 80 entri.

- [ ] C2-01 Resistor 12 entri: ideal, tempco, variable, digital pot,
  NTC, PTC, photoresistor, varistor, strain gauge, memristor, fuse,
  ferrite bead.
- [ ] C2-02 Kapasitor 13 entri: ideal, C0G, X7R, electrolytic, tantalum,
  film, variable, trimmer, supercap, feedthrough, ESR, ESL, voltage
  dependent.
- [ ] C2-03 Induktor 10 entri: ideal, lossy, saturable, coupled, air core,
  ferrite core, toroid, current transformer, flyback, common mode choke.
- [~] C2-04 Transformer 6 entri: ideal, center tap, autotransformer,
  isolation, pulse, current sense.
  - Model mutual inductance ada; reference vector dan verified belum.
- [~] C2-05 Crystal dan resonator 5 entri: quartz, ceramic, tuning fork,
  TCXO, VCXO.
  - Model BVD ada; reference vector dan verified belum.
- [ ] C2-06 Transmission line dan RF passive 9 entri: ideal line, lossy
  line, coax, microstrip, stripline, stub, attenuator, splitter,
  directional coupler.
- [~] C2-07 Independent source 12 entri: DC V, DC I, AC V, AC I, sine,
  pulse, PWL, exponential, noise V, noise I, clock, pattern.
  - Sine, pulse current, PWL, exponential, dan noise sudah solver-backed.
  - Clock dan pattern belum.
  - Reference vector dan verified belum.
- [ ] C2-08 Controlled dan behavioral source 8 entri: E, F, G, H, B
  voltage, B current, polynomial, table.
- [ ] C2-09 Battery dan energy source 5 entri: battery, fuel cell, solar
  cell, current-limited source, ideal energy port.

## 6. Semikonduktor

Target: 95 entri.

- [ ] C3-01 Rectifier, Schottky, Zener 10 entri.
- [ ] C3-02 LED, laser, IR 8 entri.
- [~] C3-03 Photodiode, phototransistor, optocoupler 8 entri.
  - 6 optocoupler sudah solver-backed.
  - 2 photodiode belum punya photocurrent.
  - Reference vector dan verified belum.
- [~] C3-04 Varactor, PIN, TVS, ESD, bridge, array 10 entri.
  - TVS, ESD, varactor, dan PIN memakai diode generik.
  - Bridge dan array memakai model bridge atau dual diode.
  - Reference vector dan verified belum.
- [ ] C3-05 Tunnel, avalanche, diac, lainnya 4 entri.
- [ ] C3-06 BJT NPN dan PNP 12 entri.
- [ ] C3-07 Darlington, array, digital transistor 8 entri.
- [ ] C3-08 JFET N dan P 8 entri.
- [ ] C3-09 MOSFET N dan P 15 entri: enhancement, depletion, dual gate,
  four terminal, body diode.
- [~] C3-10 Power MOSFET, IGBT, GaN, SiC 8 entri.
  - IGBT, GaN, dan SiC sudah solver-backed lewat model IGBT,
    JFET, atau MOSFET generik.
  - Reference vector dan verified belum.
- [~] C3-11 UJT, PUT, MESFET, HEMT 4 entri.
  - MESFET dan HEMT memakai JFET; UJT dan PUT memakai model
    unijunction.
  - Reference vector dan verified belum.

## 7. Proteksi, Switch, Relay

Target: 43 entri.

- [~] C4-01 SCR, TRIAC, DIAC, GTO 8 entri.
  - SCR/LASCR, TRIAC, DIAC, GTO/MCT, dan SCS sudah solver-backed.
  - PUT thyristor memakai model PUT.
  - Reference vector dan verified belum.
- [~] C4-02 Fuse, PTC, TVS, varistor 6 entri.
  - Fuse dan varistor sudah solver-backed.
  - Reference vector dan verified belum.
- [~] C4-03 Spark gap, crowbar, breaker 4 entri.
  - Breaker memakai fuse; spark gap, crowbar, surge arrester memakai
    varistor.
  - Reference vector dan verified belum.
- [~] C4-04 Mechanical switch 8 entri: SPST, SPDT, DPDT, push, toggle,
  DIP, reed, limit.
- [~] C4-05 Relay dan solid-state relay 7 entri.
- [~] C4-06 Analog switch dan multiplexer 5 entri.
- [~] C4-07 Connector, jumper, test point 5 entri.

Catatan: C4-04 sampai C4-07 sudah punya entri katalog dan registry,
tetapi belum memenuhi Aturan Batch. Model masih generik dan beberapa
pin count salah, sehingga statusnya sebagian, bukan selesai.

## 8. Op-Amp dan Analog IC

Target: 40 entri.

- [ ] C5-01 Op-amp 15 entri: ideal, single, dual, rail-to-rail, precision,
  low-noise, high-speed, power, fully differential.
- [ ] C5-02 Comparator 6 entri.
- [ ] C5-03 Instrumentation dan isolation amplifier 5 entri.
- [ ] C5-04 OTA, Norton, audio, video amplifier 5 entri.
- [~] C5-05 Timer, oscillator, PLL 5 entri.
  - 555, monostable, astable, dan VCO memakai model timer 8 pin.
  - PLL belum didukung.
  - Reference vector dan verified belum.
- [ ] C5-06 Voltage reference 4 entri.

## 9. Logic, Memory, MCU

Target: 100 entri.

- [ ] C6-01 Gate 12 entri: AND, OR, NOT, NAND, NOR, XOR, XNOR, buffer,
  Schmitt, open drain, tri-state, transmission gate.
- [ ] C6-02 Flip-flop dan latch 10 entri.
- [ ] C6-03 Counter dan shift register 10 entri.
- [ ] C6-04 Mux, decoder, encoder 8 entri.
- [ ] C6-05 Adder, comparator, buffer 8 entri.
- [ ] C6-06 Level shifter dan interface 6 entri.
- [ ] C6-07 Programmable logic dan state machine 6 entri.
- [ ] C6-08 Memory 15 entri: SRAM, DRAM, ROM, EEPROM, Flash, FRAM, MRAM,
  module, controller.
- [ ] C6-09 MCU core 11 entri: AVR, PIC, 8051, ARM, RISC-V, ESP, STM32.
- [ ] C6-10 ADC, DAC, PWM peripheral 5 entri.
- [ ] C6-11 UART, SPI, I2C, CAN, USB, Ethernet 6 entri.
- [ ] C6-12 Timer, RTC, watchdog 3 entri.

## 10. Power, Sensor, Actuator

Target: 85 entri.

- [ ] C7-01 Linear regulator dan LDO 6 entri.
- [~] C7-02 Buck, boost, buck-boost 8 entri.
  - Semua topologi memakai converter teregulasi dengan efisiensi.
  - Duty cycle, ripple, dan isolasi belum dimodelkan.
  - Reference vector dan verified belum.
- [~] C7-03 Charge pump dan charger 5 entri.
  - Charge pump dan charger memakai model tiga pin.
  - Topologi switching dan proteksi baterai belum dimodelkan.
  - Reference vector dan verified belum.
- [~] C7-04 Reference dan supervisor 5 entri.
  - Voltage dan current reference memakai sumber ideal dengan quiescent current.
  - Tiga supervisor memakai threshold dan hysteresis.
  - Delay dan proteksi belum dimodelkan.
  - Reference vector dan verified belum.
- [ ] C7-05 Ideal diode, ORing, load switch 3 entri.
- [~] C7-06 Motor driver 3 entri.
  - Tiga entry memakai switch high-side satu kanal.
  - Arah, PWM, dead time, dan proteksi belum dimodelkan.
  - Reference vector dan verified belum.
- [~] C7-07 Temperature sensor 6 entri.
  - NTC, PTC, RTD, thermocouple, sensor semikonduktor, dan thermal
    switch sudah solver-backed.
  - Reference vector dan verified belum.
- [~] C7-08 Light dan optical sensor 5 entri.
  - Photoresistor, photodiode, dan linear light sensor sudah solver-backed.
  - Color sensor masih satu kanal, belum RGB.
  - Reference vector dan verified belum.
- [~] C7-09 Magnetic, Hall, reed 5 entri.
  - Hall switch, Hall linear, reed switch, magnetoresistive, dan
    fluxgate sudah solver-backed.
  - Reference vector dan verified belum.
- [~] C7-10 Motion, accelerometer, gyro, IMU 5 entri.
  - Accelerometer, gyroscope, IMU, tilt, dan vibration sudah
    solver-backed.
  - Semua model hanya satu sumbu.
  - Reference vector dan verified belum.
- [~] C7-11 Pressure, force, strain 4 entri.
  - Strain gauge dan load cell memakai persamaan bridge.
  - Pressure dan force memakai model linier.
  - Reference vector dan verified belum.
- [~] C7-12 Humidity, gas, chemical 4 entri.
  - Humidity dan electrochemical memakai model linier.
  - Gas memakai chemiresistor dan pH memakai kemiringan Nernst.
  - Reference vector dan verified belum.
- [~] C7-13 Current, voltage, power sensor 3 entri.
  - Current sense resistor memakai shunt dan gain.
  - Hall current dan voltage divider memakai model linier.
  - Reference vector dan verified belum.
- [~] C7-14 Proximity, touch, ultrasonic 3 entri.
  - Inductive dan capacitive touch memakai threshold switch.
  - Ultrasonic memakai output linier terhadap jarak.
  - Reference vector dan verified belum.
- [~] C7-15 LED dan lamp 4 entri.
  - LED dan RGB memakai junction eksponensial.
  - Lamp memakai resistor dan neon memakai model DIAC.
  - Output cahaya belum dirender.
  - Reference vector dan verified belum.
- [~] C7-16 Buzzer, speaker, piezo 4 entri.
  - Buzzer dan speaker memakai resistor.
  - Piezo memakai kapasitor.
  - Output akustik belum dirender.
  - Reference vector dan verified belum.
- [~] C7-17 Motor, servo, stepper, solenoid 5 entri.
  - Motor memakai resistance dan back EMF satu winding.
  - Solenoid memakai resistor tanpa induktansi.
  - Torsi dan kontrol posisi belum dimodelkan.
  - Reference vector dan verified belum.
- [~] C7-18 LCD, OLED, e-ink 4 entri.
  - Display memakai beban supply empat pin.
  - Protokol data dan piksel belum dimodelkan.
  - Reference vector dan verified belum.
- [~] C7-19 Heater dan Peltier 3 entri.
  - Heater memakai resistor.
  - Peltier dan TEG memakai resistance dan Seebeck.
  - Aliran panas belum dimodelkan.
  - Reference vector dan verified belum.

## 11. RF dan Behavioral

Target: 43 entri.

- [~] C8-01 RF amplifier 4 entri.
  - Empat entry memakai model VCVS tiga pin.
  - Belum ada S-parameter, matching, noise, atau bandwidth.
  - Reference vector dan verified belum.
- [ ] C8-02 RF mixer dan modulator 4 entri.
- [ ] C8-03 RF filter 4 entri.
- [ ] C8-04 Antenna 3 entri.
- [ ] C8-05 Coupler, splitter, isolator 3 entri.
- [~] C8-06 Integrator, differentiator, filter 5 entri.
  - Lima filter memakai model 4 pin dengan state.
  - Cutoff diturunkan dari time constant.
  - Reference vector dan verified belum.
- [~] C8-07 Multiplier, divider, log, RMS 5 entri.
  - Lima math block memakai model 4 pin.
  - Proteksi numerik ada, proteksi rangkaian belum.
  - Reference vector dan verified belum.
- [~] C8-08 Comparator, Schmitt, peak detector 5 entri.
  - Lima detector memakai model 4 pin.
  - Delay, droop, dan aperture time belum dimodelkan.
  - Reference vector dan verified belum.
- [ ] C8-09 VCO, PLL, modulator 5 entri.
- [ ] C8-10 Sample-hold, ADC, DAC behavioral 5 entri.

## 12. Board dan Module

Target: 14 entri.

- [ ] C9-01 Arduino dan board 3 entri.
- [ ] C9-02 Breadboard dan jumper 3 entri.
- [ ] C9-03 Power module dan USB 3 entri.
- [ ] C9-04 Display dan sensor module 3 entri.
- [ ] C9-05 Enclosure dan mechanical 2 entri.

## 13. Part-Specific dan Validasi

- [ ] C10-01 Pilih 50 part number prioritas dari kategori populer.
- [ ] C10-02 Ekstrak parameter dari datasheet resmi.
- [ ] C10-03 Bandingkan model dengan ngspice atau referensi eksternal.
- [ ] C10-04 Uji suhu, corner, dan parameter toleransi.
- [ ] C10-05 Buat regression golden untuk model kritis.
- [ ] C10-06 Audit lisensi semua model dan datasheet.

## 14. Python Builder

Keputusan: binding langsung memakai `pyo3` di crate `sim-python`, bukan
subprocess. Python memakai dict untuk request dan response.

- [x] C11-01 Kunci schema request dan response.
- [x] C11-02 Buat loader katalog dari Rust.
- [ ] C11-03 Buat DSL Python tipis untuk topologi dan parameter.
- [x] C11-04 Buat exporter netlist JSON. (response dict kompatibel `json.dumps`)
- [x] C11-05 Buat binding PyO3 ke Rust. (menggantikan runner subprocess)
- [x] C11-06 Buat mapping error Rust ke exception Python.
- [x] C11-07 Buat contoh divider, RC, diode, dan logic.
- [x] C11-08 Buat contract test Python ke Rust.

## 15. Release dan Dokumentasi

- [ ] C12-01 Perbarui `docs/12-device-models.md`.
- [ ] C12-02 Perbarui `docs/11-netlist.md`.
- [x] C12-03 Perbarui `README.md` dan `docs/README.md`.
- [ ] C12-04 Tambah laporan coverage 500 komponen.
- [x] C12-05 Jalankan fmt, clippy, test, ASCII, dan package.
- [ ] C12-06 Buat tag katalog komponen.

## 16. Definition of Done

- [ ] 500 entri katalog ada dan dapat dimuat.
- [ ] Semua entri punya prinsip kerja, parameter, dan referensi.
- [ ] Semua entri punya minimal tiga unit test.
- [ ] Semua entri verified lulus reference vector.
- [ ] Setiap parameter numerik punya effect test dan mengubah hasil.
- [ ] Tidak ada parameter editable yang tidak dipakai model.
- [ ] Tidak ada dependensi eksternal baru di crate inti.
- [ ] Tidak ada `unsafe`.
- [ ] Tidak ada wall-clock di simulation.
- [ ] Semua fungsi maksimal 20 baris.
- [ ] ASCII-only.
- [ ] Python builder dapat memuat katalog dan menjalankan netlist.
- [ ] fmt, clippy, test, dan package lulus.

## 17. Pertanyaan Terbuka

- Apakah 500 entri harus 500 model generik, atau 500 part number?
- Berapa rasio part-specific yang diinginkan?
- Toleransi 1 sampai 5 persen dapat diterima untuk model nonlinear?
- Apakah model GPL boleh dipakai, atau hanya datasheet dan model MIT/BSD?
- Apakah Python builder wajib interaktif sejak fase pertama?
- Siapa yang melakukan review elektrik untuk model part-specific?
- Apakah RF memerlukan S-parameter sejak awal atau model lumped cukup?
