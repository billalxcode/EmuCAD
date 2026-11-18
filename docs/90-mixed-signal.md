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

`AvrSession` memiliki:

- `new(...)` untuk satu pin.
- `with_pins(...)` untuk beberapa link pin.
- `with_links(...)` untuk pin, ADC, dan PWM.
- `step()`, `time()`, `cpu()`, dan `cpu_mut()`.
- `pin_level(index)`, `pin_changed(index)`, dan `any_pin_changed()`.
- `node_voltage(index)` dan `voltage()` untuk link pertama.
- `adc_voltage()` dan `pwm_voltage()`.
- `run_until_pin_change(max_steps)`.

`PinLink` menghubungkan output digital ke sumber tegangan dan node
analog. `PinInput` opsional membandingkan node analog dengan threshold
untuk menggerakkan input digital. Sumber harus berupa `VoltageSource`.

`AdcLink` memetakan satu channel ADC ke node analog dan reference
voltage. Setiap langkah menghitung kode 10-bit dari rasio tegangan node
terhadap reference, lalu menyimpannya sebagai input channel sebelum
instruksi CPU berjalan.

`PwmLink` memetakan output Timer0 PWM channel A ke sumber tegangan
analog dan node pengamatan. Nilai `high` dan `low` menentukan tegangan
sumber saat PWM aktif atau tidak.

`AvrSession` membungkus `avr::Cpu`, satu atau beberapa pin AVR, dan
opsional satu link ADC serta satu link PWM. Setiap langkah menjalankan
satu instruksi AVR dan satu langkah transient analog:

- Tegangan node sebelumnya dipakai sebagai Thevenin pin input.
- Pin AVR yang dikonfigurasi output menggerakkan sumber tegangan.
- Pin AVR yang dikonfigurasi input membaca level dari node analog.
- Input ADC memakai tegangan node dari langkah sebelumnya.
- Output PWM menggerakkan sumber tegangan analog.

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

Test `avr_multiple_links_share_one_cpu`:

- Satu CPU AVR menggerakkan dua link analog.
- Kedua node RC mengikuti level PB5.
- `pin_level(0)` dan `pin_level(1)` konsisten.

Test `analog_node_feeds_avr_adc`:

- Node analog tetap pada 2.5 V dengan reference 5.0 V.
- Konversi menghasilkan kode sekitar 512.
- Flag konversi set dan status busy clear.

Test `timer_pwm_drives_analog_node`:

- Timer0 memakai fast PWM, compare 128, dan clock clk/1.
- Output PWM menggerakkan RC 10 kohm dan 100 nF.
- Node analog mencapai level rendah dan tinggi.

## 5. Batas

- Umpan balik analog hanya satu input dan satu threshold.
- Satu tick digital sama dengan satu `dt` analog.
- AvrSession dapat memakai beberapa link pin, satu link ADC, dan satu
  link PWM.
- Belum ada pemetaan penuh 32 pin.
- Breakpoint berlaku untuk semua link yang terdaftar.
- Belum ada queue event atau interpolasi.
- AVR masih hybrid, bukan full-chip structural.
- Hanya sumber tegangan yang dapat diubah.
- PWM baru mencakup Timer0 channel A.
- UART belum terhubung ke solver analog.

## 6. Hubungan dengan S7

Prototipe ini mencakup bagian S7-03, S7-04, S7-06, S7-07, S7-08, dan
S7-09:

- Urutan langkah deterministik.
- Update sumber tanpa reset state analog.
- Umpan balik analog ke digital lewat threshold.
- Wrapper AVR hybrid dengan beberapa link pin.
- Thevenin pin input dan breakpoint perubahan pin.
- ADC satu channel dan PWM Timer0 channel A.
- Test regresi untuk satu arah, loop feedback, dan bridge ADC/PWM.

S7-04 tetap hybrid, bukan full-chip structural. S7-05 baru mencakup
link pin manual, bukan pemetaan 32 pin. S7-08 belum mencakup UART dan
event timer.
