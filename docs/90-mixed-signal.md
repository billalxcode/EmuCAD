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
- `with_pin_map(...)` untuk peta 3 port x 8 bit.
- `with_links(...)` untuk pin, satu ADC, dan PWM.
- `with_bridges(...)` untuk beberapa link ADC dan PWM opsional.
- `with_pwm_links(...)` untuk beberapa link ADC dan PWM.
- `with_uart(link)` untuk endpoint UART eksternal.
- `with_uart_tx(link)` untuk garis TX UART ke sumber analog.
- `with_uart_rx(link)` untuk garis RX UART dari node analog.
- `step()`, `time()`, `cpu()`, dan `cpu_mut()`.
- `pin_level(index)`, `pin_changed(index)`, `any_pin_changed()`, dan
  `pin_count()`.
- `node_voltage(index)` dan `voltage()` untuk link pertama.
- `adc_voltage()`, `adc_voltage_at(index)`, `pwm_voltage()`,
  `pwm_voltage_at(index)`, `uart_link()`, `uart_link_mut()`,
  `uart_tx_voltage()`, dan `uart_rx_voltage()`.
- `timer0_overflow()`, `timer0_compare()`, dan
  `run_until_timer0_event(event, max_steps)`.
- `take_event()`, `event_count()`, dan `drain_events()` untuk queue
  event.
- `run_until_pin_change(max_steps)`.

`AvrPinMap` memetakan pin berdasarkan pasangan port dan bit untuk 3
port x 8 bit. `add` menolak lokasi ganda atau di luar rentang, sedangkan
`links` mengembalikan pin dalam urutan port dan bit. `with_pin_map`
membangun session dari peta yang sudah tervalidasi. Link pin langsung
juga divalidasi untuk port 0..2 dan bit 0..7.

`PinLink` menghubungkan output digital ke sumber tegangan dan node
analog. `PinInput` opsional membandingkan node analog dengan threshold
untuk menggerakkan input digital. Sumber harus berupa `VoltageSource`.

`AdcLink` memetakan satu channel ADC ke node analog dan reference
voltage. Satu session dapat memakai beberapa link ADC. Channel harus
unik, berada di bawah 16, dan reference harus positif. Setiap langkah
menghitung kode 10-bit dari rasio tegangan node terhadap reference,
lalu menyimpannya sebagai input channel sebelum instruksi CPU berjalan.

`PwmLink` memetakan output Timer0 PWM channel A ke sumber tegangan
analog dan node pengamatan. Satu session dapat memakai beberapa link
PWM. Sumber harus unik, sedangkan nilai `high` dan `low` harus finite
serta menentukan tegangan sumber saat PWM aktif atau tidak.

`UartLink` adalah buffer byte eksternal untuk USART. `push_rx`
mengantrekan byte untuk AVR, sedangkan `take_tx` mengambil byte yang
sudah selesai dikirim. Bridge mengirim satu byte RX saat RXC clear dan
mengumpulkan byte TX setelah frame selesai.

`UartTxLink` menghubungkan garis TX UART ke sumber tegangan analog.
Nilai `high` dan `low` harus finite. Garis TX mengikuti timing frame
UART.

`UartRxLink` mengambil satu node analog dengan threshold dan
mengumpankannya ke receiver 8N1 bertiming. Sampling memakai 16x clock
normal atau 8x saat double speed. Delapan data bit dan parity even/odd
mengikuti UCSR0C. Buffer byte `UartLink` tetap tersedia untuk jalur RX
byte-level.

`Timer0Event` memilih `Overflow` atau `Compare`. Timer0 menyimpan
counter event monotonik sejak konstruksi. `AvrSession` mengambil selisih
counter setelah setiap instruksi, sehingga event dapat dibaca tanpa
mengambil flag yang dipakai firmware atau interupsi.

