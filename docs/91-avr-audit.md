# 91 - Audit Struktur AVR

Tanggal: 2026-11-20
Status: audit dan cleanup awal; refactor structural belum dilakukan.

## 1. Kesimpulan

AVR saat ini adalah model hybrid, bukan implementasi full `cells`.
Sebagian modul sudah memakai `cells`, tetapi jalur eksekusi CPU masih
memakai array Rust dan fungsi byte fungsional. Ada dua implementasi
paralel:

1. Blok structural berbasis `cells` yang diuji terpisah.
2. Jalur eksekusi fungsional yang dipakai oleh `avr::Cpu`.

Akibatnya:

- Tidak ada dead code menurut compiler, karena banyak item diekspos
  sebagai API publik.
- Tetapi ada dead integration: register bank, instruction register,
  ALU, shift, dan multiply structural tidak dipakai oleh CPU.
- Klaim dokumentasi bahwa AVR memakai `cells` untuk register bank dan
  ALU saat ini terlalu kuat.
- Dependency `device` di `avr` tidak dipakai dan sudah dihapus dari
  `crates/avr/Cargo.toml`.

Untuk memenuhi aturan "AVR wajib memakai cells atau device yang sudah
ada", perlu keputusan apakah yang dimaksud adalah:

- Semua state CPU harus structural di `DigitalCircuit`, atau
- Semua primitive baru harus memakai `cells`/`device`, tetapi model
  behavioral dan jalur eksekusi fungsional masih boleh ada.

## 2. Kategori Modul

| Kategori | Modul | Status |
|---|---|---|
| Structural dan terhubung ke CPU | `gpio.rs` | Dipakai untuk PORT/DDR/PIN melalui `DigitalCircuit` |
| Structural tetapi hanya dipakai test | `register_bank.rs` | Membungkus `cells::register_file`, tidak dipakai `Cpu` |
| Structural tetapi hanya dipakai test | `instruction_register.rs` | Membungkus `cells::register`, tidak dipakai `Cpu` |
| Structural tetapi hanya dipakai test | `alu/*`, `multiply.rs`, `shift.rs` | Unit gate-level, tidak dipakai jalur eksekusi |
| Eksekusi fungsional | `cpu/*`, `decode/*`, `fetch.rs` | Array register, PC/SP integer, fungsi byte |
| Behavioral peripheral | `timer0/1/2`, `uart`, `adc`, `spi`, `twi`, `system` | State machine Rust, bukan `cells`/`device` |
| Electrical pin | `pin.rs` | Model analitik, bukan `device` |
| Legacy/generik | `cells::Cpu`, `cells::DigitalAlu` | Tidak dipakai `avr`; hanya dipakai internal `cells` dan test |

## 3. Temuan

### [High] Register bank structural tidak terhubung ke CPU

`crates/avr/src/register_bank.rs` membangun `cells::RegisterFile`
32 x 8. Namun `crates/avr/src/cpu/mod.rs` menyimpan:

- `registers: [u8; 32]`
- `write_register()` hanya menulis array tersebut.

Tidak ada `RegisterBank` di `Datapath`, dan `build_datapath()` hanya
membangun GPIO. Register bank structural hanya muncul di test-nya
sendiri.

Dampak: dua sumber kebenaran untuk register, dan CPU tidak benar-benar
menggunakan register bank structural.

Perbaikan: pilih salah satu.

- Hubungkan `RegisterBank` ke `DigitalCircuit` dan jadikan satu-satunya
  sumber state register.
- Atau pindahkan `RegisterBank` ke modul verifikasi structural dan
  hapus klaim bahwa CPU memakainya.

### [High] Instruction register structural tidak terhubung

`instruction_register.rs` membangun register 32 bit dari D flip-flop,
tetapi `Cpu` menyimpan `Instruction` hasil decode langsung dan tidak
membangun `InstructionRegister`.

Dampak: blok structural hanya menjadi artefak test.

### [High] ALU, shift, dan multiply structural tidak dipakai eksekusi

Jalur eksekusi memakai:

- `cells::add_with_carry` dan `cells::sub_with_borrow` di
  `cpu/execute.rs`.
- `logic_value`, `shift_value`, dan `multiply_value` di
  `cpu/helpers.rs`.

Fungsi tersebut adalah perhitungan byte fungsional, bukan simulasi
`DigitalCircuit`. Modul `avr/src/alu/*`, `multiply.rs`, dan `shift.rs`
berisi versi gate-level yang hanya dipakai test.

Dampak: logika AVR memiliki dua implementasi yang harus dijaga
konsisten, dan versi structural tidak menjadi bagian dari eksekusi.

### [High] PC, SP, SRAM, dan flash belum structural

`Cpu` memakai:

- `pc: u16`
- `sp: u16`
- `data: Vec<u8>` untuk SRAM
- `ProgramMemory` berbasis `Vec<u16>` untuk flash

`cells` sudah menyediakan `ProgramCounter`, `StackPointer`, `sram6t`,
dan macro memory. Semuanya belum diintegrasikan ke `Cpu`.

Dampak: state CPU utama tidak dapat diaudit sebagai netlist gate-level.

### [Medium] Peripheral utama masih behavioral

`Timer0/1/2`, `Uart`, `Adc`, `Spi`, `Twi`, dan `system` adalah model
behavioral Rust. Ini terdokumentasi di `docs/64-gate-level.md`, tetapi
belum memenuhi aturan bahwa AVR wajib memakai `cells`/`device`.

