# 15 - Digital Abstraction

Dokumen ini menjelaskan lapisan logika digital dua-state di `cells`.

---

## 1. Tujuan

Lapisan ini menjalankan blok sekuensial besar seperti register file, counter,
dan CPU uji tanpa harus menyelesaikan seluruh rangkaian pada level transistor
untuk setiap clock.

Lapisan ini bukan pengganti model transistor. Sel CMOS di `cells` tetap
menjadi model elektrik. Lapisan digital dipakai untuk komposisi gate-level
setelah perilaku sel diverifikasi.

---

## 2. Model

- Nilai net hanya `false` (0) dan `true` (1).
- Gate kombinasional: NOT, AND, OR, XOR, NAND, NOR, MUX.
- Elemen state: D flip-flop rising-edge.
- Clock ideal: satu panggilan `tick()` adalah satu tepi clock.
- Reset ideal: `reset()` mengosongkan seluruh flip-flop.
- Settle dilakukan sampai stabil.
- Loop kombinasional yang tidak stabil menghasilkan
  `SimError::ConvergenceFailure`.
- Batas settle adalah 10.000 iterasi.

Semua operasi deterministik dan tidak memakai wall-clock.

---

## 3. API

```rust
pub struct DigitalBuilder {
    // ...
}

impl DigitalBuilder {
    pub fn input(&mut self, name: &str) -> NetId;
    pub fn output(&mut self, name: &str, net: NetId);
    pub fn constant(&mut self, value: bool) -> NetId;
    pub fn not(&mut self, input: NetId) -> NetId;
    pub fn and(&mut self, first: NetId, second: NetId) -> NetId;
    pub fn or(&mut self, first: NetId, second: NetId) -> NetId;
    pub fn xor(&mut self, first: NetId, second: NetId) -> NetId;
    pub fn nand(&mut self, first: NetId, second: NetId) -> NetId;
    pub fn nor(&mut self, first: NetId, second: NetId) -> NetId;
    pub fn mux(&mut self, first: NetId, second: NetId, select: NetId) -> NetId;
    pub fn dff(&mut self, data: NetId) -> NetId;
    pub fn build(self) -> DigitalCircuit;
}
```

`DigitalCircuit` menyediakan `set_input`, `output`, `settle`, `tick`, dan
`reset`.

---

## 4. Blok Dasar

- `and_all` dan `or_all` mereduksi vektor net.
- `mux_tree` memilih satu dari `2^n` input.
- `decoder` menghasilkan one-hot output.
- `counter` adalah ripple counter dengan enable.
- `counter_loadable` menambahkan load, up/down, dan hold.
- `counter_match` dan `prescaler` menyediakan compare dan tick
  periodik.
- `shift_register` menggeser saat enable tinggi.
- `read_bits` membaca vektor bit little-endian.
- `register_file` menyediakan `count x width` dengan decoder tulis dan
  dua read port.
- `register_file_masked` menambahkan byte write mask.
- `barrel_shift_left`, `barrel_shift_right`, `arithmetic_shift_right`,
  `rotate_left`, `rotate_right`, `funnel_shift_left`, dan
  `funnel_shift_right` membangun shifter kombinasional.
- `comparator_unsigned` menghasilkan less, equal, dan greater.
- `priority_encoder`, `one_hot_encoder`, dan `leading_zero_count`
  membangun encoder dan hitungan nol di depan.
- `ProgramCounter` adalah register 16-bit dengan load dan increment.
- `StackPointer` adalah register 16-bit dengan load, increment, dan decrement.
- `register` adalah register loadable yang dipakai IR dan register file.
- `register_extended` menambahkan clear, preset, dan byte mask.

Semua blok diuji secara fungsional dengan clock ideal.

---

## 5. Batas Fidelity

- Belum ada delay gerbang.
- Belum ada nilai X atau high-Z.
- Belum ada timing setup/hold.
- Belum ada konsumsi daya.
- Clock belum terdistribusi sebagai net fisik.
- Lapisan ini tidak membuktikan perilaku elektrik.

Blok yang memakai lapisan ini harus dicatat sebagai gate-level, bukan
transistor-level, sampai diverifikasi ulang dengan model elektrik.

---

## 6. Test

Test lapisan digital mencakup:

- Truth table gate kombinasional.
- MUX.
- D flip-flop capture dan hold.
- Reset.
- Deteksi loop kombinasional.
- Decoder one-hot.
- MUX tree.
- Counter dan hold.
- Shift register dan hold.
- Barrel shift, arithmetic shift, rotate, dan funnel shift.
- Multiplier array dan Booth multiplier.
- Divider restoring dan non-restoring.
- Comparator, priority encoder, one-hot encoder, dan leading-zero
  count.
