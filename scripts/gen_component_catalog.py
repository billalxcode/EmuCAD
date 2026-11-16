#!/usr/bin/env python3
"""Generate crates/sim-device/src/catalog_data/ from the 500 entry plan.

Run from the repository root:

    python3 scripts/gen_component_catalog.py
"""

import re
import shutil
from pathlib import Path


def slug(text):
    out = []
    for ch in text.lower():
        if ch.isalnum():
            out.append(ch)
        elif ch in " +/":
            out.append("_")
    result = "".join(out)
    while "__" in result:
        result = result.replace("__", "_")
    return result.strip("_")


def norm(name):
    if isinstance(name, tuple):
        return name
    return (name, None)


GATE_CODES = {
    "AND gate": 0,
    "OR gate": 1,
    "NOT gate": 2,
    "NAND gate": 3,
    "NOR gate": 4,
    "XOR gate": 5,
    "XNOR gate": 6,
    "buffer": 7,
    "Schmitt trigger": 8,
}

groups = []


def add(prefix, cls, device, pins, param, default, unit, names):
    groups.append({
        "prefix": prefix,
        "class": cls,
        "device": device,
        "pins": pins,
        "param": param,
        "default": default,
        "unit": unit,
        "names": [norm(n) for n in names],
    })


add("passive.resistor", "Resistor", "Resistor", 2, "resistance", 1000.0, "ohm", [
    "ideal resistor", "temperature coefficient resistor", "potentiometer",
    "digital potentiometer", "NTC thermistor", "PTC thermistor",
    "photoresistor", "varistor", "strain gauge", "memristor", "fuse",
    "ferrite bead",
])
add("passive.capacitor", "Capacitor", "Capacitor", 2, "capacitance", 1e-6, "F", [
    "ideal capacitor", "C0G ceramic capacitor", "X7R ceramic capacitor",
    "electrolytic capacitor", "tantalum capacitor", "film capacitor",
    "variable capacitor", "trimmer capacitor", "supercapacitor",
    "feedthrough capacitor", "capacitor with ESR", "capacitor with ESL",
    "voltage dependent capacitor",
])
add("passive.inductor", "Inductor", "Inductor", 2, "inductance", 1e-3, "H", [
    "ideal inductor", "lossy inductor", "saturable inductor",
    "coupled inductor", "air core inductor", "ferrite core inductor",
    "toroidal inductor", "current transformer", "flyback transformer",
    "common mode choke",
])
add("passive.transformer", "Transformer", "Transformer", 4, "turns_ratio", 1.0, "ratio", [
    "ideal transformer", "center tap transformer", "autotransformer",
    "isolation transformer", "pulse transformer", "current sense transformer",
])
add("passive.crystal", "Crystal", "Crystal", 2, "frequency", 1e6, "Hz", [
    "quartz crystal", "ceramic resonator", "tuning fork crystal", "TCXO",
    "VCXO",
])
add("passive.transmission_line", "TransmissionLine", "TransmissionLine", 4,
    "impedance", 50.0, "ohm", [
        "ideal transmission line", "lossy transmission line", "coaxial line",
        "microstrip line", "stripline", "stub", "attenuator", "power splitter",
        "directional coupler",
    ])

add("source.independent", "VoltageSource", "VoltageSource", 2, "voltage", 5.0, "V", [
    "DC voltage source",
])
add("source.independent", "CurrentSource", "CurrentSource", 2, "current", 1e-3, "A", [
    "DC current source",
])
add("source.independent", "VoltageSource", "VoltageSource", 2, "voltage", 5.0, "V", [
    "AC voltage source",
])
add("source.independent", "CurrentSource", "CurrentSource", 2, "current", 1e-3, "A", [
    "AC current source",
])
add("source.independent", "BehavioralSource", None, 2, "voltage", 5.0, "V", [
    "sine voltage source",
])
add("source.independent", "BehavioralSource", None, 2, "current", 1e-3, "A", [
    "sine current source",
])

