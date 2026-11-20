# Catalog Coverage

Laporan ini menghitung cakupan 500 entri katalog secara deterministik dari
kode dan data katalog. Fokusnya adalah angka yang dapat dibuktikan, bukan
klaim bahwa model sudah benar.

## 1. Ringkasan

- 500 entri katalog, 16 family, dan 60 model class.
- 379 entri solver-backed: field `device` berisi `DeviceKind`.
- 379 entri mapped: registry dapat membuat `DeviceInstance` dan model solver.
- 78 entri no-device memiliki model digital di `cells::catalog_digital`.
- 43 entri tidak memiliki model katalog maupun model digital.
- Fidelity entri: 44 ideal, 154 compact, 181 behavioral, 0 calibrated,
  dan 121 unsupported.
- Tidak ada field `status` per entri di kode katalog. Status 500 entri
  tidak diketahui dari kode.

Angka penting yang mudah salah dibaca:

- "Behavioral" di ringkasan CLI berarti 121 entri tanpa `device`, bukan
  181 entri dengan fidelity `behavioral`.
- `unsupported` adalah label fidelity karena tidak ada `device`. Label ini
  tidak berarti 121 entri tidak bisa disimulasikan sama sekali, karena 78
  di antaranya punya model digital.
- `mapped` pada tabel batch `TODO-COMPONENTS.md` memakai arti yang lebih
  luas daripada field `device`. Jumlah kolomnya 449, bukan 379.

## 2. Sumber dan Metode

Sumber angka:

- `crates/device/src/catalog_data/` untuk `id`, `ModelClass`, dan `device`.
- `crates/device/src/catalog/model/class.rs` untuk pemetaan class ke family.
- `crates/device/src/catalog/model/binding.rs` untuk pemetaan fidelity.
- `crates/cells/src/catalog_digital.rs` untuk model digital berbasis nama.
- `docs/data/class-fidelity.csv` sebagai pembanding class-level.
- `docs/data/behavioral-classification.csv` sebagai klasifikasi rencana
  untuk entri tanpa `device`.

Regenerasi angka:

```bash
python3 scripts/catalog_coverage.py
```

Script hanya membaca file, memakai Python standard library, dan mencetak
tabel Markdown. Script memvalidasi 500 entri, ID unik, family yang cocok
dengan ID, ID digital yang ada di katalog, dan tidak ada entri yang punya
`device` sekaligus model digital.

Verifikasi silang:

```bash
cargo run -p cli -- catalog
cargo run -p cli -- catalog --json
```

Perintah pertama menghasilkan 500 entri dan 379 solver-backed. Perintah
kedua menghasilkan family dan model class yang sama dengan script.

## 3. Definisi Metrik

| Metrik | Definisi | Jumlah |
|---|---|---:|
| Solver-backed | `ComponentSpec::device` bernilai `Some` | 379 |
| Mapped | Solver-backed dan registry dapat membuat model | 379 |
| Digital-backed | `cells::catalog_digital::digital_entry` mengembalikan entri | 78 |
| Behavioral (CLI) | Tidak punya `device`, disebut "behavioral" oleh CLI | 121 |
| Behavioral (fidelity) | Solver-backed dengan class yang dipetakan ke `Fidelity::Behavioral` | 181 |
| Unsupported (fidelity) | `SimulationBinding::fidelity` mengembalikan `unsupported` | 121 |
| Lacking a model | Tidak punya `device` dan tidak punya model digital | 43 |

`Mapped` sama dengan solver-backed karena test
`registry_instantiates_and_models_solver_entries` memastikan seluruh entri
solver-backed dapat diinstansiasi dan dibuatkan model. Tidak ada angka
mapped terpisah di kode.

## 4. Fidelity

| Fidelity | Jumlah | Arti di kode |
|---|---:|---|
| `ideal` | 44 | Class passif dan source ideal |
| `compact` | 154 | Class semikonduktor, protection, switch, passif terkopel |
| `behavioral` | 181 | Solver-backed dengan class analog, digital, power, sensor, dan sejenisnya |
| `calibrated` | 0 | Tidak ada entri yang dipetakan ke label ini |
| `unsupported` | 121 | Tidak punya `device` |
| **Total** | **500** | |

Fidelity adalah label turunan dari `SimulationBinding::fidelity()`, bukan
field yang disimpan di `ComponentSpec` atau `CatalogRecord`.

## 5. Family