- Bit reverse, byte swap, popcount, dan saturating arithmetic.

## 7. Register File

- 32 register x 8 bit.
- Decoder 5-ke-32 memilih register tulis.
- Write enable menjaga isi saat rendah.
- Dua read port 32:1 berbasis mux tree.
- Reset mengosongkan seluruh register.
- Test menulis dan membaca 32 register, hold, port independen, dan reset.
- `register_file_masked` mendukung width 1..=64, jumlah register 2..=32,
  byte write mask, dan 2 read port.
- Test 32 x 32 memeriksa word utuh, retensi byte yang tidak dimask,
  kedua read port, dan validasi lebar mask.
- `register_extended` mendukung load, clear, preset, dan byte mask
  opsional untuk register tunggal.
- Prioritas kontrol: clear, preset, load, lalu hold.

## 7.1 Counter dan Prescaler

- `counter_loadable` mendukung width 1..=64, load, enable, dan up/down.
- Load memiliki prioritas di atas enable; enable rendah menahan nilai.
- `counter_match` membandingkan bit counter dengan nilai target.
- `prescaler(width, divisor, enable)` menghasilkan `count` dan `tick`.
- `tick` tinggi selama satu cycle ketika counter mencapai
  `divisor - 1`.
- Divisor valid 1..=2^width untuk width 1..=32.
- Test memeriksa load, hitung naik/turun, wrap, hold, match, dan tick.

## 7.2 FIFO Sinkron

- `fifo_sync` mendukung width 1..=64 dan depth power-of-two 2..=32.
- Penyimpanan memakai `register_file`; pointer tulis dan baca memakai
  `counter_loadable`.
- Status `full` dan `empty` dihitung dari counter jumlah entri.
- Write saat full dan read saat empty diabaikan.
- Read dan write bersamaan menahan count.
- Test mengisi, mengosongkan, menolak write saat full, dan menolak read
  saat empty.

## 7.3 Memory Sinkron

- `memory_sync` mendukung depth 2..=32, width 1..=64, dan byte mask
  opsional.
- Latency 0 menghasilkan read kombinasional; latency 1 mendaftarkan
  read data saat `read_enable` tinggi.
- `ReadDuringWrite::ReadFirst` mengembalikan nilai lama; `WriteFirst`
  mem-bypass write data pada alamat yang sama.
- Storage di-reset ke nol; initial contents arbitrary belum didukung.
- Test memeriksa word, byte mask, read-first, write-first, latency 1,
  reset, dan validasi shape.

## 8. Program Counter, IR, dan Stack Pointer

- PC 16-bit: increment setiap tick atau load alamat target.
- IR 16-bit: load instruction word dan menahan nilainya.
- SP 16-bit: load, increment, dan decrement.
- Prioritas SP: load, lalu increment, lalu decrement, lalu hold.
- Reset global mengosongkan PC dan SP.

---

## 9. Digital ALU

- `digital_alu_width` mendukung operand 1..=64 bit dengan opcode 3 bit.
- `digital_alu` adalah wrapper 8 bit untuk kompatibilitas.
- Operasi: ADD, SUB, AND, OR, XOR, dan pass B.
- SUB memakai two's complement.
- Output: result, zero, carry/borrow, negative, half carry/borrow.
- Carry dan half carry dibalik saat SUB agar berarti borrow.

## 9.1 Barrel Shifter

- `amount` memakai `log2(width)` bit untuk width power-of-two 1..=64.
- `barrel_shift_left` dan `barrel_shift_right` mengisi bit kosong
  dengan nol.
- `arithmetic_shift_right` mengisi dengan sign bit.
- `rotate_left` dan `rotate_right` membungkus bit yang keluar.
- `funnel_shift_left` dan `funnel_shift_right` menggeser dua operand
  sebagai satu nilai 2 x width.
- Implementasi struktural memakai tahap mux logaritmik.
- Test menyapu semua nilai amount 0..width-1 terhadap model Rust.

## 9.2 Multiplier

- `multiplier_array` mengalikan dua operand unsigned.
- `multiplier_booth` mengalikan dua operand signed two's complement.
- Lebar operand power-of-two 1..=32; hasil 2 x width bit.
- Array multiplier menjumlahkan partial product yang digeser.
- Booth multiplier memakai recoding radix-2 dan penjumlahan atau
  pengurangan operand tergantung pasangan bit multiplier.
