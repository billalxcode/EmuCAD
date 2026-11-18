# Examples

Contoh runnable berada di masing-masing crate:

```bash
cargo run -p analysis --example rc_analysis
cargo run -p analysis --example inverter_vtc
cargo run -p cells --example ring_oscillator
cargo run -p avr --example cpu_program
cargo run -p board --example blink
cargo run -p avr --example uart_hello
cargo run -p avr --example timer_interrupt
```

`rc_analysis` menunjukkan DC operating point, transient, dan AC pada satu
rangkaian RC.

Contoh firmware ada di `../firmware/`.

Python binding:

```bash
cargo build -p python
bash scripts/test-python.sh
```

`python/build_netlists.py` membangun contoh divider, RC low-pass, diode
clamp, dan logic gate memakai `emucad.build_netlist`.

Proyek GUI:

- `led.emu.json`: resistor dan LED.
- `uart-hello.emu.json`: resistor, LED, dan tombol.

Buka melalui tombol `Open` di `emucad-gui`.

Panduan lengkap ada di `../docs/43-user-guide.md` dan
`../docs/58-gui-user-guide.md`.