| Family | Entries | Solver | Digital | No model | ideal | compact | behavioral | unsupported |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| passive | 55 | 55 | 0 | 0 | 35 | 20 | 0 | 0 |
| source | 25 | 20 | 3 | 2 | 9 | 2 | 9 | 5 |
| diode | 40 | 40 | 0 | 0 | 0 | 34 | 6 | 0 |
| transistor | 55 | 55 | 0 | 0 | 0 | 55 | 0 | 0 |
| protection | 18 | 18 | 0 | 0 | 0 | 18 | 0 | 0 |
| switch | 25 | 25 | 0 | 0 | 0 | 25 | 0 | 0 |
| analog | 40 | 40 | 0 | 0 | 0 | 0 | 40 | 0 |
| logic | 60 | 9 | 51 | 0 | 0 | 0 | 9 | 51 |
| memory | 15 | 0 | 15 | 0 | 0 | 0 | 0 | 15 |
| mcu | 25 | 4 | 9 | 12 | 0 | 0 | 4 | 21 |
| power | 30 | 30 | 0 | 0 | 0 | 0 | 30 | 0 |
| sensor | 35 | 35 | 0 | 0 | 0 | 0 | 35 | 0 |
| actuator | 20 | 20 | 0 | 0 | 0 | 0 | 20 | 0 |
| rf | 18 | 4 | 0 | 14 | 0 | 0 | 4 | 14 |
| behavioral | 25 | 24 | 0 | 1 | 0 | 0 | 24 | 1 |
| board | 14 | 0 | 0 | 14 | 0 | 0 | 0 | 14 |
| **Total** | **500** | **379** | **78** | **43** | **44** | **154** | **181** | **121** |

## 6. Model Class

| Model class | Family | Entries | Solver | Digital | No model | Solver fidelity |
|---|---|---:|---:|---:|---:|---|
| Actuator | actuator | 16 | 16 | 0 | 0 | behavioral |
| Adc | mcu | 5 | 4 | 0 | 1 | behavioral |
| Adder | logic | 8 | 0 | 8 | 0 | - |
| AnalogSwitch | switch | 5 | 5 | 0 | 0 | compact |
| Battery | source | 2 | 2 | 0 | 0 | compact |
| BehavioralBlock | behavioral | 25 | 24 | 0 | 1 | behavioral |
| BehavioralSource | source | 11 | 9 | 0 | 2 | behavioral |
| Bjt | transistor | 20 | 20 | 0 | 0 | compact |
| BoardModule | board | 14 | 0 | 0 | 14 | - |
| Capacitor | passive | 13 | 13 | 0 | 0 | ideal |
| Cccs | source | 1 | 1 | 0 | 0 | ideal |
| Ccvs | source | 1 | 1 | 0 | 0 | ideal |
| ChargePump | power | 5 | 5 | 0 | 0 | behavioral |
| Comparator | analog | 6 | 6 | 0 | 0 | behavioral |
| Connector | switch | 5 | 5 | 0 | 0 | compact |
| Counter | logic | 10 | 0 | 10 | 0 | - |
| Crystal | passive | 5 | 5 | 0 | 0 | compact |
| CurrentSource | source | 2 | 2 | 0 | 0 | ideal |
| DcDc | power | 8 | 8 | 0 | 0 | behavioral |
| DigitalSource | source | 3 | 0 | 3 | 0 | - |
| Diode | diode | 20 | 20 | 0 | 0 | compact |
| DiodeArray | diode | 4 | 4 | 0 | 0 | compact |
| Display | actuator | 4 | 4 | 0 | 0 | behavioral |
| FlipFlop | logic | 10 | 0 | 10 | 0 | - |
| Inductor | passive | 10 | 10 | 0 | 0 | ideal |
| Jfet | transistor | 8 | 8 | 0 | 0 | compact |
| Led | diode | 8 | 8 | 0 | 0 | compact |
| LevelShifter | logic | 6 | 0 | 6 | 0 | - |
| LogicGate | logic | 9 | 9 | 0 | 0 | behavioral |
| LogicGateSpecial | logic | 3 | 0 | 3 | 0 | - |
| Mcu | mcu | 11 | 0 | 0 | 11 | - |
| Memory | memory | 15 | 0 | 15 | 0 | - |
| Mosfet | transistor | 17 | 17 | 0 | 0 | compact |
| MotorDriver | power | 3 | 3 | 0 | 0 | behavioral |
| Multiplexer | logic | 8 | 0 | 8 | 0 | - |
| OpAmp | analog | 25 | 25 | 0 | 0 | behavioral |
| Optocoupler | diode | 6 | 6 | 0 | 0 | behavioral |
| OtherTransistor | transistor | 10 | 10 | 0 | 0 | compact |
| Photodiode | diode | 2 | 2 | 0 | 0 | compact |
| PowerSwitch | power | 3 | 3 | 0 | 0 | behavioral |
| ProgrammableLogic | logic | 6 | 0 | 6 | 0 | - |
| Protection | protection | 10 | 10 | 0 | 0 | compact |
| Regulator | power | 6 | 6 | 0 | 0 | behavioral |
| Relay | switch | 3 | 3 | 0 | 0 | compact |
| Resistor | passive | 12 | 12 | 0 | 0 | ideal |
| RfBlock | rf | 18 | 4 | 0 | 14 | behavioral |
| Sensor | sensor | 35 | 35 | 0 | 0 | behavioral |
| SerialInterface | mcu | 6 | 0 | 6 | 0 | - |
| SolidStateRelay | switch | 4 | 4 | 0 | 0 | compact |
| Supervisor | power | 5 | 5 | 0 | 0 | behavioral |
| Switch | switch | 8 | 8 | 0 | 0 | compact |
| Thyristor | protection | 8 | 8 | 0 | 0 | compact |
| Timer | analog | 5 | 5 | 0 | 0 | behavioral |
| TimerPeripheral | mcu | 3 | 0 | 3 | 0 | - |
| Transformer | passive | 6 | 6 | 0 | 0 | compact |
| TransmissionLine | passive | 9 | 9 | 0 | 0 | compact |
| Vccs | source | 1 | 1 | 0 | 0 | ideal |
| Vcvs | source | 1 | 1 | 0 | 0 | ideal |
| VoltageReference | analog | 4 | 4 | 0 | 0 | behavioral |
| VoltageSource | source | 3 | 3 | 0 | 0 | ideal |