add("source.pulse", "VoltageSource", "PulseVoltageSource", 2, "v2", 5.0, "V", [
    "pulse voltage source",
])
add("source.pulse", "BehavioralSource", None, 2, "voltage", 5.0, "V", [
    "PWL voltage source",
])
add("source.pulse", "BehavioralSource", None, 2, "voltage", 5.0, "V", [
    "exponential voltage source",
])
add("source.pulse", "BehavioralSource", None, 2, "voltage", 5.0, "V", [
    "voltage noise source",
])
add("source.pulse", "BehavioralSource", None, 2, "current", 1e-3, "A", [
    "current noise source",
])
add("source.pulse", "BehavioralSource", None, 2, "current", 1e-3, "A", [
    "pulse current source",
])
add("source.controlled", "Vcvs", "Vcvs", 4, "gain", 1.0, "ratio", [
    "VCVS",
])
add("source.controlled", "Vcvs", "Vccs", 4, "gm", 1e-3, "S", [
    "VCCS",
])
add("source.controlled", "Vcvs", "Cccs", 4, "gain", 1.0, "ratio", [
    "CCCS",
])
add("source.controlled", "Vcvs", "Ccvs", 4, "rm", 1.0, "ohm", [
    "CCVS",
])
add("source.behavioral", "BehavioralSource", None, 2, "gain", 1.0, "ratio", [
    "behavioral voltage source", "behavioral current source",
    "polynomial source", "table source",
])
add("source.digital", "DigitalSource", None, 1, "frequency", 1e3, "Hz", [
    "digital clock source", "digital pattern source", "PRBS source",
])
add("source.battery", "Battery", "VoltageSource", 2, "voltage", 1.5, "V", [
    "battery", "solar cell",
])

add("diode.rectifier", "Diode", "Diode", 2, "is", 1e-12, "A", [
    "rectifier diode", "fast rectifier diode", "Schottky diode",
    "silicon carbide Schottky diode", "Zener diode", "avalanche diode",
    "tunnel diode", "PIN diode", "varactor diode", "current limiting diode",
])
add("diode.led", "Led", "Diode", 2, "forward_voltage", 2.0, "V", [
    "red LED", "green LED", "blue LED", "white LED", "IR LED", "UV LED",
    "laser diode", "LED array",
])
add("diode.photo", "Photodiode", "Diode", 2, "responsivity", 0.5, "A/W", [
    "photodiode", "avalanche photodiode",
])
add("diode.opto", "Optocoupler", None, 4, "current_transfer_ratio", 1.0, "ratio", [
    "phototransistor", "photo Darlington", "optocoupler",
    "photovoltaic isolator", "slotted optical switch",
    "reflective optical sensor",
])
add("diode.tvs", "Diode", "Diode", 2, "is", 1e-12, "A", [
    "TVS diode", "ESD protection diode", "bidirectional TVS",
    "snapback ESD device", "tuning varactor", "PIN limiter diode",
])
add("diode.bridge", "DiodeArray", None, 4, "forward_voltage", 0.7, "V", [
    "diode bridge", "dual diode", "diode array", "Schottky bridge",
])
add("diode.other", "Diode", "Diode", 2, "is", 1e-12, "A", [
    "backward diode", "Gunn diode", "IMPATT diode", "step recovery diode",
])

