# 12 - Device Models

Katalog device yang tersedia di `sim-device` dan parameter utamanya.

**Status:** seluruh device inti, device nonlinier, controlled source, switch,
dan transmission line sudah diimplementasikan. Total test `sim-device`: 183.

---

## 1a. Katalog generik 500

`sim-device` juga menyediakan katalog 500 model generik melalui
`catalog()`, `ComponentSpec`, `Family`, dan `ModelClass`. Katalog ini
menjadi registry untuk Python builder dan pemetaan ke solver.

Detail taksonomi, akurasi, dan status ada di
`docs/60-component-catalog.md` dan `TODO-COMPONENTS.md`.

Registry:

```rust
let registry = ComponentRegistry::new();
let request = ComponentRequest::new(
    "passive.resistor.ideal_resistor",
    "R1",
    DeviceId::new(0),
    vec![a, b],
)
.with_parameter("resistance", 1000.0);
registry.add_to(&mut builder, &request)?;
```

Registry memvalidasi pin, parameter default, dan pemetaan ke
`DeviceKind`. Test emulasi membangun voltage divider dan saklar ideal
melalui registry, lalu menjalankan analisis DC.

---

## 1. API model

```rust
pub trait DeviceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError>;

    fn update(&mut self, solution: &Solution) -> Result<(), SimError>;
}
```

- `dt = None` untuk DC dan AC bias.
- `dt = Some(value)` untuk transient.
- `update` menyimpan state device setelah solusi konvergen.

Factory:

```rust
pub fn model_from_instance(
    instance: &DeviceInstance,
) -> Result<Box<dyn DeviceModel>, SimError>
```

---

## 2. Katalog device