`CoSimEvent` mencatat `kind`, `cycles`, dan `time`. Queue menggabungkan
event pin, Timer0, UART TX, UART RX, edge PWM, edge garis TX UART, dan
konversi ADC. Event CPU memakai waktu
analog awal langkah, sedangkan event pin memakai waktu akhir langkah.
Pemanggil harus mengambil event secara berkala agar queue tidak tumbuh
terus.

`AvrSession` membungkus `avr::Cpu`, satu atau beberapa pin AVR, dan
opsional beberapa link ADC, beberapa link PWM, serta satu link UART. Setiap
langkah menjalankan satu instruksi AVR dan satu langkah transient
analog:

- Tegangan node sebelumnya dipakai sebagai Thevenin pin input.
- Pin AVR yang dikonfigurasi output menggerakkan sumber tegangan.
- Pin AVR yang dikonfigurasi input membaca level dari node analog.
- Input ADC memakai tegangan node dari langkah sebelumnya.
- Output PWM menggerakkan sumber tegangan analog.
- Byte RX dikirim ke USART sebelum instruksi CPU.
- Byte TX yang selesai dikirim dikumpulkan setelah instruksi CPU.
- Garis TX UART menggerakkan sumber analog mengikuti level frame.
- Garis RX UART diset dari sampel analog sebelumnya sebelum instruksi.
- Event Timer0 pada langkah tersebut dicatat tanpa menghapus flag model.
- Event pin, Timer0, UART TX, UART RX, edge PWM, edge garis TX UART,
  dan ADC masuk queue dengan timestamp.

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
- Event `AdcConversion` mencatat channel 0 dan kode hasil.

Test `multiple_adc_channels_sample_independent_nodes`:

- Dua node analog berada pada 1.25 V dan 3.75 V.
- Setiap channel membaca node yang berbeda.
- Konversi menghasilkan kode sekitar 256 dan 767.

Test `adc_links_reject_duplicate_and_invalid`:

- Dua link dengan channel yang sama ditolak.
- Channel 16 ditolak.
- Reference nol ditolak.

Test `timer_pwm_drives_analog_node`:

- Timer0 memakai fast PWM, compare 128, dan clock clk/1.
- Output PWM menggerakkan RC 10 kohm dan 100 nF.
- Node analog mencapai level rendah dan tinggi.
- Event `PwmEdge` mencatat transisi tinggi dan rendah.

Test `timer_pwm_drives_multiple_analog_nodes`:

- Satu output Timer0 menggerakkan dua jaringan RC.
- Kedua node mengikuti level PWM yang sama.
- Kedua node mencapai level rendah dan tinggi.

Test `pwm_links_reject_duplicate_sources_and_invalid_voltage`:

- Dua link dengan sumber yang sama ditolak.
- Tegangan PWM tak hingga ditolak.

Test `uart_transmit_reaches_external_link`:

- USART diaktifkan dan satu byte dimuat ke transmitter.
- Setelah frame selesai, `take_tx` mengembalikan byte tersebut.
- Buffer TX kosong setelah byte diambil.
- Event `UartTx(0x41)` tercatat pada 160 siklus CPU.

Test `uart_receive_reaches_avr_register`:

- Byte eksternal diantrekan lewat `push_rx`.
- Firmware membaca UDR0 dengan `lds` ke register.
- RXC clear setelah pembacaan.

Test `uart_tx_line_drives_analog_node`:

- Garis TX UART dihubungkan ke sumber analog.
- Start bit dan level data menggerakkan node analog.
- Node mencapai level rendah dan tinggi selama frame.
- Event `UartTxEdge` mencatat transisi garis TX.

Test `uart_tx_line_loops_back_to_rx`:

- Node TX dihubungkan kembali ke RX.
- Frame 8N1 dikirim dan diterima sebagai byte yang sama.
- Receiver menyelesaikan frame dalam batas langkah.
- Event `UartRx(0x5A)` tercatat di queue.

Test `parity_bit_is_transmitted_in_even_mode`:

- UCSR0C memakai even parity.
- Byte dengan satu bit tinggi mengirim parity 1.
- Byte dengan dua bit tinggi mengirim parity 0.

