# 88 - Domain Digital

Dokumen ini mencatat domain digital minimal di `sim-cells` dan perintah
`emucad digital`.

## 1. Cakupan

Domain digital memakai `DigitalCircuit` dua-state dengan input dan
output bernama. Saat ini 73 entry katalog sudah dapat dijalankan:

| Kelompok | Entry |
|---|---|
| Multiplexer | 2:1, 4:1, 8:1 |
| Demux/decoder/encoder | 1:2 demux, 2:4 decoder, 3:8 decoder, 8:3 encoder, priority encoder |
| Adder/comparator | half adder, full adder, 4-bit adder, magnitude comparator |
| Parity/buffer | parity generator, parity checker, bus buffer |
| Flip-flop | D, JK, T, SR, D latch, edge D, master-slave JK, dual/quad D, octal latch |
| Counter | 4-bit, 8-bit, decade, up/down, ripple, Johnson |
| Shift register | 4-bit, 8-bit, SIPO, PISO |
| Level | level shifter, line driver, line receiver, optocoupler interface |
| Timer | timer, RTC, watchdog |
| Digital source | clock, PRBS, pattern |
| Programmable | PAL, GAL, CPLD, FPGA logic block, lookup table (LUT 4-input), FSM 3-input |
| High-Z | open-drain, tri-state, transmission gate, bidirectional buffer, bidirectional/open-drain level |
| Serial | UART, SPI, I2C master, dan CAN transmitter |
| Memory | sel SRAM/DRAM 1-bit, array 128x8, ROM, PROM, module, dan controller |

## 2. Perintah

```
emucad digital <id> [nama=0|1]... [--ticks N] [--pattern V] [--mask V] [--fsm V]
```

Contoh:

```
emucad digital logic.adder.4bit_adder a0=1 a1=1 b0=1 b2=1
emucad digital logic.counter.4bit_counter --ticks 3
emucad digital logic.flipflop.d_flipflop d=1 clk=1 --ticks 1
emucad digital logic.programmable.lookup_table a0=1 a1=1 --mask 8
emucad digital source.digital.digital_pattern_source reset=1 --pattern 1 --ticks 1
```

`--pattern`, `--mask`, dan `--fsm` menerima desimal atau hex `0x`. LUT
memakai empat input `a0..a3` dan mask 16 bit; bit ke-i adalah output
untuk kombinasi input i. FSM memakai dua bit state, satu input, dan
tabel 16 bit; default `0x0605` adalah pencacah dua bit.

Tanpa `--ticks`, rangkaian hanya di-settle satu kali. Dengan
`--ticks N`, clock global dijalankan N kali.

## 3. Port

Port ditentukan per entry, bukan dari jumlah pin katalog. Contoh:

- `logic.adder.4bit_adder`: input `a0..a3`, `b0..b3`, `cin`; output
  `s0..s3`, `cout`.
- `logic.counter.4bit_counter`: input `reset`; output `q0..q3`.
- `logic.mux.8_to_1_multiplexer`: input `in0..in7`, `sel0..sel2`;
  output `y`.
- `logic.adder.magnitude_comparator`: input `a0..a3`, `b0..b3`;
  output `eq`, `gt`, `lt`.
- `logic.flipflop.d_flipflop`: input `d`, `clk`; output `q`, `qn`.

## 4. Batas

- Net dasar hanya dua-state 0/1; tidak ada drive strength atau delay.
- High-Z dimodelkan eksplisit lewat sinyal `z`, bukan nilai ketiga pada
  net.
- Clock flip-flop adalah clock global dari `tick`, bukan clock net.
- D flip-flop memakai deteksi tepi naik dengan register clock internal.
- Counter dan shift register aktif pada setiap tick.
- Tidak ada timing, setup/hold, atau propagation delay.
- Memory array memakai sel DFF, bukan transistor, dan belum memiliki
  timing akses, refresh, atau retensi non-volatile.
