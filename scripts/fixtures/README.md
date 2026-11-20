# Fixture harness ngspice

File di direktori ini adalah input sintetis untuk self-test parser dan
comparator. Isinya bukan hasil pengukuran ngspice dan bukan golden data.

- `ngspice_rc_synthetic.log`: contoh format log `.meas` ngspice.
- `emucad_rc.txt`: contoh keluaran `cargo run -p analysis --example rc_analysis`.

Perbandingan golden dijalankan oleh `scripts/ngspice_golden.sh` dengan
ngspice yang terpasang atau lewat image Docker.