add("transistor.bjt_npn", "Bjt", "Npn", 3, "beta", 100.0, "ratio", [
    "NPN small signal", "NPN power", "NPN RF", "NPN switching",
    "NPN Darlington", "NPN digital",
])
add("transistor.bjt_pnp", "Bjt", "Pnp", 3, "beta", 100.0, "ratio", [
    "PNP small signal", "PNP power", "PNP RF", "PNP switching",
    "PNP Darlington", "PNP digital",
])
add("transistor.bjt_array", "Bjt", "Npn", 3, "beta", 100.0, "ratio", [
    "dual NPN", "dual PNP", "NPN PNP pair", "Darlington array",
    "Sziklai pair", "bias resistor transistor", "transistor array 4x",
    "transistor array 8x",
])
add("transistor.jfet_n", "Jfet", "JfetN", 3, "pinch_off", -2.0, "V", [
    "N JFET", "N JFET switch", "dual N JFET", "N JFET RF",
])
add("transistor.jfet_p", "Jfet", "JfetP", 3, "pinch_off", 2.0, "V", [
    "P JFET", "P JFET switch", "dual P JFET", "P JFET RF",
])
add("transistor.mosfet_n", "Mosfet", "Nmos", 3, "threshold", 2.0, "V", [
    "N MOSFET enhancement", "N MOSFET depletion", "N MOSFET body diode",
    "dual N MOSFET", "four terminal N MOSFET", "N MOSFET with capacitance",
    "N MOSFET temperature dependent", "N MOSFET power",
])
add("transistor.mosfet_p", "Mosfet", "Pmos", 3, "threshold", -2.0, "V", [
    "P MOSFET enhancement", "P MOSFET depletion", "P MOSFET body diode",
    "dual P MOSFET", "four terminal P MOSFET", "P MOSFET with capacitance",
    "P MOSFET temperature dependent",
])
add("transistor.power", "Mosfet", "Nmos", 3, "threshold", 2.0, "V", [
    ("power N MOSFET", "Nmos"), ("power P MOSFET", "Pmos"),
])
add("transistor.power_semiconductor", "OtherTransistor", None, 3, "threshold", 5.0, "V", [
    "IGBT", "IGBT with diode", "GaN HEMT", "SiC MOSFET",
    "SiC Schottky FET", "cascode GaN",
])
add("transistor.other", "OtherTransistor", None, 3, "gain", 10.0, "ratio", [
    "UJT", "PUT", "MESFET", "HEMT",
])

add("protection.thyristor", "Thyristor", None, 3, "holding_current", 0.01, "A", [
    "SCR", "TRIAC", "DIAC", "GTO", "MCT", "SCS", "LASCR", "PUT thyristor",
])
add("protection.device", "Protection", "Fuse", 2, "trip_current", 1.0, "A", [
    "fuse", "resettable PTC fuse", "polyfuse", "thermal fuse",
])
add("protection.device", "Protection", "Varistor", 2, "clamp_voltage", 6.0, "V", [
    "TVS diode", "metal oxide varistor",
])
add("protection.breaker", "Protection", "Varistor", 2, "clamp_voltage", 100.0, "V", [
    "spark gap", "crowbar", "surge arrester",
])
add("protection.breaker", "Protection", "Fuse", 2, "trip_current", 10.0, "A", [
    "circuit breaker",
])

add("switch.mechanical", "Switch", "IdealSwitch", 2, "ron", 0.1, "ohm", [
    "SPST switch", "SPDT switch", "DPST switch", "DPDT switch", "pushbutton",
    "toggle switch", "DIP switch", "reed switch",
])
add("switch.relay", "Relay", "Relay", 5, "rcoil", 100.0, "ohm", [
    "electromechanical relay", "latching relay", "reed relay",
])
add("switch.ssr", "SolidStateRelay", "Switch", 4, "vth", 2.5, "V", [
    "solid state relay", "opto MOSFET relay", "TRIAC output SSR",
    "time delay relay",
])
add("switch.analog", "AnalogSwitch", "Switch", 4, "ron", 1.0, "ohm", [
    "analog switch", "SPDT analog switch", "4 to 1 analog multiplexer",
    "8 to 1 analog multiplexer", "analog demultiplexer",
])
add("switch.connector", "Connector", "Resistor", 2, "resistance", 0.001, "ohm", [
    "header connector", "terminal block", "jumper", "test point", "IC socket",
])