## 7. Status Simulasi

Tidak ada field `status` di `ComponentSpec` atau `CatalogRecord`. Schema
contoh di `docs/60-component-catalog.md` memakai `status`, tetapi kode
tidak menyimpannya. Karena itu:

| Status per entri | Jumlah | Bukti |
|---|---:|---|
| `draft` | tidak diketahui | Tidak ada field |
| `implemented` | tidak diketahui | Tidak ada field |
| `tested` | tidak diketahui | Tidak ada field |
| `verified` | tidak diketahui | Tidak ada field |
| `deprecated` | tidak diketahui | Tidak ada field |
| **Total** | **500** | Semua status tidak diketahui |

Yang dapat diamati dari kode hanyalah kemampuan model, bukan status
verifikasi:

| Kemampuan | Jumlah |
|---|---:|
| Punya `device` katalog | 379 |
| Punya model digital | 78 |
| Punya salah satu model | 457 |
| Tidak punya model katalog atau digital | 43 |

Tidak ada entri yang punya `device` sekaligus model digital. Karena itu
jumlah 457 adalah 379 + 78 tanpa overlap.

Rincian 121 entri tanpa `device`:

| Family | No device | Digital | No model |
|---|---:|---:|---:|
| logic | 51 | 51 | 0 |
| memory | 15 | 15 | 0 |
| mcu | 21 | 9 | 12 |
| source | 5 | 3 | 2 |
| rf | 14 | 0 | 14 |
| behavioral | 1 | 0 | 1 |
| board | 14 | 0 | 14 |
| **Total** | **121** | **78** | **43** |

43 entri tanpa model terdiri dari:

- 14 `RfBlock`: mixer, modulator, filter, antenna, coupler, splitter,
  dan isolator.
- 14 `BoardModule`: board, breadboard, module, power module, enclosure.
- 11 `Mcu`: AVR, PIC, 8051, ARM, RISC-V, ESP, STM32, dan generic core.
- 1 `Adc`: `mcu.converter.r2r_dac`.
- 2 `BehavioralSource`: behavioral voltage source dan current source.
- 1 `BehavioralBlock`: `behavioral.converter.r2r_dac_behavioral`.

Klasifikasi rencana untuk 121 entri tanpa `device`, dari
`docs/data/behavioral-classification.csv`, bukan status runtime:

| Kategori | Jumlah |
|---|---:|
| `digital` | 63 |
| `structural` | 26 |
| `rf` | 14 |
| `unsupported` | 14 |
| `behavioral-electrical` | 3 |
| `mixed-signal` | 1 |
| `compact-analog` | 0 |
| **Total** | **121** |

## 8. Temuan dan Ketidaksesuaian

1. Status per entri tidak ada di kode.
   - `docs/60` mendefinisikan status `draft` sampai `deprecated`, tetapi
     `ComponentSpec` dan `CatalogRecord` tidak punya field tersebut.
   - Laporan ini tidak mengisi status yang tidak dapat dibuktikan.
2. Istilah "behavioral" dipakai untuk dua populasi berbeda.
   - CLI menyebut 121 entri tanpa `device` sebagai "behavioral".
   - Fidelity `behavioral` berlaku untuk 181 entri solver-backed.
   - Angka 121 dan 181 sama-sama benar untuk definisinya masing-masing,
     tetapi tidak boleh ditukar.
3. `unsupported` tidak sama dengan tidak punya model.
   - 121 entri mendapat fidelity `unsupported` karena tidak punya `device`.
   - 78 di antaranya punya model digital di `cells::catalog_digital`.
   - Hanya 43 entri yang tidak punya model katalog maupun digital.
