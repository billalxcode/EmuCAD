"""Build four example netlists with the EmuCAD Python bindings."""

import emucad


def divider() -> dict:
    return emucad.build_netlist(
        {
            "nodes": [{"name": "vcc", "kind": "supply"}, {"name": "out"}],
            "components": [
                {
                    "name": "r1",
                    "part": "passive.resistor.ideal_resistor",
                    "pins": ["vcc", "out"],
                    "parameters": {"resistance": 1000.0},
                },
                {
                    "name": "r2",
                    "part": "passive.resistor.ideal_resistor",
                    "pins": ["out", "0"],
                    "parameters": {"resistance": 2000.0},
                },
            ],
        }
    )


def rc_low_pass() -> dict:
    return emucad.build_netlist(
        {
            "nodes": [{"name": "in"}, {"name": "out"}],
            "components": [
                {
                    "name": "v1",
                    "part": "source.independent.dc_voltage_source",
                    "pins": ["in", "0"],
                    "parameters": {"voltage": 5.0},
                },
                {
                    "name": "r1",
                    "part": "passive.resistor.ideal_resistor",
                    "pins": ["in", "out"],
                    "parameters": {"resistance": 1000.0},
                },
                {
                    "name": "c1",
                    "part": "passive.capacitor.ideal_capacitor",
                    "pins": ["out", "0"],
                    "parameters": {"capacitance": 1e-6},
                },
            ],
        }
    )


def diode_clamp() -> dict:
    return emucad.build_netlist(
        {
            "nodes": [{"name": "in"}, {"name": "out"}],
            "components": [
                {
                    "name": "v1",
                    "part": "source.independent.dc_voltage_source",
                    "pins": ["in", "0"],
                    "parameters": {"voltage": 5.0},
                },
                {
                    "name": "r1",
                    "part": "passive.resistor.ideal_resistor",
                    "pins": ["in", "out"],
                    "parameters": {"resistance": 1000.0},
                },
                {
                    "name": "d1",
                    "part": "diode.rectifier.rectifier_diode",
                    "pins": ["out", "0"],
                },
            ],
        }
    )


def logic_gate() -> dict:
    return emucad.build_netlist(
        {
            "nodes": [
                {"name": "a"},
                {"name": "b"},
                {"name": "vcc", "kind": "supply"},
                {"name": "out"},
            ],
            "components": [
                {
                    "name": "u1",
                    "part": "logic.gate.and_gate",
                    "pins": ["a", "b", "vcc", "0", "out"],
                }
            ],
        }
    )


def main() -> None:
    examples = {
        "divider": divider(),
        "rc_low_pass": rc_low_pass(),
        "diode_clamp": diode_clamp(),
        "logic_gate": logic_gate(),
    }
    for name, netlist in examples.items():
        assert netlist["devices"], name
        print(f"{name}: {len(netlist['nodes'])} nodes, {len(netlist['devices'])} devices")


if __name__ == "__main__":
    main()