add("analog.opamp", "OpAmp", "OpAmp", 5, "gain", 100000.0, "ratio", [
    "ideal op-amp", "single op-amp", "dual op-amp", "quad op-amp",
    "rail-to-rail op-amp", "precision op-amp", "low noise op-amp",
    "high speed op-amp", "power op-amp", "fully differential op-amp",
    "chopper op-amp", "current feedback op-amp", "instrumentation op-amp",
    "isolation op-amp", "audio op-amp",
])
add("analog.comparator", "Comparator", "Comparator", 5, "gain", 10000.0, "ratio", [
    "comparator", "open collector comparator", "push-pull comparator",
    "window comparator", "hysteresis comparator", "high speed comparator",
])
add("analog.instrumentation", "OpAmp", "OpAmp", 5, "gain", 1.0, "ratio", [
    "instrumentation amplifier", "isolation amplifier",
    "current sense amplifier", "difference amplifier",
    "programmable gain amplifier",
])
add("analog.ota", "OpAmp", "OpAmp", 5, "gain", 1000.0, "ratio", [
    "OTA", "Norton amplifier", "audio power amplifier", "video amplifier",
    "buffer amplifier",
])
add("analog.timer", "Timer", None, 8, "period", 1e-3, "s", [
    "555 timer", "monostable multivibrator", "astable multivibrator",
    "voltage controlled oscillator", "PLL",
])
add("analog.reference", "VoltageReference", "VoltageSource", 2, "voltage", 2.5, "V", [
    "shunt voltage reference", "series voltage reference",
    "bandgap reference", "current reference",
])

add("logic.gate", "LogicGate", "LogicGate", 5, "vth", 2.5, "V", [
    "AND gate", "OR gate", "NOT gate", "NAND gate", "NOR gate", "XOR gate",
    "XNOR gate", "buffer", "Schmitt trigger",
])
add("logic.gate_special", "LogicGateSpecial", None, 4, "propagation_delay", 1e-8, "s", [
    "open drain buffer", "tri-state buffer", "transmission gate",
])
add("logic.flipflop", "FlipFlop", None, 5, "propagation_delay", 1e-8, "s", [
    "SR latch", "D latch", "D flip-flop", "JK flip-flop", "T flip-flop",
    "edge triggered D flip-flop", "master slave JK", "dual D flip-flop",
    "quad D flip-flop", "octal latch",
])
add("logic.counter", "Counter", None, 6, "bits", 4.0, "count", [
    "4-bit counter", "8-bit counter", "decade counter", "up down counter",
    "ripple counter", "Johnson counter", "4-bit shift register",
    "8-bit shift register", "serial in parallel out",
    "parallel in serial out",
])
add("logic.mux", "Multiplexer", None, 6, "channels", 4.0, "count", [
    "2 to 1 multiplexer", "4 to 1 multiplexer", "8 to 1 multiplexer",
    "1 to 2 demultiplexer", "2 to 4 decoder", "3 to 8 decoder",
    "8 to 3 encoder", "priority encoder",
])
add("logic.adder", "Adder", None, 5, "bits", 4.0, "count", [
    "half adder", "full adder", "4-bit adder", "magnitude comparator",
    "parity generator", "parity checker", "bus buffer",
    "bidirectional buffer",
])
add("logic.level", "LevelShifter", None, 4, "voltage", 3.3, "V", [
    "logic level shifter", "bidirectional level shifter",
    "open drain level shifter", "line driver", "line receiver",
    "optocoupler interface",
])
add("logic.programmable", "ProgrammableLogic", None, 8, "macrocells", 16.0, "count", [
    "PAL", "GAL", "CPLD", "FPGA logic block", "finite state machine",
    "look-up table",
])

add("memory.device", "Memory", None, 4, "capacity", 1024.0, "bit", [
    "SRAM cell", "SRAM array", "DRAM cell", "ROM", "PROM", "EEPROM",
    "Flash", "FRAM", "MRAM", "NVRAM", "memory controller", "SPI flash",
    "I2C EEPROM", "SD card", "memory module",
])

add("mcu.core8", "Mcu", None, 8, "clock", 16e6, "Hz", [
    "AVR MCU", "ATmega328P", "PIC MCU", "8051 MCU", "AVR peripheral block",
])
add("mcu.core32", "Mcu", None, 8, "clock", 48e6, "Hz", [
    "ARM Cortex-M", "RISC-V MCU", "ESP32", "STM32", "RP2040",
    "generic 32-bit MCU",
])
add("mcu.converter", "Adc", None, 4, "resolution", 10.0, "bit", [
    "SAR ADC", "delta-sigma ADC", "flash ADC", "R-2R DAC", "PWM generator",
])
add("mcu.serial", "SerialInterface", None, 4, "baud", 9600.0, "baud", [
    "UART", "SPI", "I2C", "CAN", "USB", "Ethernet",
])
add("mcu.timer", "TimerPeripheral", None, 4, "period", 1e-3, "s", [
    "timer", "RTC", "watchdog",
])