- USB dan Ethernet belum didukung.

## 5. High-Z

Engine dua-state tidak memiliki nilai high-Z pada net. Entry high-Z
mengekspos sinyal `z`:

- `z=1` berarti output dilepas.
- `z=0` berarti output valid pada `y`.

Model ini memungkinkan pemakaian buffer dan open-drain secara eksplisit,
tetapi tidak melakukan resolusi bus otomatis. Penggabungan beberapa
driver harus memakai sinyal `z` di tingkat rangkaian.

## 6. I2C

`mcu.serial.i2c` adalah master 7-bit address dengan satu byte data:

- Input: `reset`, `start`, `rw`, `ack`, `addr0..addr6`, `din0..din7`.
- Output: `scl`, `sda`, `busy`, `done`, `nack`.
- Urutan bit: address MSB-first, `rw`, ACK, data MSB-first, ACK.
- Master menghasilkan START, clock per bit, ACK sampling, dan STOP.
- `scl` dan `sda` adalah nilai line dengan pull-up internal; `0` berarti
  ditarik low. Satu tick adalah satu setengah bit.
- `ack=1` berarti slave menarik SDA low pada slot ACK. Jika `ack=0`,
  output `nack` menjadi 1 dan bertahan sampai reset atau START baru.
- `sda` sudah memasukkan ACK slave pada slot ACK, tetapi tidak
  melakukan arbitrase, clock stretching, atau multi-master.

## 7. CAN

`mcu.serial.can` adalah transmitter frame CAN 2.0A:

- Input: `reset`, `start`, `ack`, `id0..id10`, `din0..din63`.
- Output: `tx`, `busy`, `done`, `nack`.
- Frame: SOF, 11-bit ID, RTR, IDE, r0, DLC 8, delapan byte data,
  CRC-15, CRC delimiter, ACK slot, ACK delimiter, EOF, dan intermission.
- CRC memakai polinomial `0x4599` dan bit stuffing diterapkan dari SOF
  sampai CRC.
- Satu tick adalah satu bit CAN; `ack=1` berarti receiver mengakui frame.
- Input ID dan data harus stabil selama transmisi.
- Tidak ada bit timing, baud rate, arbitrase, remote frame, extended ID,
  CAN FD, error frame, atau DLC selain 8.

## 8. Memory

Dua belas entry memory memakai model digital:

- `memory.device.sram_cell`: satu bit storage dengan input `we` dan
  `d`, output `q`. Nilai ditahan saat `we=0`.
- `memory.device.dram_cell`: satu bit storage dengan input `refresh`.
  Tanpa refresh atau write, data hilang setelah 15 tick. Output
  `expired` menandai window retensi habis. Ini model digital, bukan
  fisika charge.
- `memory.device.sram_array`: 128 kata x 8 bit (1024 bit), input
  `reset`, `we`, `addr0..addr6`, `din0..din7`, output `dout0..dout7`.
- EEPROM, Flash, FRAM, MRAM, dan NVRAM memakai array 128x8 yang sama
  dengan retensi setelah `reset`.
- `memory.device.rom` dan `memory.device.prom` memakai array 128x8
  dengan port `load` untuk mengisi kata. Ini hook simulasi, bukan pin
  fisik ROM. PROM hanya menerima satu load per kata; ROM dapat memuat
  ulang.
- `memory.device.memory_module` adalah bank 128x8 dengan `cs`, `we`,
  dan `oe`. Output `z=1` saat bus tidak aktif.
- `memory.device.memory_controller` adalah controller sinkron generik
  dengan `req`, `we`, `addr`, `din`, `ready`, dan `valid`. Satu
  request selesai dalam satu tick dan `ready` tinggi selama satu tick.

