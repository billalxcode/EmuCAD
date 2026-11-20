# 94 - SoC Specification

Dokumen ini mencatat crate `soc` dan schema `McuSpec`.

## 1. Tujuan

`soc` menyimpan deskripsi MCU atau SoC sebagai data, bukan circuit.
Crate ini tidak membangun netlist dan tidak bergantung pada `cells`.
Dengan pemisahan ini, `cells` tetap menjadi library primitive, sedangkan
`soc` menjadi kontrak konfigurasi untuk builder MCU di masa depan.

## 2. Batas Crate

- `soc` hanya bergantung pada `kernel` untuk `SimError`.
- `soc` tidak bergantung pada `cells`, `device`, `analysis`, atau `avr`.
- `cells` dan `soc` tidak saling bergantung.
- Builder MCU nantinya boleh memakai `cells` dan `soc` bersama-sama.
- Adapter seperti `cli` atau `gui` dapat membaca spec tanpa mengetahui
  detail implementasi MCU.

## 3. Schema

`McuSpec` berisi:

- `name`.
- `cpu`.
- `clocks`.
- `buses`.
- `memories`.
- `peripherals`.
- `pins`.

`CpuSpec` berisi:

- `data_width`: 8, 16, 32, atau 64.
- `address_width`: 1 sampai 64.
- `register_count`: minimal 1.
- `isa`: nama ISA, misalnya `avr`, `rv32i`, atau `cortex-m3`.

`ClockSpec` berisi nama dan `frequency_hz`. Frequency harus positif.

`BusSpec` berisi:

- `name`.
- `protocol`: Internal, APB, AHB-Lite, AXI-Lite, Wishbone, atau Other.
- `data_width`: 1 sampai 64.
- `address_width`: 1 sampai 64.

`MemorySpec` berisi:

- `name`.
- `kind`: RAM, ROM, Flash, EEPROM, FRAM, atau Other.
- `address_bits`: 1 sampai 64.
- `data_bits`: 1 sampai 64.

`PeripheralSpec` berisi:

- `name`.
- `kind`: Timer, UART, SPI, I2C, GPIO, ADC, DAC, PWM, DMA,
  InterruptController, atau Other.
- `base_address`.
- `irq` opsional.

`PinSpec` berisi nama pin dan fungsi atau alternate function.

## 4. Validasi

`McuSpec::validate` memeriksa:

- Nama MCU tidak kosong.
- CPU valid.
- Minimal satu clock.
- Minimal satu memory.
- Nama clock, bus, memory, peripheral, dan pin unik.
- Frequency clock positif.
- Width bus dan memory valid.
- Base address peripheral unik.

Validasi mengembalikan `SimError::InvalidNetlist` dengan pesan yang
menyebut field yang salah.

## 5. Contoh

```rust
use soc::{BusProtocol, BusSpec, ClockSpec, CpuSpec, McuSpec, MemoryKind, MemorySpec};

let spec = McuSpec {
    name: "example".to_string(),
    cpu: CpuSpec {
        data_width: 32,
        address_width: 32,
        register_count: 32,
        isa: "rv32i".to_string(),
    },
    clocks: vec![ClockSpec {
        name: "core".to_string(),
        frequency_hz: 32_000_000,
    }],
    buses: vec![BusSpec {
        name: "apb".to_string(),
        protocol: BusProtocol::Apb,
        data_width: 32,
        address_width: 32,
    }],
    memories: vec![MemorySpec {
        name: "sram".to_string(),
        kind: MemoryKind::Ram,
        address_bits: 12,
        data_bits: 32,
    }],
    peripherals: Vec::new(),
    pins: Vec::new(),
};
assert!(spec.validate().is_ok());
```

## 6. Belum Termasuk

- Address map hierarchy dan region overlap antar memory.
- IRQ map lengkap dan prioritas.
- Pin mux matrix.
- Fidelity per blok.
- Elaboration ke `cells`.
- Generator peripheral dari register map.

Item tersebut masuk backlog `TODO-CELLS.md` setelah P0 selesai.