add("power.linear", "Regulator", "Regulator", 3, "output_voltage", 5.0, "V", [
    "linear regulator", "LDO", "adjustable linear regulator",
    "negative linear regulator", "current regulator", "shunt regulator",
])
add("power.dcdc", "DcDc", None, 4, "output_voltage", 5.0, "V", [
    "buck converter", "boost converter", "buck-boost converter",
    "Cuk converter", "SEPIC converter", "flyback converter",
    "forward converter", "push-pull converter",
])
add("power.charge", "ChargePump", None, 3, "output_voltage", 5.0, "V", [
    "charge pump", "inverting charge pump", "linear charger",
    "switching charger", "solar charger",
])
add("power.supervisor", "Supervisor", None, 3, "threshold", 2.5, "V", [
    "voltage reference", "current reference", "voltage supervisor",
    "reset supervisor", "power monitor",
])
add("power.switch", "PowerSwitch", "IdealSwitch", 2, "ron", 0.05, "ohm", [
    "ideal diode", "ORing controller", "load switch",
])
add("power.motor", "MotorDriver", None, 4, "current", 1.0, "A", [
    "H-bridge", "half-bridge", "stepper driver",
])

add("sensor.temperature", "Sensor", None, 3, "sensitivity", 0.01, "ratio", [
    "NTC thermistor sensor", "PTC thermistor sensor", "RTD",
    "thermocouple", "semiconductor temperature sensor", "thermal switch",
])
add("sensor.light", "Sensor", None, 3, "sensitivity", 1.0, "ratio", [
    "photoresistor", "photodiode sensor", "phototransistor sensor",
    "ambient light sensor", "color sensor",
])
add("sensor.magnetic", "Sensor", None, 3, "sensitivity", 1.0, "ratio", [
    "Hall sensor", "linear Hall sensor", "reed switch sensor",
    "magnetoresistive sensor", "fluxgate sensor",
])
add("sensor.motion", "Sensor", None, 3, "sensitivity", 1.0, "ratio", [
    "accelerometer", "gyroscope", "IMU", "tilt sensor", "vibration sensor",
])
add("sensor.pressure", "Sensor", None, 3, "sensitivity", 1.0, "ratio", [
    "strain gauge sensor", "load cell", "pressure sensor", "force sensor",
])
add("sensor.humidity", "Sensor", None, 3, "sensitivity", 1.0, "ratio", [
    "humidity sensor", "gas sensor", "pH sensor", "electrochemical sensor",
])
add("sensor.current", "Sensor", None, 3, "sensitivity", 1.0, "ratio", [
    "current sense resistor", "Hall current sensor", "voltage divider sensor",
])
add("sensor.proximity", "Sensor", None, 3, "sensitivity", 1.0, "ratio", [
    "inductive proximity sensor", "capacitive touch sensor",
    "ultrasonic distance sensor",
])

add("actuator.led", "Actuator", None, 2, "gain", 1.0, "ratio", [
    "LED", "RGB LED", "incandescent lamp", "neon lamp",
])
add("actuator.buzzer", "Actuator", None, 2, "gain", 1.0, "ratio", [
    "buzzer", "speaker", "piezo transducer", "piezo buzzer",
])
add("actuator.motor", "Actuator", None, 2, "gain", 1.0, "ratio", [
    "DC motor", "brushless DC motor", "servo motor", "stepper motor",
    "solenoid",
])
add("actuator.display", "Display", None, 4, "resolution", 128.0, "pixel", [
    "LCD character display", "LCD graphic display", "OLED display",
    "e-ink display",
])
add("actuator.heater", "Actuator", None, 2, "gain", 1.0, "ratio", [
    "resistive heater", "Peltier cooler", "thermoelectric generator",
])

