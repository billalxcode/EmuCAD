# EmuCAD

**Native Transistor-Level Emulator** - simulator elektronika native Rust
yang memodelkan transistor, gerbang CMOS, CPU AVR, peripheral, dan board,
serta mampu menjalankan firmware AVR.

> **Status v0.1:** 1167 test lulus, 10 crate, crate simulasi zero external
> dependency; dependensi GUI dan binding Python dibatasi pada daftar yang
> disetujui di `AGENTS.md`.
> Rilis biner GitHub mengikuti versi workspace; aset dan SHA512
> diperbarui setiap merge ke `main`.

Dokumentasi lengkap: [`docs/README.md`](docs/README.md).
Panduan pengguna: [`docs/43-user-guide.md`](docs/43-user-guide.md).
Panduan GUI: [`docs/58-gui-user-guide.md`](docs/58-gui-user-guide.md).
Fidelity dan batasan: [`docs/24-fidelity.md`](docs/24-fidelity.md).

## Struktur Crate

Seluruh crate memakai nama pendek tanpa prefix:

- `kernel`, `device`, `analysis`, `cells`, `avr`, `board`, `mixed`.
- `cli` menghasilkan binary `emucad`.
- `gui` menghasilkan binary `emucad-gui`.
- `python` menghasilkan extension module `emucad`.

Config GUI disimpan di `~/.config/emucad/gui.json`. Format proyek
memakai `emucad-gui`.

---

## 1. Apa ini?

Satu binary native Rust yang:

- Memodelkan rangkaian pada level transistor.
- Memiliki solver SPICE sendiri: MNA, Newton-Raphson, DC, transient, AC,
  noise, distorsi, dan sensitivitas.
- Memodelkan sel CMOS, CPU AVR, peripheral, dan board.
- Memuat Intel HEX dan ELF32.
- Berjalan tanpa VM, browser, atau simulator eksternal saat runtime.
- Hanya memakai Rust standard library dan crate internal workspace; binding
  Python memakai `pyo3` di crate terpisah.
- Tidak memakai `unsafe` manual; glue FFI `pyo3` diisolasi di `python`.

Target akhirnya adalah **firmware-observable equivalence** untuk board
profile tertentu, bukan replika silikon identik.

---

## 2. Kemampuan saat ini

| Area | Isi |
|---|---|
| Solver analog | DC operating point, DC sweep, transient tetap dan adaptive, AC, noise, distorsi, sensitivitas |
| Device | Resistor, capacitor, inductor, sumber, pulse source, diode, MOSFET N/P, BJT NPN/PNP, JFET N/P, transmission line, VCVS, VCCS, CCCS, CCVS, switch |
| Sel digital | Inverter, NAND, NOR, buffer, DFF, latch, SRAM, adder, ALU |
| AVR | ISA, decoder, ALU, register, SRAM, flash, GPIO, timer, UART, SPI, TWI, ADC |
| Board | LED, resistor, button, UART host, pin electrical, CSV, VCD |
| Format | Intel HEX, ELF32, CSV, VCD digital, VCD analog |
| Virtual serial | STK500 upload, firmware run, TX/RX UART, sketsa Arduino core, cleanup otomatis |
| Virtual USB | C++ bridge, ConfigFS ACM, `/dev/ttyACM0`, STK500v1, upload avrdude |
| CLI | `run`, `monitor`, `disasm`, `bench`, `serial`, `usb` |

---

## 3. Quick start

### 3.1 Build dan test

```bash
cargo build --workspace
cargo test --workspace
```

### 3.2 Contoh runnable

```bash
cargo run -p analysis --example rc_analysis
cargo run -p analysis --example inverter_vtc
cargo run -p cells --example ring_oscillator
cargo run -p avr --example cpu_program
cargo run -p board --example blink
cargo run -p avr --example uart_hello
cargo run -p avr --example timer_interrupt
```

### 3.3 CLI

```bash
./target/debug/emucad monitor E005 E013 0F01 9598
./target/debug/emucad disasm E005 E013 0F01 9598
./target/debug/emucad bench 1000 10
./target/debug/emucad catalog --json
./target/debug/emucad run firmware/blink.hex 100000 --realtime
./target/release/emucad serial
./target/release/emucad
./target/release/emucad usb --socket /tmp/emucad-usb.sock
```

Perintah `run` memerlukan file Intel HEX. Contoh firmware assembly ada di
[`firmware/`](firmware/README.md). Menjalankan binary tanpa argumen, atau
menggunakan subcommand `serial`, membuka virtual serial port STK500 melalui
`socat` dan menutupnya otomatis saat Ctrl-C. Arduino IDE GUI mungkin tidak
mendeteksi PTY; gunakan `arduino-cli` atau custom discovery.

### 3.4 Pemeriksaan lengkap

```bash
bash .opencode/skills/clean-code/scripts/check.sh
./scripts/package.sh
```

---

## 4. Struktur workspace