4. `TODO-COMPONENTS.md` memakai dua arti "mapped".
   - Ringkasan menyebut 379 mapped dari field `device`.
   - Jumlah kolom Mapped pada matriks batch adalah 449.
   - Selisih 70 sebagian besar berasal dari matriks yang menghitung entri
     digital sebagai mapped walaupun field `device` kosong. Contoh:
     C6-02 sampai C6-08 berjumlah 63 mapped, tetapi solver-backed aktual
     untuk FlipFlop, Counter, Multiplexer, Adder, LevelShifter,
     ProgrammableLogic, dan Memory adalah 0.
5. `TODO-COMPONENTS.md` menyebut 59 entri dapat dijalankan di domain
   digital. Kode `catalog_digital.rs` memetakan 78 entri, dan `docs/62`
   juga menyebut 78 (63 logic/peripheral dan 15 memory gate-level).
6. `docs/63-fidelity.md` masih memuat catatan BehavioralSource yang saling
   bertentangan:
   - Baris 44-45 menyebut 2 source solver-backed dan 9 unsupported.
   - Baris 48-49 menyebut 7 solver-backed dan 4 unsupported.
   - Baris 52-53 menyebut 9 solver-backed dan 2 unsupported.
   - Katalog saat ini berisi 11 `BehavioralSource`: 9 solver-backed dan
     2 unsupported. Hanya catatan terakhir yang cocok.
7. `docs/63-fidelity.md` menyebut Timer hanya 4 solver-backed dan PLL
   masih unsupported. Katalog saat ini memetakan 5 dari 5 `Timer` ke
   `DeviceKind`, termasuk `TimerPll`, dan test model solver lulus.
8. `docs/data/class-fidelity.csv` memberi label fidelity per class, bukan
   per entri. Untuk class dengan dukungan sebagian, label class tidak
   berlaku untuk entri tanpa `device`.
   - CSV menghitung 103 unsupported, sedangkan label per entri menghitung
     121 unsupported.
   - Selisih 18 berasal dari Adc 1, BehavioralBlock 1, BehavioralSource 2,
     dan RfBlock 14.

## 9. Verifikasi

Perintah yang dijalankan:

```bash
python3 scripts/catalog_coverage.py
cargo test -p device catalog:: --lib
cargo test -p cells catalog_digital --lib
cargo test --workspace
cargo run -p cli -- catalog
cargo run -p cli -- catalog --json
python3 .opencode/skills/senior-writing/scripts/check_ascii.py docs/catalog-coverage.md
python3 .opencode/skills/senior-writing/scripts/check_ascii.py scripts/catalog_coverage.py
```

Hasil:

- Script mencetak 500 entri, 379 solver-backed, 78 digital-backed,
  43 tanpa model, dan total fidelity yang cocok dengan tabel di atas.
- `cargo test -p device catalog:: --lib` lulus 42 test.
- `cargo test -p cells catalog_digital --lib` lulus 25 test.
- `cargo test --workspace` lulus dengan 3 test ignored.
- `emucad catalog` mencetak `catalog: 500 entries, 379 solver-backed,
  121 behavioral`.
- `emucad catalog --json` menghasilkan 500 record dan 379
  `solver_backed: true`.
- Pemeriksaan ASCII lulus untuk kedua file.

## 10. Batasan

- Angka digital menunjukkan adanya mapping di `catalog_digital.rs`, bukan
  bukti bahwa setiap dari 78 entri sudah diuji perilakunya. Test
  `catalog_digital` saat ini berjumlah 25, bukan 78 test per entri.
- Fidelity adalah label turunan, bukan hasil pengukuran atau kalibrasi.
- Tidak ada reference vector per entri, sehingga tidak ada entri yang
  dapat disebut `verified` dari data katalog saja.
- `components` di luar katalog, misalnya model AVR di crate `avr`, tidak
  dihitung sebagai model katalog kecuali ada mapping eksplisit.
- Script mem-parse teks Rust yang dihasilkan. Jika format data berubah,
  script harus diperbarui. Validasi family dan ID digital membantu
  mendeteksi perubahan besar, tetapi bukan pengganti test Rust.

## 11. Langkah Berikutnya

1. Putuskan satu definisi resmi untuk `mapped` dan `behavioral`, lalu
   selaraskan `docs/60`, `docs/63`, CLI, dan `TODO-COMPONENTS.md`.
2. Tambahkan field status dan references ke schema jika status per entri
   memang dibutuhkan. Tanpa field itu, laporan hanya dapat menghitung
   kemampuan model.
3. Tambahkan test yang memastikan setiap ID di `catalog_digital.rs` dapat
   dibangun dan memiliki port yang diharapkan.
4. Tambahkan reference vector per entry sebelum memakai label `verified`.