Test `line_receiver_checks_even_parity`:

- Frame dengan parity benar diterima.
- Frame dengan parity salah tidak menyalakan RXC.

Test `timer0_compare_event_stops_session`:

- Timer0 CTC compare 4 dengan clock clk/1.
- Breakpoint pertama berhenti pada 4 siklus CPU, kedua pada 8.
- Event berulang terdeteksi tanpa menghapus flag.

Test `timer0_overflow_event_stops_session`:

- Timer0 normal mode compare 200.
- Overflow terdeteksi pada 256 siklus CPU.
- `timer0_overflow` true pada langkah berhenti.

Test `timer0_event_queue_records_time_and_cycles`:

- Timer0 CTC compare 4 pada clock clk/1.
- Event `Timer0Compare` tercatat pada 4 siklus CPU.
- Timestamp event sama dengan awal langkah kedua.

Test `pin_event_queue_records_level`:

- Program blink mengubah level PB5.
- Event `PinChanged` untuk index 0 dan level tinggi masuk queue.
- Timestamp event lebih besar dari nol.

Test `pin_map_tracks_one_pin_per_location`:

- Satu lokasi port dan bit menyimpan satu pin.
- Duplikat, port 3, dan bit 8 ditolak.
- `get`, `remove`, `len`, dan `is_empty` konsisten.

Test `pins_reject_invalid_locations`:

- Link pin dengan port 3 ditolak.
- Link pin dengan bit 8 ditolak.

Test `avr_pin_map_links_all_io`:

- Peta berisi 24 pin IO dari port B, C, dan D.
- Urutan link dimulai dari port 0 bit 0 dan berakhir port 2 bit 7.
- Session berjalan dengan 24 link pin.

## 5. Batas

- Umpan balik analog hanya satu input dan satu threshold.
- Satu tick digital sama dengan satu `dt` analog.
- AvrSession dapat memakai beberapa link pin, beberapa link ADC,
  beberapa link PWM, dan satu link UART.
- Pemetaan 24 pin IO model (3 port x 8 bit) tersedia.
- Pin daya, reset, dan paket 32 pin penuh belum dipetakan.
- Index pin pada session mengikuti urutan `links`, bukan nomor package.
- Breakpoint pin berlaku untuk semua link yang terdaftar.
- Queue event gabungan tersedia dengan resolusi satu langkah.
- Interpolasi dan timestamp sub-langkah belum ada.
- AVR masih hybrid, bukan full-chip structural.
- Hanya sumber tegangan yang dapat diubah.
- PWM baru mencakup Timer0 channel A.
- UART level garis mendukung 8 data bit dan parity even/odd.
- Frame 9 data bit, parity 9-bit, dan flow control belum.
- Buffer RX byte-level menahan byte berikutnya selama RXC masih set.

## 6. Hubungan dengan S7

Prototipe ini mencakup bagian S7-03, S7-04, S7-06, S7-07, S7-08, dan
S7-09:

- Urutan langkah deterministik.
- Update sumber tanpa reset state analog.
- Umpan balik analog ke digital lewat threshold.
- Wrapper AVR hybrid dengan beberapa link pin dan peta 24 pin IO.
- Thevenin pin input dan breakpoint perubahan pin.
- ADC multi-channel, PWM Timer0 multi-load, serta UART TX dan RX
  line-level.
- Breakpoint event Timer0 untuk overflow dan compare.
- Queue event gabungan untuk pin, Timer0, UART TX/RX, edge PWM, edge
  garis TX UART, dan ADC.
- Test regresi untuk satu arah, loop feedback, dan bridge ADC/PWM/UART.

S7-04 tetap hybrid, bukan full-chip structural. S7-05 sudah mencakup
semua pin IO yang dimodelkan, tetapi belum pin daya, reset, atau paket
32 pin penuh. S7-08 belum mencakup UART 9-bit dan flow control; S7-09
belum mencakup interpolasi dan timestamp sub-langkah.