| Crate | Peran |
|---|---|
| `kernel` | Waktu, satuan, node, device, matrix, error, netlist |
| `device` | Model device dan stamping MNA |
| `analysis` | DC, transient, AC, noise, distorsi, sensitivitas |
| `cells` | Sel CMOS struktural dan pustaka digital |
| `avr` | CPU AVR fast functional core, decoder, peripheral, primitif struktural |
| `board` | Board, LED, button, UART host, Intel HEX, ELF, CSV/VCD |
| `cli` | Binary `emucad`, monitor, disassembler, benchmark, catalog JSON |
| `python` | Binding Python untuk katalog dan builder netlist |
| `drivers/` | Scaffold driver virtual USB C++ (opsional, di luar crate) |

Aturan utama:

- `kernel` tidak bergantung pada crate lain.
- `cli` adalah adapter dan komposisi.
- `python` adalah adapter binding Python untuk katalog dan netlist.
- Format CSV, VCD, HEX, dan ELF berada di adapter.
- Logika simulasi tidak berada di `main.rs`.

Detail ada di [`docs/02-architecture.md`](docs/02-architecture.md).

---

## 5. Dokumentasi

Mulai dari [`docs/README.md`](docs/README.md) untuk peta lengkap.

Dokumen yang sering dipakai:

| Dokumen | Isi |
|---|---|
| [`AGENTS.md`](AGENTS.md) | Aturan wajib agent dan kontributor |
| [`docs/43-user-guide.md`](docs/43-user-guide.md) | Build, CLI, library, device, firmware |
| [`docs/12-device-models.md`](docs/12-device-models.md) | Katalog device dan parameter |
| [`docs/13-analysis.md`](docs/13-analysis.md) | Solver dan analisis |
| [`docs/24-fidelity.md`](docs/24-fidelity.md) | Akurasi dan batasan model |
| [`docs/29-ac.md`](docs/29-ac.md) | AC small-signal |
| [`docs/53-release-notes.md`](docs/53-release-notes.md) | Isi rilis v0.1 |
| [`docs/57-gui-egui.md`](docs/57-gui-egui.md) | Desain GUI egui dan kanvas CAD |
| [`docs/58-gui-user-guide.md`](docs/58-gui-user-guide.md) | Panduan pengguna GUI |
| [`docs/59-gui-golden.md`](docs/59-gui-golden.md) | Status dan prosedur golden RGBA GUI |
| [`docs/60-component-catalog.md`](docs/60-component-catalog.md) | Katalog 500 komponen, akurasi, dan schema |
| [`docs/90-mixed-signal.md`](docs/90-mixed-signal.md) | Scheduler co-sim digital-analog dengan AVR, ADC, PWM, dan UART |
| [`docs/91-avr-audit.md`](docs/91-avr-audit.md) | Audit struktur AVR, integrasi cells, dan rencana cleanup |
| [`docs/92-cells-mcu-requirements.md`](docs/92-cells-mcu-requirements.md) | Kesiapan cells untuk multi-MCU dan prioritas library |
| [`docs/93-catalog-coverage.md`](docs/93-catalog-coverage.md) | Cakupan deterministik 500 entri katalog |
| [`docs/61-design-system.md`](docs/61-design-system.md) | Design system GUI, font, warna, dan spacing |
| [`TODO-COMPONENTS.md`](TODO-COMPONENTS.md) | Backlog katalog komponen dan Python builder |
| [`TODO-CELLS.md`](TODO-CELLS.md) | Backlog library cells untuk multi-MCU |
| [`TODO.md`](TODO.md) | Pekerjaan yang tersisa |
| [`TODO-GUI.md`](TODO-GUI.md) | Backlog GUI egui |

---

## 6. Batas dan status verifikasi

Sudah:

- 1167 test workspace.
- fmt, clippy, test, dan build release lulus.
- Paket Linux dapat dibuat.
- Determinisme Linux diuji.
- Firmware `avr-gcc` bare-metal dan sketsa Arduino core
  `Serial.println("Hello World")` berjalan.
- GUI egui untuk editing skematik, simulasi PB5, monitor UART TX,
  graph digital, minimap, dan export PNG.
- Co-sim AVR hybrid dengan peta 24 pin IO, ADC multi-channel, PWM
  multi-load, UART TX/RX line-level, dan queue event.
- Library GUI memuat 6 komponen kurasi dan 500 entri katalog dengan 16 kategori, definisi pin, dan simbol SVG unik per entri.
- Design system GUI berbasis Adobe Spectrum dengan font Source Sans 3 dan Source Code Pro, token warna dark/light, serta spacing yang konsisten.

Belum:

- Golden comparison dengan ngspice.
- Upload dan Serial Monitor dari GUI Arduino IDE.
- Determinisme macOS dan Windows.
- Eksekusi GitHub Actions.
- Probe generik dan grafik analog.
- Parser netlist SPICE.

Model device memakai Level 1 atau model sederhana. Rincian ada di
[`docs/24-fidelity.md`](docs/24-fidelity.md).

---

## 7. Lisensi

MIT. Lihat [`LICENSE`](LICENSE).