| Device | Pin | Model | Dokumen |
|---|---|---|---|
| Resistor | 2 | Ohm, koefisien suhu | [33](33-temperature.md) |
| Capacitor | 2 | Linear | - |
| Inductor | 2 | Linear, branch current | [28](28-integration.md) |
| Voltage source | 2 | DC dan AC | - |
| Current source | 2 | DC dan AC | - |
| Pulse source | 2 | Rise, fall, width, period | [41](41-pulse-source.md) |
| Diode | 2 | Shockley, series resistance, junction capacitance | [46](46-diode-nonideal.md) |
| MOSFET | 3 atau 4 | Shichman-Hodges Level 1, body effect, Meyer | [44](44-body-effect.md), [45](45-mosfet-capacitance.md) |
| BJT | 3 | Ebers-Moll, junction dan diffusion capacitance | [35](35-bjt.md) |
| JFET | 3 | Shichman-Hodges, kapasitansi konstan | [36](36-jfet.md) |
| Transmission line | 4 | Lossless, constant step | [37](37-transmission-line.md) |
| Varistor | 2 | Nonlinear clamp | - |
| Fuse | 2 | Overcurrent heat integration | - |
| Transformer | 4 | Mutual inductance, winding resistance | - |
| Crystal | 2 | BVD series RLC dan C0 | - |
| IGBT | 3 | Behavioral channel dan body diode | - |
| UJT | 3 | Latching unijunction | - |
| PUT | 3 | Programmable unijunction | - |
| SCR | 3 | Latching thyristor dengan gate | - |
| TRIAC | 3 | Latching thyristor dua arah | - |
| DIAC | 2 | Breakover switch dua arah | - |
| GTO | 3 | Thyristor dengan gate turn-off | - |
| SCS | 4 | Latching switch dengan dua gate | - |
| Diode bridge | 4 | Empat diode bridge | - |
| Dual diode | 4 | Dua diode independen | - |
| Optocoupler | 4 | LED dan output transistor dengan CTR | - |
| NTC | 3 | Beta equation dan divider | - |
| PTC | 3 | Linear positive coefficient | - |
| RTD | 3 | Callendar-Van Dusen dan divider | - |
| Thermocouple | 3 | Seebeck sensitivity dan cold junction | - |
| Temperature sensor | 3 | Linear V/C output | - |
| Thermal switch | 3 | Threshold dan hysteresis | - |
| Photoresistor | 3 | Gamma resistance dan divider | - |
| Photodiode sensor | 3 | Transimpedance photocurrent | - |
| Linear light sensor | 3 | Linear V/lux output | - |
| Hall switch | 3 | Threshold dan hysteresis | - |
| Linear Hall sensor | 3 | Offset dan V/T output | - |
| Reed switch sensor | 3 | Threshold dan hysteresis | - |
| Magnetoresistive sensor | 3 | Field-dependent divider | - |
| Fluxgate sensor | 3 | Linear V/T output | - |
| Accelerometer | 3 | Mid-supply linear V/(m/s2) | - |
| Gyroscope | 3 | Mid-supply linear V/(deg/s) | - |
| IMU | 3 | Accelerometer satu sumbu | - |
| Tilt sensor | 3 | Linear V/deg | - |
| Vibration sensor | 3 | Linear vibration amplitude | - |
| Strain gauge sensor | 3 | Bridge gauge factor equation | - |
| Load cell | 3 | Rated output mV/V | - |
| Pressure sensor | 3 | Linear V/Pa output | - |
| Force sensor | 3 | Linear V/N output | - |
| Humidity sensor | 3 | Linear V/%RH output | - |
| Gas sensor | 3 | Chemiresistor divider | - |
| pH sensor | 3 | Nernst slope V/pH | - |
| Electrochemical sensor | 3 | Linear V/ppm output | - |
| Current sense resistor | 3 | Shunt dan gain | - |
| Hall current sensor | 3 | Linear V/A output | - |
| Voltage divider sensor | 3 | Linear V/V output | - |
| Inductive proximity sensor | 3 | Threshold switch pada jarak | - |
| Capacitive touch sensor | 3 | Threshold switch pada kapasitansi | - |
| Ultrasonic distance sensor | 3 | Linear V/m output | - |
| LED actuator | 2 | Exponential junction dan series resistance | - |
| DC motor | 2 | Winding resistance dan back EMF | - |
| Peltier | 2 | Resistance dan Seebeck back EMF | - |
| Thermoelectric generator | 2 | Thevenin Seebeck source | - |
| Display module | 4 | Supply load dan input leakage | - |
| RF amplifier | 3 | VCVS dengan input resistance | [74](74-rf-amplifiers.md) |
| DC-DC converter | 4 | Output teregulasi dan input current | [75](75-converters.md) |
| Charge pump | 3 | Output teregulasi dan input current | [75](75-converters.md) |
| Voltage reference | 3 | Fixed output dan quiescent current | [76](76-supervisors.md) |
| Current reference | 3 | Fixed output current | [76](76-supervisors.md) |
| Supervisor | 3 | Threshold, hysteresis, dan polaritas | [76](76-supervisors.md) |
| Motor driver | 4 | Switch high-side satu kanal | [77](77-motor-drivers.md) |
| Sine voltage source | 2 | Sine dengan amplitude dan phase | [78](78-sine-sources.md) |
| Sine current source | 2 | Sine dengan amplitude dan phase | [78](78-sine-sources.md) |
| Timer 555 dan astable | 8 | Square wave period dan duty | [79](79-timers.md) |
| Monostable timer | 8 | Trigger edge dan pulse width | [79](79-timers.md) |
| VCO timer | 8 | Frequency dari control voltage | [79](79-timers.md) |
| Pulse current source | 2 | Pulse current | [80](80-waveform-sources.md) |
| Exponential voltage source | 2 | Dua segmen eksponensial | [80](80-waveform-sources.md) |
| PWL voltage source | 2 | Empat titik interpolasi | [80](80-waveform-sources.md) |
| Noise source | 2 | PRNG deterministik | [80](80-waveform-sources.md) |
| VCVS | 4 | Linear gain | [47](47-controlled-sources.md) |
| VCCS | 4 | Linear transconductance | [47](47-controlled-sources.md) |
| CCCS | 4 | Linear current gain | [48](48-cccs-switch.md) |
| CCVS | 4 | Linear transresistance | [49](49-ccvs.md) |
| Switch | 4 | Smooth on/off conductance | [48](48-cccs-switch.md) |
| AnalogSwitch | 4 | Voltage-controlled analog conductance | - |
| Relay | 5 | Coil and changeover contacts | - |
| Regulator | 3 | Ideal behavioral output voltage | - |
| LogicGate | 5 | Behavioral gate with rail saturation | - |
| IdealSwitch | 2 | Fixed on/off conductance | - |
| Connector | 2 | Ideal wire sebagai resistor 1 milliohm | - |
| SolidStateRelay | 4 | Switch terkontrol tegangan dengan pin terisolasi | - |
| OpAmp | 5 | Linear high-gain VCVS macromodel | - |
| Comparator | 5 | Saturating nonlinear comparator | - |