add("rf.amplifier", "RfBlock", None, 3, "gain", 10.0, "ratio", [
    "RF amplifier", "low noise amplifier", "power amplifier",
    "variable gain amplifier",
])
add("rf.mixer", "RfBlock", None, 3, "gain", 1.0, "ratio", [
    "RF mixer", "amplitude modulator", "frequency modulator",
    "phase modulator",
])
add("rf.filter", "RfBlock", None, 2, "frequency", 1e6, "Hz", [
    "low pass filter", "high pass filter", "band pass filter",
    "band stop filter",
])
add("rf.antenna", "RfBlock", None, 1, "frequency", 1e6, "Hz", [
    "dipole antenna", "monopole antenna", "loop antenna",
])
add("rf.coupler", "RfBlock", None, 3, "gain", 1.0, "ratio", [
    "directional coupler", "power splitter", "isolator",
])

add("behavioral.filter", "BehavioralBlock", None, 4, "gain", 1.0, "ratio", [
    "integrator", "differentiator", "first order low pass",
    "first order high pass", "state variable filter",
])
add("behavioral.math", "BehavioralBlock", None, 4, "gain", 1.0, "ratio", [
    "multiplier", "divider", "logarithmic amplifier", "antilog amplifier",
    "RMS converter",
])
add("behavioral.detector", "BehavioralBlock", None, 4, "gain", 1.0, "ratio", [
    "behavioral comparator", "Schmitt trigger", "peak detector",
    "track and hold", "precision rectifier",
])
add("behavioral.oscillator", "BehavioralBlock", None, 4, "frequency", 1e3, "Hz", [
    "VCO", "PLL", "amplitude modulator", "frequency modulator",
    "pulse width modulator",
])
add("behavioral.converter", "BehavioralBlock", None, 4, "resolution", 10.0, "bit", [
    "sample and hold", "SAR ADC behavioral", "delta sigma ADC behavioral",
    "R-2R DAC behavioral", "PWM DAC",
])

add("board.arduino", "BoardModule", None, 8, "voltage", 5.0, "V", [
    "Arduino Uno board", "breadboard power board", "generic MCU board",
])
add("board.breadboard", "BoardModule", None, 2, "voltage", 0.0, "V", [
    "breadboard", "jumper wire", "jumper header",
])
add("board.power", "BoardModule", None, 2, "voltage", 5.0, "V", [
    "USB power module", "DC-DC module", "battery holder",
])
add("board.module", "BoardModule", None, 4, "voltage", 5.0, "V", [
    "OLED module", "sensor module", "relay module",
])
add("board.mechanical", "BoardModule", None, 2, "voltage", 0.0, "V", [
    "enclosure", "mounting hardware",
])

entries = []
for group in groups:
    for name, override in group["names"]:
        device = override if override is not None else group["device"]
        device_expr = "None" if device is None else f"Some(DeviceKind::{device})"
        cls = group["class"]
        if cls in ("VoltageSource", "CurrentSource", "Vcvs"):
            cls = {
                "VoltageSource": "VoltageSource",
                "CurrentSource": "CurrentSource",
                "Vcvs": "Vcvs",
                "Vccs": "Vccs",
                "Cccs": "Cccs",
                "Ccvs": "Ccvs",
            }.get(device, cls)
        param = group["param"]
        default = group["default"]
        unit = group["unit"]
        if cls == "CurrentSource":
            param, default, unit = "current", 1e-3, "A"
        elif cls == "TransmissionLine":
            param = "z0"
        elif cls == "Bjt":
            param = "bf"
        elif cls == "Jfet":
            param = "vto"
        elif cls == "Mosfet":
            param = "vto"
        elif cls in ("Led", "Photodiode"):
            param, default, unit = "is", 1e-12, "A"
        default_text = f"{default:.12g}"
        if default_text == "0":
            default_text = "0.0"
        if "." not in default_text and "e" not in default_text:
            default_text += ".0"
        entries.append({
            "id": f"{group['prefix']}.{slug(name)}",
            "name": name,
            "class": cls,
            "device": device_expr,
            "pins": group["pins"],
            "param": param,
            "default": default_text,
            "unit": unit,
            "variant": GATE_CODES.get(name, 0),
        })

if len(entries) != 500:
    print("COUNT", len(entries))
    raise SystemExit(1)

