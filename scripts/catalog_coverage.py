#!/usr/bin/env python3
"""Print deterministic coverage tables for the 500 entry catalog.

The script reads the generated Rust data under
`crates/device/src/catalog_data/` and applies the same family and
fidelity mapping as `crates/device/src/catalog/model/`. It is
read-only and uses only the Python standard library.

Run from the repository root:

    python3 scripts/catalog_coverage.py
"""

import collections
import re
import sys
from pathlib import Path

CATALOG_DIR = Path("crates/device/src/catalog_data")
DIGITAL_FILE = Path("crates/cells/src/catalog_digital.rs")
ENTRY_RE = re.compile(
    r'ComponentSpec \{\s*'
    r'id: "([^"]+)",\s*'
    r'name: "[^"]*",\s*'
    r'class: ModelClass::(\w+),\s*'
    r'device: (?:None|Some\(DeviceKind::(\w+)\)),',
    re.MULTILINE,
)
DIGITAL_RE = re.compile(r'^\s*"([a-z0-9_.]+)"\s*=>', re.MULTILINE)

FAMILY_OF_CLASS = {
    "Resistor": "passive",
    "Capacitor": "passive",
    "Inductor": "passive",
    "Transformer": "passive",
    "Crystal": "passive",
    "TransmissionLine": "passive",
    "VoltageSource": "source",
    "CurrentSource": "source",
    "Vcvs": "source",
    "Vccs": "source",
    "Cccs": "source",
    "Ccvs": "source",
    "BehavioralSource": "source",
    "DigitalSource": "source",
    "Battery": "source",
    "Diode": "diode",
    "DiodeArray": "diode",
    "Led": "diode",
    "Photodiode": "diode",
    "Optocoupler": "diode",
    "Bjt": "transistor",
    "Jfet": "transistor",
    "Mosfet": "transistor",
    "OtherTransistor": "transistor",
    "Thyristor": "protection",
    "Protection": "protection",
    "Switch": "switch",
    "Relay": "switch",
    "SolidStateRelay": "switch",
    "AnalogSwitch": "switch",
    "Connector": "switch",
    "OpAmp": "analog",
    "Comparator": "analog",
    "Timer": "analog",
    "VoltageReference": "analog",
    "LogicGate": "logic",
    "LogicGateSpecial": "logic",
    "FlipFlop": "logic",
    "Counter": "logic",
    "Multiplexer": "logic",
    "Adder": "logic",
    "LevelShifter": "logic",
    "ProgrammableLogic": "logic",
    "Memory": "memory",
    "Mcu": "mcu",
    "Adc": "mcu",
    "Dac": "mcu",
    "SerialInterface": "mcu",
    "TimerPeripheral": "mcu",
    "Regulator": "power",
    "DcDc": "power",
    "ChargePump": "power",
    "Supervisor": "power",
    "PowerSwitch": "power",
    "MotorDriver": "power",
    "Sensor": "sensor",
    "Actuator": "actuator",
    "Display": "actuator",
    "RfBlock": "rf",
    "BehavioralBlock": "behavioral",
    "BoardModule": "board",
}

IDEAL_CLASSES = {
    "Resistor",
    "Capacitor",
    "Inductor",
    "VoltageSource",
    "CurrentSource",
    "Vcvs",
    "Vccs",
    "Cccs",
    "Ccvs",
}

COMPACT_CLASSES = {
    "Diode",
    "DiodeArray",
    "Led",
    "Photodiode",
    "Bjt",
    "Jfet",
    "Mosfet",
    "OtherTransistor",
    "Thyristor",
    "Protection",
    "Switch",
    "Relay",
    "SolidStateRelay",
    "AnalogSwitch",
    "Connector",
    "Crystal",
    "Transformer",
    "TransmissionLine",
    "Battery",
}

FAMILY_ORDER = [
    "passive",
    "source",
    "diode",
    "transistor",
    "protection",
    "switch",
    "analog",
    "logic",
    "memory",
    "mcu",
    "power",
    "sensor",
    "actuator",
    "rf",
    "behavioral",
    "board",
]

FIDELITY_ORDER = ["ideal", "compact", "behavioral", "calibrated", "unsupported"]


def fail(message):
    print("error: " + message, file=sys.stderr)
    raise SystemExit(1)


def read_entry_file(path):
    text = path.read_text(encoding="ascii")
    return [
        {
            "id": match.group(1),
            "model_class": match.group(2),
            "device": match.group(3),
        }
        for match in ENTRY_RE.finditer(text)
    ]


def load_entries(root):
    paths = sorted((root / CATALOG_DIR).rglob("*.rs"))
    files = [path for path in paths if path.name != "mod.rs"]
    if not files:
        fail("catalog data directory is empty")
    entries = []
    for path in files:
        entries.extend(read_entry_file(path))
    return entries


def load_digital_ids(root):
    text = (root / DIGITAL_FILE).read_text(encoding="ascii")
    head = text.split("_ => return Ok(None),", 1)[0]
    return set(DIGITAL_RE.findall(head))


