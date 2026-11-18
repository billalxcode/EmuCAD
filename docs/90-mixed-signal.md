# 90 - Mixed-Signal One-Way Scheduler

Dokumen ini mencatat prototipe `sim-mixed` untuk co-simulation digital
dan analog.

## 1. Tujuan

Prototipe ini membuktikan urutan deterministik:

1. Satu tick digital.
2. Output digital diterapkan ke sumber tegangan analog.
3. Satu langkah transient analog.
4. Tegangan node analog dikembalikan ke pemanggil.

## 2. API

`MixedSession` memiliki:

- `new(circuit, config, source, output, high, low)`.
- `step() -> Result<f64, SimError>`.
- `time()`.
- `clock()`.

Sumber harus berupa `VoltageSource`. Output digital diambil dari clock
entry `source.digital.digital_clock_source`.

## 3. Update Sumber

`TransientSession::set_source_voltage` mengubah nilai sumber tegangan
tanpa membangun ulang model. State kapasitor dan induktor tetap
dipertahankan. Ini yang memungkinkan co-simulation tanpa reset
transient.

## 4. Verifikasi

Test `clock_toggles_source_and_preserves_capacitor_state`:

- RC 100 ohm dan 1 nF.
- Clock digital mengubah sumber antara 0 V dan 5 V.
- Langkah pertama menghasilkan tegangan tinggi.
- Langkah kedua menghasilkan tegangan rendah.
- Waktu analog naik tepat satu `dt` per tick.

## 5. Batas

- Hanya satu arah digital ke analog.
- Satu tick digital sama dengan satu `dt` analog.
- Belum ada queue event, interpolasi, atau breakpoint pin.
- Belum terhubung ke AVR atau `sim-board`.
- Hanya sumber tegangan yang dapat diubah.
- Tidak ada pembacaan analog kembali ke digital.

## 6. Hubungan dengan S7

Prototipe ini mencakup bagian awal S7-03 dan S7-09:

- Urutan langkah deterministik.
- Update sumber tanpa reset state analog.
- Test regresi untuk arah digital ke analog.

S7-04, S7-06, dan S7-07 masih membutuhkan wrapper AVR structural,
Thevenin pin, dan breakpoint pin.