---

## 3. Parameter passif dan sumber

| Device | Parameter | Default | Wajib |
|---|---|---|---|
| Resistor | `resistance` | - | ya |
| Resistor | `temp` | 300.15 K | tidak |
| Resistor | `tc1` | 0 | tidak |
| Resistor | `tc2` | 0 | tidak |
| Capacitor | `capacitance` | - | ya |
| Inductor | `inductance` | - | ya |
| Voltage source | `voltage` | 0.0 | tidak |
| Voltage source | `ac` | `voltage` | tidak |
| Voltage source | `ac_phase` | 0 deg | tidak |
| Current source | `current` | 0.0 | tidak |
| Current source | `ac` | `current` | tidak |
| Current source | `ac_phase` | 0 deg | tidak |

Sumber pulsa:

| Parameter | Default |
|---|---|
| `v1` | 0.0 V |
| `v2` | 5.0 V |
| `delay` | 0 s |
| `rise` | 1 ns |
| `fall` | 1 ns |
| `width` | 1 us |
| `period` | 2 us |

---

## 4. Diode

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `is` | 1e-12 A | saturation current |
| `n` | 1.0 | emission coefficient |
| `vt` | k*T/q | thermal voltage |
| `rs` | 0 ohm | series resistance |
| `cj0` | 0 F | zero-bias junction capacitance |
| `vj` | 0.7 V | junction potential |
| `m` | 0.5 | grading coefficient |

Arus:

```text
I = is * (exp(V / (n * vt)) - 1)
```

Junction capacitance:

```text
C = cj0 / (1 - V / vj)^m
```

Dengan `rs > 0`, tegangan junction diselesaikan secara iteratif.

---

## 5. MOSFET

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `vto` | 0.7 V | threshold |
| `kp` | 1e-3 A/V^2 | transconductance |
| `lambda` | 0.0 | channel-length modulation |
| `w` | 1e-6 m | width |
| `l` | 1e-6 m | length |
| `gamma` | 0.0 | body effect |
| `phi` | 0.7 V | surface potential |
| `cox` | 0 F | oxide capacitance |
| `cgso` | 0 F | gate-source overlap |
| `cgdo` | 0 F | gate-drain overlap |

Region:

- Cutoff: `vgs <= vto`.
- Linear: `vds < vgs - vto`.
- Saturation: `vds >= vgs - vto`.

Kapasitansi memakai pembagian Meyer sederhana berdasarkan region.
MOSFET tiga terminal memakai source sebagai body.

---

## 6. BJT

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `is` | 1e-15 A | saturation current |
| `bf` | 100 | forward beta |
| `br` | 1 | reverse beta |
| `vt` | k*T/q | thermal voltage |
| `temp` | 300.15 K | suhu |
| `cje` | 0 F | base-emitter capacitance |
| `vje` | 0.75 V | base-emitter potential |
| `mje` | 0.33 | base-emitter grading |
| `cjc` | 0 F | base-collector capacitance |
| `vjc` | 0.75 V | base-collector potential |
| `mjc` | 0.33 | base-collector grading |
| `tf` | 0 s | forward transit time |
| `tr` | 0 s | reverse transit time |

Arus Ebers-Moll:

```text
IF = is * (exp(vbe / vt) - 1)
IR = is * (exp(vbc / vt) - 1)
Ic = alpha_f * IF - IR
Ib = (1 - alpha_f) * IF + (1 - alpha_r) * IR
Ie = -IF + alpha_r * IR
```

Kapasitansi:

```text
Cbe = depletion(cje, vje, mje, vbe) + tf * gbe
Cbc = depletion(cjc, vjc, mjc, vbc) + tr * gbc
```

PNP memakai transformasi tanda pada `vbe`, `vbc`, dan arus.

---

## 7. JFET

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `vto` | -2 V untuk N, +2 V untuk P | threshold |
| `beta` | 1e-3 A/V^2 | transconductance |
| `lambda` | 0.0 | channel-length modulation |
| `cgs` | 0 F | gate-source capacitance |
| `cgd` | 0 F | gate-drain capacitance |

Arus channel:

```text
overdrive = vgs - vto
overdrive <= 0: Id = 0
vds < overdrive: Id = beta * (2 * overdrive * vds - vds^2) * (1 + lambda * vds)
vds >= overdrive: Id = beta * overdrive^2 * (1 + lambda * vds)
```