ids = [entry["id"] for entry in entries]
if len(set(ids)) != 500:
    print("DUPLICATE IDS")
    for item in sorted(ids):
        if ids.count(item) > 1:
            print(item)
    raise SystemExit(1)

FAMILIES = [
    "passive", "source", "diode", "transistor", "protection", "switch",
    "analog", "logic", "memory", "mcu", "power", "sensor", "actuator",
    "rf", "behavioral", "board",
]

root = Path("crates/sim-device/src/catalog_data")
if root.exists():
    shutil.rmtree(root)
root.mkdir()

by_family = {family: [] for family in FAMILIES}
for entry in entries:
    family = entry["id"].split(".", 1)[0]
    by_family[family].append(entry)

def entry_lines(entries):
    lines = []
    for entry in entries:
        lines.append(
            '    ComponentSpec {{ id: "{}", name: "{}", class: ModelClass::{}, device: {}, pins: {}, parameter: "{}", default: {}, unit: "{}", variant: {} }},'.format(
                entry["id"], entry["name"], entry["class"], entry["device"],
                entry["pins"], entry["param"], entry["default"], entry["unit"],
                entry["variant"]
            )
        )
    return lines


def class_slug(name):
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


def data_header(family, entries):
    needs_device = any(entry["device"] != "None" for entry in entries)
    lines = [
        f"//! Generated {family} component catalog data.",
        "",
        "use crate::catalog::{ComponentSpec, ModelClass};",
    ]
    if needs_device:
        lines.append("use sim_core::DeviceKind;")
    lines.append("")
    return lines


def write_flat_family(family, entries):
    lines = data_header(family, entries)
    lines.append("pub(crate) const ENTRIES: &[ComponentSpec] = &[")
    lines.extend(entry_lines(entries))
    lines.extend(["];", "", "pub(crate) fn entries() -> &'static [ComponentSpec] {", "    ENTRIES", "}", ""])
    (root / f"{family}.rs").write_text("\n".join(lines))


def write_split_family(family, entries):
    folder = root / family
    folder.mkdir()
    by_class = {}
    for entry in entries:
        by_class.setdefault(entry["class"], []).append(entry)
    names = []
    for class_name, class_entries in by_class.items():
        name = class_slug(class_name)
        names.append(name)
        lines = data_header(f"{family} {name}", class_entries)
        lines.append("pub(crate) const ENTRIES: &[ComponentSpec] = &[")
        lines.extend(entry_lines(class_entries))
        lines.extend(["];", ""])
        (folder / f"{name}.rs").write_text("\n".join(lines))
    lines = [
        f"//! Generated {family} component catalog data.",
        "",
        *[f"mod {name};" for name in names],
        "",
        "use crate::catalog::ComponentSpec;",
        "use std::sync::LazyLock;",
        "",
        "static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {",
        "    let mut entries = Vec::new();",
        *[f"    entries.extend_from_slice({name}::ENTRIES);" for name in names],
        "    entries",
        "});",
        "",
        "pub(crate) fn entries() -> &'static [ComponentSpec] {",
        "    &CATALOG",
        "}",
        "",
    ]
    (folder / "mod.rs").write_text("\n".join(lines))


for family in FAMILIES:
    family_entries = by_family[family]
    if len(family_entries) > 40:
        write_split_family(family, family_entries)
    else:
        write_flat_family(family, family_entries)

module_lines = [
    "//! Generated component catalog aggregator.",
    "",
    *[f"mod {family};" for family in FAMILIES],
    "",
    "use crate::catalog::ComponentSpec;",
    "use std::sync::LazyLock;",
    "",
    "static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {",
    "    let mut entries = Vec::new();",
    *[f"    entries.extend_from_slice({family}::entries());" for family in FAMILIES],
    "    entries",
    "});",
    "",
    "pub(crate) fn entries() -> &'static [ComponentSpec] {",
    "    &CATALOG",
    "}",
    "",
]
Path("crates/sim-device/src/catalog_data/mod.rs").write_text("\n".join(module_lines))

family_counts = {}
for entry in entries:
    family = entry["class"]
    family_counts[family] = family_counts.get(family, 0) + 1
print("entries", len(entries))
print("unique ids", len(set(ids)))
print("classes", len(family_counts))
