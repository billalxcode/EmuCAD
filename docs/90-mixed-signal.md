# 90 - Mixed-Signal Scheduler

Dokumen ini mencatat prototipe `mixed` untuk co-simulation digital
dan analog.

## 1. Tujuan

Prototipe ini membuktikan urutan deterministik:

1. Tegangan node analog dibandingkan dengan threshold untuk input digital.
2. Satu tick digital.
3. Output digital diterapkan ke sumber tegangan analog.
4. Satu langkah transient analog.
5. Tegangan node analog disimpan untuk langkah berikutnya.

## 2. API

`MixedSession` memiliki:

- `new(circuit, config, digital, link)`.
- `step() -> Result<f64, SimError>`.
- `time()`.
- `voltage()`.
- `digital_output()`.
- `pin_level()`.
- `pin_changed()`.
- `run_until_pin_change(max_steps)`.

`PinLink` menghubungkan output digital ke sumber tegangan dan node
analog. `PinInput` opsional membandingkan node analog dengan threshold
untuk menggerakkan input digital. Sumber harus berupa `VoltageSource`.

`AvrSession` membungkus `avr::Cpu`, satu pin AVR, dan satu sumber
tegangan analog. Setiap langkah menjalankan satu instruksi AVR dan
satu langkah transient analog:

- Tegangan node sebelumnya dipakai sebagai Thevenin pin input.
- Pin AVR yang dikonfigurasi output menggerakkan sumber tegangan.
- Pin AVR yang dikonfigurasi input membaca level dari node analog.

Wrapper ini memakai core AVR hybrid, bukan full-chip structural.

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

Test `analog_feedback_drives_digital_inverter`:

- RC 1 kohm dan 1 uF.
- Inverter digital membaca threshold 2.5 V.
- Output inverter menggerakkan sumber 0 V atau 5 V.
- Loop menghasilkan osilasi relaksasi deterministik.
- Tegangan tetap dalam rentang 0 V sampai 5 V.

Test `avr_pin_drives_analog_rc`:

- Program blink AVR menggerakkan PB5.
- PB5 dihubungkan lewat resistor seri 45 ohm (Thevenin) ke RC 1 kohm
  dan 1 uF.
- Node analog mengikuti level PB5.
- Program dan pin AVR berjalan bersama transient analog.

Test `avr_pin_change_breakpoint_stops`:

- `run_until_pin_change` berhenti saat level PB5 berubah.
- Dua breakpoint berturut-turut ditemukan dalam batas langkah.

## 5. Batas

- Umpan balik analog hanya satu input dan satu threshold.
- Satu tick digital sama dengan satu `dt` analog.
- AvrSession hanya satu pin dan satu sumber tegangan.
- Breakpoint hanya pada satu pin.
- Belum ada queue event atau interpolasi.
- AVR masih hybrid, bukan full-chip structural.
- Hanya sumber tegangan yang dapat diubah.

## 6. Hubungan dengan S7

Prototipe ini mencakup bagian awal S7-03 dan S7-09:

- Urutan langkah deterministik.
- Update sumber tanpa reset state analog.
- Umpan balik analog ke digital lewat threshold.
- Test regresi untuk satu arah dan loop feedback.

S7-04, S7-06, dan S7-07 masih membutuhkan wrapper AVR structural,
Thevenin pin, dan breakpoint pin.