Array menulis satu kata pada setiap tick ketika `we=1`, lalu membaca
kata yang dialamatkan secara kombinasional. Array SRAM menghapus semua
kata saat `reset`; array non-volatile, ROM, dan PROM mempertahankan
isi. Model non-volatile belum memiliki erase, endurance, wear, atau
persistensi antar proses simulasi. DRAM hanya tersedia sebagai sel
1-bit dengan refresh global; array dan timing DRAM belum ada. SPI
flash, I2C EEPROM, dan SD card belum dipetakan karena protokol
serialnya belum dimodelkan. Controller bersifat generik, bukan
controller DRAM dengan refresh dan scheduling.

## 9. Entry Digital yang Belum Didukung

Masih ada 2 entry digital di luar domain ini:

- USB dan Ethernet.

## 10. Verifikasi

Test domain digital mencakup:

- Select mux 2:1 dan mux 8:1.
- Decoder 3:8 dan priority encoder.
- Penjumlahan 3 + 5 pada 4-bit adder.
- Comparator 5 > 3.
- Parity checker mendeteksi error.
- Bus buffer mengikuti enable.
- D flip-flop dan dual D flip-flop menangkap pada tepi naik.
- D latch menahan nilai saat enable rendah.
- Up/down counter menghitung dua arah.
- PISO menggeser data parallel ke serial.
- Watchdog timeout setelah 255 tick.
- Clock source toggle setiap tick.
- Open-drain dan tri-state memakai sinyal `z` untuk kondisi released.
- Bidirectional buffer mengarahkan satu sisi dan melepas sisi lain.
- UART mengirim start, delapan bit LSB-first, dan stop; receiver
  mengembalikan byte yang sama.
- SPI mengirim MSB-first, menerima delapan bit, dan menghasilkan `done`.
- FSM tiga input mencacah state 0, 1, 2, 3 dengan mask `0x0605`.
- Sel SRAM menahan nilai saat write enable rendah.
- Array SRAM menyimpan dan membaca kembali kata pada alamat 5 dan 100.
- Array non-volatile mempertahankan kata setelah reset.
- ROM membaca kata yang dimuat, dan PROM mengabaikan load kedua pada
  kata yang sama.
- Memory module menulis dan membaca dengan chip select, lalu melepas
  bus saat tidak dipilih.
- Memory controller menyelesaikan tulis dan baca lewat handshake
  `req`/`ready`.
- Sel DRAM kehilangan data setelah window retensi dan bertahan saat
  di-refresh.
- I2C mengirim address dan data MSB-first, menghasilkan START/STOP,
  dan menandai NACK saat ACK tidak diberikan.
- CAN mengirim frame 2.0A dengan CRC-15 dan satu bit stuff, lalu
  menandai NACK saat ACK tidak diberikan.
- PRBS source bergeser setelah reset.
- Pattern source memuat pola saat reset.
- LUT 4-input dengan mask AND menghasilkan output yang benar.
- Counter 4-bit setelah tiga tick.
- Decade counter kembali ke nol setelah sepuluh tick.
- Shift register memindahkan data.

## 11. Referensi

- Logic gate, dasar gerbang dua-state:
  https://en.wikipedia.org/wiki/Logic_gate
- Flip-flop, latch dan edge triggering:
  https://en.wikipedia.org/wiki/Flip-flop_(electronics)
- Counter, rangkaian pencacah:
  https://en.wikipedia.org/wiki/Counter_(digital)
- Multiplexer, pemilihan input:
  https://en.wikipedia.org/wiki/Multiplexer
- Static random-access memory, sel dan array:
  https://en.wikipedia.org/wiki/Static_random-access_memory
- I2C, bus open-drain dan protokol:
  https://en.wikipedia.org/wiki/I%C2%B2C
- CAN bus, frame, dan CRC:
  https://en.wikipedia.org/wiki/CAN_bus
- CRC-15-CAN, polinomial dan properti:
  https://www.can-cia.org/can-knowledge/cyclic-redundancy-check-crc-in-can-frames
- Adder, penjumlahan biner:
  https://en.wikipedia.org/wiki/Adder_(electronics)