P-JFET memakai transformasi tanda. Kapasitansi `cgs` dan `cgd` konstan.

---

## 8. Controlled source dan switch

| Device | Parameter | Default |
|---|---|---|
| VCVS | `gain` | 1.0 |
| VCCS | `gm` | 1e-3 S |
| CCCS | `gain` | 1.0 |
| CCVS | `rm` | 1000 ohm |
| Switch | `ron` | 1 ohm |
| Switch | `roff` | 1e9 ohm |
| Switch | `vth` | 2.5 V |
| Switch | `width` | 0.1 V |
| IdealSwitch | `ron` | 0.01 ohm |
| IdealSwitch | `roff` | 1e12 ohm |
| IdealSwitch | `closed` | 0 |
| Connector | `resistance` | 0.001 ohm | ya |
| SolidStateRelay | `vth` | 2.5 V | tidak |
| AnalogSwitch | `ron` | 1 ohm |
| Relay | `rcoil` | 100 ohm |
| Relay | `ron` | 0.1 ohm |
| Regulator | `output_voltage` | 5 V |
| LogicGate | `vth` | 2.5 V |
| LogicGate | `variant` | 0 sampai 8 |
| OpAmp | `gain` | 1e5 |
| Comparator | `gain` | 1e4 |

CCVS memakai dua branch: sense branch dan output branch.

SolidStateRelay memakai model Switch dengan urutan pin `output+, output-,
control+, control-`. Rating isolasi tidak disimulasikan; input dan output
hanya terpisah secara elektrik melalui model switch. Connector dimodelkan
sebagai resistor 1 milliohm, bukan model mekanis.

Switch memakai conductance halus:

```text
g = 1/roff + (1/ron - 1/roff) * sigmoid((vctrl - vth) / width)
```

---

## 9. Transmission line

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `z0` | 50 ohm | characteristic impedance |
| `delay` | 1 ns | one-way delay |

Transmission line hanya mendukung transient dengan time step konstan.
Device ini belum didukung AC.

---

## 10. Test

Test device mencakup:

- Validasi parameter.
- Stamping resistor, capacitor, dan inductor.
- Sumber tegangan dan arus.
- MOSFET cutoff, linear, saturation, lambda, width/length.
- Body effect dan `gmb`.
- Kapasitansi MOSFET.
- Diode IV, series resistance, dan junction capacitance.
- BJT arus, gain, suhu, dan kapasitansi.
- JFET arus, tanda P, dan kapasitansi.
- Controlled source dan switch.
- Transmission line.
- Transformer mutual inductance.
- Crystal BVD.
- IGBT channel dan body diode.
- UJT dan PUT latching.
- SCR, TRIAC, DIAC, GTO, dan SCS latching.
- Diode bridge dan dual diode.
- Optocoupler CTR.
- Sensor suhu NTC, PTC, RTD, thermocouple, semikonduktor, dan switch.
- Sensor cahaya photoresistor, photodiode, dan linear light.
- Sensor magnetik Hall switch, Hall linear, reed switch, magnetoresistive,
  dan fluxgate.
- Sensor gerak accelerometer, gyroscope, IMU, tilt, dan vibration.
- Sensor tekanan strain gauge, load cell, pressure, dan force.
- Sensor kelembapan dan kimia humidity, gas, pH, dan electrochemical.
- Sensor arus current sense resistor, Hall current, dan voltage divider.
- Sensor proximity inductive, capacitive touch, dan ultrasonic.
- Actuator LED, lamp, buzzer, motor, heater, Peltier, TEG, dan display.
- RF amplifier VCVS tiga pin.
- Konverter DC-DC empat pin dan charge pump tiga pin.
- Reference dan supervisor tiga pin.
- Motor driver empat pin dengan switch satu kanal.
- Sine voltage dan current source.
- Timer 555, astable, monostable, dan VCO.
- Pulse current, exponential, PWL, dan noise source.

Total test `sim-device`: 178 unit + 5 integration = 183.

---

## 11. Batas

- MOSFET Level 1 tidak akurat untuk short-channel.
- Meyer capacitance bukan charge-conserving.
- BJT dan JFET memakai model kapasitansi sederhana.
- Diode belum memiliki breakdown dan diffusion capacitance.
- Switch belum memiliki hysteresis atau delay.
- Transmission line hanya lossless dan constant step.
- Parameter belum dikalibrasi ke silikon.

Matriks fidelity lengkap ada di [24-fidelity.md](24-fidelity.md).