Dampak: timing peripheral tidak berasal dari netlist gate-level.

Perbaikan: klasifikasikan sebagai adapter behavioral dengan batas
fidelity eksplisit, atau buat jalur structural terpisah jika wajib.

### [Resolved] Dependency `device` tidak dipakai

`crates/avr/Cargo.toml` sebelumnya mencantumkan `device`, tetapi tidak
ada `use device` atau `device::` di `crates/avr/src`.

Status: dependency sudah dihapus. Jika nanti model electrical pin ingin
memakai `device`, dependency dapat ditambahkan kembali dengan alasan
yang jelas.

### [Medium] Register map dan ukuran memory masih hardcoded

Contoh literal yang tersebar:

- Alamat UART: `0xC0`, `0xC1`, `0xC6`.
- Alamat Timer0: `0x24`, `0x25`, `0x35`.
- Ukuran data: `0x1000`.
- Stack pointer awal: `0x08FF`.
- Mask alamat data: `0x0FFF`.

Sebagian adalah konstanta hardware yang sah, tetapi penyebarannya
menyulitkan audit dan rawan duplikasi.

Perbaikan: buat modul `avr::spec` atau `avr::registers` berisi konstanta
bernama untuk alamat, bit, ukuran SRAM, RAMEND, dan vektor interrupt.

### [Medium] `cells::Cpu` generik tidak dipakai AVR

`cells::Cpu` adalah CPU generik sederhana di `cells`. Tidak ada crate
lain yang memakainya selain internal `cells` dan contoh/test. Ini bukan
dead code compiler, tetapi kandidat relokasi ke contoh atau modul
verifikasi agar tidak terlihat sebagai bagian AVR.

### [Low] Dokumentasi terlalu kuat

`crates/avr/src/cpu/mod.rs` menyebut "structural datapath", dan
`docs/64-gate-level.md` menyebut AVR memakai `cells` untuk register
bank dan ALU. Kondisi aktual: hanya GPIO yang structural dan terhubung;
register bank/ALU/instruction register masih terputus.

### [Low] `circuit_stats()` hanya menghitung circuit GPIO

`Cpu::circuit_stats()` mengembalikan statistik `self.circuit`, yang
saat ini hanya berisi GPIO. Angka tersebut bukan statistik AVR penuh.

## 4. Rekomendasi Struktur

Target pengelompokan modul:

```text
crates/avr/src/
  core/          eksekusi, decode, fetch, state CPU
  datapath/      blok structural yang benar-benar dipakai CPU
  verification/  ALU/register/multiply/shift structural untuk audit
  peripherals/   timer, UART, ADC, SPI, TWI, system
  spec/          register map, ukuran memory, vektor, pin map
  electrical/    model pin dan Thevenin
```

Pemisahan ini membuat jelas mana yang menjadi sumber kebenaran
eksekusi dan mana yang hanya model verifikasi.

## 5. Rencana Cleanup Bertahap

### P0 - Baseline

- Kunci perilaku dengan test yang ada: 1154 test.
- Catat benchmark AVR sebelum refactor.
- Jangan mengubah perilaku dan struktur sekaligus.

### P1 - Cleanup aman

- Hapus dependency `device` dari `avr` bila tidak dipakai.
- Perbaiki dokumentasi yang mengklaim integrasi structural.
- Tambahkan `avr::spec` untuk alamat register dan ukuran memory.
- Ganti literal register map secara bertahap dengan konstanta bernama.

### P2 - Keputusan model eksekusi

Pilihan A: full structural.

- Register bank, instruction register, ALU, PC, SP, SRAM, dan flash
  dipindahkan ke `DigitalCircuit`.
- Array shadow dihapus.
- Perlu benchmark ulang karena akses memory gate-level jauh lebih
  lambat.

Pilihan B: hybrid terdisiplin.

- Jalur eksekusi fungsional tetap sebagai fast path.
- Blok structural dipindahkan ke `avr::verification` dan dipakai untuk
  equivalence test.
- Peripheral behavioral diberi label fidelity.
- Dokumentasi tidak boleh menyebut CPU full structural.

Pilihan C: dual path.

- Fast path untuk simulasi harian.
- Structural path opsional untuk audit.
- Keduanya diuji dengan hasil yang sama.

### P3 - Peripheral

- Tentukan apakah timer/UART/ADC/SPI/TWI wajib structural.
- Jika tidak, isolasi sebagai adapter behavioral dengan batas jelas.
- Jika ya, gunakan `cells::counter`, `register`, `shift_register`, dan
  memory macro yang sudah ada.

### P4 - Verifikasi

- Tambahkan equivalence test antara jalur fungsional dan structural.
- Ukur ulang performa.
- Perbarui `docs/64-gate-level.md` dan status S7.

## 6. Keputusan yang Dibutuhkan

Pertanyaan utama:

1. Apakah "AVR wajib memakai cells atau device" berarti seluruh state
   CPU harus structural di `DigitalCircuit`?
2. Atau cukup semua primitive baru memakai `cells`/`device`, sementara
   model peripheral dan fast path fungsional tetap boleh ada?
3. Apakah `device` harus dipakai untuk model electrical pin, atau
   dependency tersebut boleh dihapus?

Rekomendasi saya: pilih Pilihan B atau C lebih dulu, kecuali target
fidelitas memang full structural. Full structural adalah refactor besar
dan berisiko menurunkan performa jauh di bawah real-time.