def class_family(model_class):
    family = FAMILY_OF_CLASS.get(model_class)
    if family is None:
        fail("unknown model class: " + model_class)
    return family


def fidelity(entry):
    if entry["device"] is None:
        return "unsupported"
    if entry["model_class"] in IDEAL_CLASSES:
        return "ideal"
    if entry["model_class"] in COMPACT_CLASSES:
        return "compact"
    return "behavioral"


def validate(entries, digital_ids):
    if len(entries) != 500:
        fail("expected 500 entries, found {}".format(len(entries)))
    ids = [entry["id"] for entry in entries]
    if len(set(ids)) != len(ids):
        fail("duplicate catalog IDs")
    for entry in entries:
        expected = entry["id"].split(".", 1)[0]
        actual = class_family(entry["model_class"])
        if actual != expected:
            fail("family mismatch for {}: {}".format(entry["id"], actual))
    unknown = digital_ids - set(ids)
    if unknown:
        fail("digital IDs missing from catalog: {}".format(", ".join(sorted(unknown))))
    overlap = digital_ids & {entry["id"] for entry in entries if entry["device"]}
    if overlap:
        fail("digital IDs also have a catalog device: {}".format(", ".join(sorted(overlap))))


def count_by(entries, key):
    return collections.Counter(key(entry) for entry in entries)


def solver_count(entries):
    return sum(1 for entry in entries if entry["device"] is not None)


def digital_count(entries, digital_ids):
    return sum(1 for entry in entries if entry["id"] in digital_ids)


def no_model_count(entries, digital_ids):
    return sum(
        1 for entry in entries if entry["device"] is None and entry["id"] not in digital_ids
    )


def fidelity_counts(entries):
    return count_by(entries, fidelity)


def family_rows(entries, digital_ids):
    rows = []
    for family in FAMILY_ORDER:
        group = [entry for entry in entries if class_family(entry["model_class"]) == family]
        fidelities = fidelity_counts(group)
        rows.append(
            (
                family,
                len(group),
                solver_count(group),
                digital_count(group, digital_ids),
                no_model_count(group, digital_ids),
                fidelities.get("ideal", 0),
                fidelities.get("compact", 0),
                fidelities.get("behavioral", 0),
                fidelities.get("unsupported", 0),
            )
        )
    return rows


def class_rows(entries, digital_ids):
    rows = []
    classes = sorted({entry["model_class"] for entry in entries})
    for model_class in classes:
        group = [entry for entry in entries if entry["model_class"] == model_class]
        solver = [entry for entry in group if entry["device"] is not None]
        labels = sorted({fidelity(entry) for entry in solver})
        rows.append(
            (
                model_class,
                class_family(model_class),
                len(group),
                len(solver),
                digital_count(group, digital_ids),
                no_model_count(group, digital_ids),
                ", ".join(labels) if labels else "-",
            )
        )
    return rows


def print_summary(entries, digital_ids):
    fidelities = fidelity_counts(entries)
    solver = solver_count(entries)
    digital = digital_count(entries, digital_ids)
    no_model = no_model_count(entries, digital_ids)
    print("## Summary")
    print()
    print("| Metric | Count |")
    print("|---|---:|")
    print("| Catalog entries | {} |".format(len(entries)))
    print("| Model classes | {} |".format(len({entry["model_class"] for entry in entries})))
    print("| Families | {} |".format(len({class_family(entry["model_class"]) for entry in entries})))
    print("| Solver-backed (`device` present) | {} |".format(solver))
    print("| Digital-backed (`catalog_digital`) | {} |".format(digital))
    print("| Entries with at least one model | {} |".format(solver + digital))
    print("| No `device` | {} |".format(len(entries) - solver))
    print("| No model in catalog or `cells` | {} |".format(no_model))
    for label in FIDELITY_ORDER:
        print("| Fidelity `{}` | {} |".format(label, fidelities.get(label, 0)))
    print()


def print_families(entries, digital_ids):
    print("## Families")
    print()
    print("| Family | Entries | Solver | Digital | No model | ideal | compact | behavioral | unsupported |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    for row in family_rows(entries, digital_ids):
        print("| {} | {} | {} | {} | {} | {} | {} | {} | {} |".format(*row))
    print()


def print_classes(entries, digital_ids):
    print("## Model classes")
    print()
    print("| Model class | Family | Entries | Solver | Digital | No model | Solver fidelity |")
    print("|---|---|---:|---:|---:|---:|---|")
    for row in class_rows(entries, digital_ids):
        print("| {} | {} | {} | {} | {} | {} | {} |".format(*row))
    print()


def main():
    root = Path.cwd()
    entries = load_entries(root)
    digital_ids = load_digital_ids(root)
    validate(entries, digital_ids)
    print_summary(entries, digital_ids)
    print_families(entries, digital_ids)
    print_classes(entries, digital_ids)


if __name__ == "__main__":
    main()
