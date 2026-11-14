"""Smoke test for the EmuCAD Python extension module."""

import emucad


def test_catalog() -> None:
    entries = emucad.catalog()
    assert len(entries) == 500
    assert entries[0]["id"] == "passive.resistor.ideal_resistor"
    assert entries[0]["family"] == "passive"
    assert entries[0]["solver_backed"] is True
    assert emucad.find_component("passive.resistor.ideal_resistor")["name"] == "ideal resistor"
    assert emucad.find_component("missing.part") is None


def test_netlist() -> None:
    result = emucad.build_netlist(
        {
            "nodes": [
                {"name": "vcc", "kind": "supply"},
                {"name": "out"},
            ],
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
    assert [(node["name"], node["kind"]) for node in result["nodes"]] == [
        ("0", "ground"),
        ("vcc", "supply"),
        ("out", "signal"),
    ]
    assert result["devices"][0]["pins"] == ["vcc", "out"]
    assert result["devices"][0]["parameters"]["resistance"] == 1000.0
    assert result["devices"][1]["pins"] == ["out", "0"]


def test_errors() -> None:
    try:
        emucad.build_netlist(
            {"components": [{"name": "x", "part": "missing.part", "pins": []}]}
        )
    except ValueError as error:
        assert "unknown component" in str(error)
    else:
        raise AssertionError("missing component must fail")


if __name__ == "__main__":
    test_catalog()
    test_netlist()
    test_errors()
    print("python smoke tests passed")