- Test array memeriksa model unsigned dan batas 255 x 255.
- Test Booth memeriksa model signed termasuk nilai negatif dan -128.

## 9.3 Divider

- `divider_restoring` memakai trial subtraction dan restore remainder.
- `divider_non_restoring` menambah atau mengurangi divisor berdasarkan
  tanda remainder, lalu mengoreksi di akhir.
- Dividend dan divisor unsigned dengan width power-of-two 1..=32.
- Hasil berisi `quotient` dan `remainder`, masing-masing width bit.
- Pembagian oleh nol tidak didefinisikan dan tidak dideteksi.
- Test menyapu semua dividend 8-bit untuk divisor batas 1 sampai 255.

## 9.4 Comparator dan Encoder

- `comparator_unsigned` membandingkan dua vektor unsigned dengan lebar
  1..=64 dan menghasilkan `less`, `equal`, dan `greater`.
- `priority_encoder` memilih index terendah yang aktif dan
  mengembalikan `index` serta `valid`.
- `one_hot_encoder` mengubah vektor one-hot menjadi index biner.
- `leading_zero_count` menghitung jumlah nol dari bit paling
  signifikan sampai bit satu pertama.
- Jumlah input encoder harus power-of-two 1..=64.
- Test menyapu seluruh pola 8-bit untuk priority encoder dan
  leading-zero count.

## 9.5 Bit Manipulation

- `bit_reverse` membalik urutan bit untuk width 1..=64.
- `byte_swap` membalik urutan byte untuk width kelipatan 8.
- `popcount` menghitung bit satu untuk width power-of-two 1..=64.
- `saturating_add_unsigned` berhenti di nilai maksimum saat carry.
- `saturating_sub_unsigned` berhenti di nol saat borrow.
- Semua fungsi mengembalikan vektor LSB-first.
- Test memeriksa model Rust untuk reverse, swap, popcount, dan
  saturasi.

## 9.6 Bus Mux dan Crossbar

- `bus_mux` memilih satu dari sekumpulan bus dengan lebar sama.
- Jumlah bus harus power-of-two 1..=64 dan lebar select
  `log2(jumlah bus)`.
- `crossbar` memakai satu select per output sehingga setiap output
  dapat memilih input sendiri.
- Lebar seluruh bus harus sama dan berada di 1..=64.
- Test memilih setiap bus dan menukar rute dua output.

## 9.7 LUT dan FSM

- `lookup_table` menerima 1..=10 input dan truth table `bool` dengan
  panjang `2^n`.
- `finite_state_machine` menerima `FsmSpec` berisi `state_bits`,
  `input_bits`, `next_state`, dan `output`.
- State 1..=8 bit; input 0..=8 bit; total address maksimal 12 bit.
- Next-state table diindeks `{state, input}` dengan input least
  significant.
- Output adalah Moore output yang hanya bergantung pada state.
- Test memeriksa LUT, FSM bebas, FSM dengan input hold/toggle, dan
  validasi tabel.

---

## 10. CPU Uji

CPU kecil memakai FSM dua fase: FETCH dan EXECUTE.

- FETCH: IR memuat instruction word, PC naik.
- EXECUTE: register write, branch, atau halt.
- Zero flag disimpan di D flip-flop saat operasi arithmetic/logic.
- Branch memakai zero flag tersimpan, bukan output ALU saat instruksi branch.

Instruksi:

| Opcode | Instruksi | Format |
|---|---|---|
| 0 | NOP | - |
| 1 | LDI rd, imm6 | rd 5 bit, imm 6 bit |
| 2 | ADD rd, rs | rd 5 bit, rs 5 bit |
| 3 | SUB rd, rs | rd 5 bit, rs 5 bit |
| 4 | AND rd, rs | rd 5 bit, rs 5 bit |
| 5 | OR rd, rs | rd 5 bit, rs 5 bit |
| 6 | XOR rd, rs | rd 5 bit, rs 5 bit |
| 7 | MOV rd, rs | rd 5 bit, rs 5 bit |
| 8 | JMP addr11 | target 11 bit |
| 9 | JZ addr11 | target 11 bit |
| 10 | JNZ addr11 | target 11 bit |
| 31 | HLT | - |

- Register file: 32 x 8 bit.
- Program memory: behavioral `Vec<u16>`, maksimum 2048 word.
- Control PLA: decoder opcode, write enable, flag enable, ALU op, branch, dan halt.
- `CpuTrace` mencatat PC, IR, halt, dan snapshot 32 register.
- Test mencakup arithmetic, branch, loop, JMP, reset, halt, dan trace.

Total test `cells` saat ini: 71.
