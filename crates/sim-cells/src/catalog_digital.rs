//! Named digital circuits for catalog entries.

use crate::{DigitalBuilder, DigitalCircuit, NetId};
use sim_core::SimError;

/// A catalog digital entry with named ports.
pub struct DigitalEntry {
    /// Catalog identifier.
    pub id: &'static str,
    /// Two-state circuit.
    pub circuit: DigitalCircuit,
    /// Input port names.
    pub inputs: Vec<&'static str>,
    /// Output port names.
    pub outputs: Vec<&'static str>,
}

/// Builds a named digital circuit for a catalog entry.
pub fn digital_entry(id: &str) -> Result<Option<DigitalEntry>, SimError> {
    let entry = match id {
        "logic.mux.2_to_1_multiplexer" => mux2_entry(),
        "logic.mux.4_to_1_multiplexer" => mux4_entry(),
        "logic.adder.half_adder" => half_adder_entry(),
        "logic.adder.full_adder" => full_adder_entry(),
        "logic.adder.4bit_adder" => adder4_entry(),
        "logic.flipflop.d_flipflop" => dff_entry(),
        "logic.flipflop.jk_flipflop" => jk_entry(),
        "logic.flipflop.t_flipflop" => t_entry(),
        "logic.flipflop.sr_latch" => sr_entry(),
        "logic.counter.4bit_counter" => counter4_entry(),
        "logic.counter.decade_counter" => decade_entry(),
        "logic.counter.johnson_counter" => johnson_entry(),
        "logic.counter.4bit_shift_register" => shift4_entry(),
        "logic.mux.8_to_1_multiplexer" => mux8_entry(),
        "logic.mux.1_to_2_demultiplexer" => demux_entry(),
        "logic.mux.2_to_4_decoder" => decoder24_entry(),
        "logic.mux.3_to_8_decoder" => decoder38_entry(),
        "logic.mux.8_to_3_encoder" => encoder83_entry(),
        "logic.mux.priority_encoder" => priority_encoder_entry(),
        "logic.adder.magnitude_comparator" => comparator4_entry(),
        "logic.adder.parity_generator" => parity_generator_entry(),
        "logic.adder.parity_checker" => parity_checker_entry(),
        "logic.adder.bus_buffer" => bus_buffer_entry(),
        "logic.counter.8bit_counter" => counter8_entry(),
        "logic.counter.ripple_counter" => ripple_entry(),
        "logic.counter.8bit_shift_register" => shift8_entry(),
        "logic.counter.serial_in_parallel_out" => sipo_entry(),
        "logic.level.logic_level_shifter" => level_entry("logic.level.logic_level_shifter"),
        "logic.level.line_driver" => level_entry("logic.level.line_driver"),
        "logic.level.line_receiver" => level_entry("logic.level.line_receiver"),
        _ => return Ok(None),
    };
    Ok(Some(entry))
}

fn finish(
    id: &'static str,
    build: impl FnOnce(&mut DigitalBuilder) -> (Vec<&'static str>, Vec<&'static str>),
) -> DigitalEntry {
    let mut builder = DigitalBuilder::new();
    let (inputs, outputs) = build(&mut builder);
    DigitalEntry {
        id,
        circuit: builder.build(),
        inputs,
        outputs,
    }
}

fn mux2_entry() -> DigitalEntry {
    finish("logic.mux.2_to_1_multiplexer", |builder| {
        let first = builder.input("in0");
        let second = builder.input("in1");
        let select = builder.input("sel");
        let output = builder.mux(first, second, select);
        builder.output("y", output);
        (vec!["in0", "in1", "sel"], vec!["y"])
    })
}

fn mux4_entry() -> DigitalEntry {
    finish("logic.mux.4_to_1_multiplexer", |builder| {
        let inputs: Vec<NetId> = (0..4)
            .map(|index| builder.input(&format!("in{index}")))
            .collect();
        let select0 = builder.input("sel0");
        let select1 = builder.input("sel1");
        let low = builder.mux(inputs[0], inputs[1], select0);
        let high = builder.mux(inputs[2], inputs[3], select0);
        let output = builder.mux(low, high, select1);
        builder.output("y", output);
        (vec!["in0", "in1", "in2", "in3", "sel0", "sel1"], vec!["y"])
    })
}

fn half_adder_entry() -> DigitalEntry {
    finish("logic.adder.half_adder", |builder| {
        let first = builder.input("a");
        let second = builder.input("b");
        let sum = builder.xor(first, second);
        let carry = builder.and(first, second);
        builder.output("sum", sum);
        builder.output("carry", carry);
        (vec!["a", "b"], vec!["sum", "carry"])
    })
}

fn full_adder_entry() -> DigitalEntry {
    finish("logic.adder.full_adder", |builder| {
        let first = builder.input("a");
        let second = builder.input("b");
        let carry_in = builder.input("cin");
        let (sum, carry) = full_adder_nets(builder, first, second, carry_in);
        builder.output("sum", sum);
        builder.output("carry", carry);
        (vec!["a", "b", "cin"], vec!["sum", "carry"])
    })
}

fn full_adder_nets(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    carry_in: NetId,
) -> (NetId, NetId) {
    let half = builder.xor(first, second);
    let sum = builder.xor(half, carry_in);
    let first_and_second = builder.and(first, second);
    let half_and_carry = builder.and(half, carry_in);
    let carry = builder.or(first_and_second, half_and_carry);
    (sum, carry)
}

fn adder4_entry() -> DigitalEntry {
    finish("logic.adder.4bit_adder", |builder| {
        let mut first = Vec::new();
        let mut second = Vec::new();
        for index in 0..4 {
            first.push(builder.input(&format!("a{index}")));
            second.push(builder.input(&format!("b{index}")));
        }
        let carry_in = builder.input("cin");
        let mut carry = carry_in;
        for index in 0..4 {
            let (sum, next_carry) = full_adder_nets(builder, first[index], second[index], carry);
            builder.output(&format!("s{index}"), sum);
            carry = next_carry;
        }
        builder.output("cout", carry);
        (
            vec!["a0", "a1", "a2", "a3", "b0", "b1", "b2", "b3", "cin"],
            vec!["s0", "s1", "s2", "s3", "cout"],
        )
    })
}

fn dff_entry() -> DigitalEntry {
    finish("logic.flipflop.d_flipflop", |builder| {
        let data = builder.input("d");
        let clock = builder.input("clk");
        let (q, _) = edge_register(builder, data, clock);
        builder.output("q", q);
        let qn = builder.not(q);
        builder.output("qn", qn);
        (vec!["d", "clk"], vec!["q", "qn"])
    })
}

fn edge_register(builder: &mut DigitalBuilder, data: NetId, clock: NetId) -> (NetId, NetId) {
    let q = builder.net();
    let previous_clock = builder.dff(clock);
    let not_previous = builder.not(previous_clock);
    let rising = builder.and(clock, not_previous);
    let next = builder.mux(q, data, rising);
    builder.dff_into(next, q);
    (q, previous_clock)
}

fn jk_entry() -> DigitalEntry {
    finish("logic.flipflop.jk_flipflop", |builder| {
        let j = builder.input("j");
        let k = builder.input("k");
        let clock = builder.input("clk");
        let q = builder.net();
        let previous_clock = builder.dff(clock);
        let not_previous = builder.not(previous_clock);
        let rising = builder.and(clock, not_previous);
        let not_q = builder.not(q);
        let set = builder.and(j, not_q);
        let reset = builder.and(k, q);
        let not_reset = builder.not(reset);
        let next_state = builder.or(set, not_reset);
        let next = builder.mux(q, next_state, rising);
        builder.dff_into(next, q);
        builder.output("q", q);
        builder.output("qn", not_q);
        (vec!["j", "k", "clk"], vec!["q", "qn"])
    })
}

fn t_entry() -> DigitalEntry {
    finish("logic.flipflop.t_flipflop", |builder| {
        let toggle = builder.input("t");
        let clock = builder.input("clk");
        let q = builder.net();
        let previous_clock = builder.dff(clock);
        let not_previous = builder.not(previous_clock);
        let rising = builder.and(clock, not_previous);
        let toggled = builder.xor(q, toggle);
        let next = builder.mux(q, toggled, rising);
        builder.dff_into(next, q);
        let not_q = builder.not(q);
        builder.output("q", q);
        builder.output("qn", not_q);
        (vec!["t", "clk"], vec!["q", "qn"])
    })
}

fn sr_entry() -> DigitalEntry {
    finish("logic.flipflop.sr_latch", |builder| {
        let set = builder.input("s");
        let reset = builder.input("r");
        let q = builder.net();
        let not_q = builder.not(q);
        let not_reset = builder.not(reset);
        let hold = builder.and(not_q, not_reset);
        let next = builder.or(set, hold);
        builder.dff_into(next, q);
        builder.output("q", q);
        builder.output("qn", not_q);
        (vec!["s", "r"], vec!["q", "qn"])
    })
}

fn counter_nets(builder: &mut DigitalBuilder, width: usize) -> (Vec<NetId>, Vec<NetId>) {
    let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    let mut carry = builder.constant(true);
    let mut next = Vec::new();
    let last = bits.len().saturating_sub(1);
    for (index, bit) in bits.iter().enumerate() {
        next.push(builder.xor(*bit, carry));
        if index < last {
            carry = builder.and(carry, *bit);
        }
    }
    (bits, next)
}

fn counter4_entry() -> DigitalEntry {
    finish("logic.counter.4bit_counter", |builder| {
        let reset = builder.input("reset");
        let (bits, next) = counter_nets(builder, 4);
        let zero = builder.constant(false);
        for index in 0..4 {
            let value = builder.mux(next[index], zero, reset);
            builder.dff_into(value, bits[index]);
            builder.output(&format!("q{index}"), bits[index]);
        }
        (vec!["reset"], vec!["q0", "q1", "q2", "q3"])
    })
}

fn decade_entry() -> DigitalEntry {
    finish("logic.counter.decade_counter", |builder| {
        let reset = builder.input("reset");
        let (bits, next) = counter_nets(builder, 4);
        let not_one = builder.not(bits[1]);
        let not_two = builder.not(bits[2]);
        let upper = builder.and(not_one, not_two);
        let middle = builder.and(upper, bits[3]);
        let is_nine = builder.and(bits[0], middle);
        let clear = builder.or(reset, is_nine);
        let zero = builder.constant(false);
        for index in 0..4 {
            let value = builder.mux(next[index], zero, clear);
            builder.dff_into(value, bits[index]);
            builder.output(&format!("q{index}"), bits[index]);
        }
        (vec!["reset"], vec!["q0", "q1", "q2", "q3"])
    })
}

fn johnson_entry() -> DigitalEntry {
    finish("logic.counter.johnson_counter", |builder| {
        let reset = builder.input("reset");
        let bits: Vec<NetId> = (0..4).map(|_| builder.net()).collect();
        let feedback = builder.not(bits[3]);
        for index in 0..4 {
            let source = if index == 0 {
                feedback
            } else {
                bits[index - 1]
            };
            let zero = builder.constant(false);
            let next = builder.mux(source, zero, reset);
            builder.dff_into(next, bits[index]);
            builder.output(&format!("q{index}"), bits[index]);
        }
        (vec!["reset"], vec!["q0", "q1", "q2", "q3"])
    })
}

fn shift4_entry() -> DigitalEntry {
    finish("logic.counter.4bit_shift_register", |builder| {
        let data = builder.input("din");
        let reset = builder.input("reset");
        let bits: Vec<NetId> = (0..4).map(|_| builder.net()).collect();
        let zero = builder.constant(false);
        for index in 0..4 {
            let source = if index == 0 { data } else { bits[index - 1] };
            let next = builder.mux(source, zero, reset);
            builder.dff_into(next, bits[index]);
            builder.output(&format!("q{index}"), bits[index]);
        }
        (vec!["din", "reset"], vec!["q0", "q1", "q2", "q3"])
    })
}

const Q4: [&str; 4] = ["q0", "q1", "q2", "q3"];
const Q8: [&str; 8] = ["q0", "q1", "q2", "q3", "q4", "q5", "q6", "q7"];

fn mux8_entry() -> DigitalEntry {
    finish("logic.mux.8_to_1_multiplexer", |builder| {
        let inputs: Vec<NetId> = (0..8)
            .map(|index| builder.input(&format!("in{index}")))
            .collect();
        let select0 = builder.input("sel0");
        let select1 = builder.input("sel1");
        let select2 = builder.input("sel2");
        let mut level: Vec<NetId> = inputs
            .chunks(2)
            .map(|pair| builder.mux(pair[0], pair[1], select0))
            .collect();
        level = level
            .chunks(2)
            .map(|pair| builder.mux(pair[0], pair[1], select1))
            .collect();
        let output = builder.mux(level[0], level[1], select2);
        builder.output("y", output);
        (
            vec![
                "in0", "in1", "in2", "in3", "in4", "in5", "in6", "in7", "sel0", "sel1", "sel2",
            ],
            vec!["y"],
        )
    })
}

fn demux_entry() -> DigitalEntry {
    finish("logic.mux.1_to_2_demultiplexer", |builder| {
        let input = builder.input("in");
        let select = builder.input("sel");
        let not_select = builder.not(select);
        let first = builder.and(input, not_select);
        let second = builder.and(input, select);
        builder.output("y0", first);
        builder.output("y1", second);
        (vec!["in", "sel"], vec!["y0", "y1"])
    })
}

fn decoder24_entry() -> DigitalEntry {
    finish("logic.mux.2_to_4_decoder", |builder| {
        let a0 = builder.input("a0");
        let a1 = builder.input("a1");
        let not0 = builder.not(a0);
        let not1 = builder.not(a1);
        let outputs = [
            builder.and(not0, not1),
            builder.and(a0, not1),
            builder.and(not0, a1),
            builder.and(a0, a1),
        ];
        for (index, net) in outputs.iter().enumerate() {
            builder.output(&format!("y{index}"), *net);
        }
        (vec!["a0", "a1"], vec!["y0", "y1", "y2", "y3"])
    })
}

fn decoder38_entry() -> DigitalEntry {
    finish("logic.mux.3_to_8_decoder", |builder| {
        let a0 = builder.input("a0");
        let a1 = builder.input("a1");
        let a2 = builder.input("a2");
        let not0 = builder.not(a0);
        let not1 = builder.not(a1);
        let not2 = builder.not(a2);
        for index in 0..8 {
            let first = if index & 1 == 0 { not0 } else { a0 };
            let second = if index & 2 == 0 { not1 } else { a1 };
            let third = if index & 4 == 0 { not2 } else { a2 };
            let low = builder.and(first, second);
            let output = builder.and(low, third);
            builder.output(&format!("y{index}"), output);
        }
        (
            vec!["a0", "a1", "a2"],
            vec!["y0", "y1", "y2", "y3", "y4", "y5", "y6", "y7"],
        )
    })
}

fn encoder83_entry() -> DigitalEntry {
    finish("logic.mux.8_to_3_encoder", |builder| {
        let inputs: Vec<NetId> = (0..8)
            .map(|index| builder.input(&format!("d{index}")))
            .collect();
        let y0 = crate::or_all(builder, &[inputs[1], inputs[3], inputs[5], inputs[7]]);
        let y1 = crate::or_all(builder, &[inputs[2], inputs[3], inputs[6], inputs[7]]);
        let y2 = crate::or_all(builder, &[inputs[4], inputs[5], inputs[6], inputs[7]]);
        let valid = crate::or_all(builder, &inputs);
        builder.output("y0", y0);
        builder.output("y1", y1);
        builder.output("y2", y2);
        builder.output("valid", valid);
        (
            vec!["d0", "d1", "d2", "d3", "d4", "d5", "d6", "d7"],
            vec!["y0", "y1", "y2", "valid"],
        )
    })
}

fn priority_encoder_entry() -> DigitalEntry {
    finish("logic.mux.priority_encoder", |builder| {
        let inputs: Vec<NetId> = (0..8)
            .map(|index| builder.input(&format!("d{index}")))
            .collect();
        let y2 = crate::or_all(builder, &[inputs[4], inputs[5], inputs[6], inputs[7]]);
        let low = crate::or_all(builder, &[inputs[2], inputs[3]]);
        let y1 = builder.or(low, y2);
        let y0 = crate::or_all(builder, &[inputs[1], inputs[3], inputs[5], inputs[7]]);
        let valid = crate::or_all(builder, &inputs);
        builder.output("y0", y0);
        builder.output("y1", y1);
        builder.output("y2", y2);
        builder.output("valid", valid);
        (
            vec!["d0", "d1", "d2", "d3", "d4", "d5", "d6", "d7"],
            vec!["y0", "y1", "y2", "valid"],
        )
    })
}

fn comparator4_entry() -> DigitalEntry {
    finish("logic.adder.magnitude_comparator", |builder| {
        let a: Vec<NetId> = (0..4)
            .map(|index| builder.input(&format!("a{index}")))
            .collect();
        let b: Vec<NetId> = (0..4)
            .map(|index| builder.input(&format!("b{index}")))
            .collect();
        let mut greater = builder.constant(false);
        let mut equal = builder.constant(true);
        for index in (0..4).rev() {
            let not_b = builder.not(b[index]);
            let a_greater = builder.and(a[index], not_b);
            let difference = builder.xor(a[index], b[index]);
            let bit_equal = builder.not(difference);
            let greater_here = builder.and(equal, a_greater);
            greater = builder.or(greater, greater_here);
            equal = builder.and(equal, bit_equal);
        }
        let either = builder.or(greater, equal);
        let less = builder.not(either);
        builder.output("eq", equal);
        builder.output("gt", greater);
        builder.output("lt", less);
        (
            vec!["a0", "a1", "a2", "a3", "b0", "b1", "b2", "b3"],
            vec!["eq", "gt", "lt"],
        )
    })
}

fn parity_generator_entry() -> DigitalEntry {
    finish("logic.adder.parity_generator", |builder| {
        let inputs: Vec<NetId> = (0..4)
            .map(|index| builder.input(&format!("a{index}")))
            .collect();
        let parity = xor_all(builder, &inputs);
        builder.output("parity", parity);
        (vec!["a0", "a1", "a2", "a3"], vec!["parity"])
    })
}

fn parity_checker_entry() -> DigitalEntry {
    finish("logic.adder.parity_checker", |builder| {
        let inputs: Vec<NetId> = (0..4)
            .map(|index| builder.input(&format!("a{index}")))
            .collect();
        let check = builder.input("parity");
        let parity = xor_all(builder, &inputs);
        let error = builder.xor(parity, check);
        builder.output("error", error);
        (vec!["a0", "a1", "a2", "a3", "parity"], vec!["error"])
    })
}

fn xor_all(builder: &mut DigitalBuilder, inputs: &[NetId]) -> NetId {
    inputs[1..]
        .iter()
        .fold(inputs[0], |left, right| builder.xor(left, *right))
}

fn bus_buffer_entry() -> DigitalEntry {
    finish("logic.adder.bus_buffer", |builder| {
        let enable = builder.input("en");
        let inputs: Vec<NetId> = (0..4)
            .map(|index| builder.input(&format!("a{index}")))
            .collect();
        for (index, input) in inputs.iter().enumerate() {
            let output = builder.and(*input, enable);
            builder.output(&format!("y{index}"), output);
        }
        (
            vec!["a0", "a1", "a2", "a3", "en"],
            vec!["y0", "y1", "y2", "y3"],
        )
    })
}

fn counter_entry(id: &'static str, width: usize, names: &'static [&'static str]) -> DigitalEntry {
    finish(id, |builder| {
        let reset = builder.input("reset");
        let (bits, next) = counter_nets(builder, width);
        let zero = builder.constant(false);
        for (index, bit) in bits.iter().enumerate() {
            let value = builder.mux(next[index], zero, reset);
            builder.dff_into(value, *bit);
            builder.output(names[index], *bit);
        }
        (vec!["reset"], names.to_vec())
    })
}

fn counter8_entry() -> DigitalEntry {
    counter_entry("logic.counter.8bit_counter", 8, &Q8)
}

fn ripple_entry() -> DigitalEntry {
    counter_entry("logic.counter.ripple_counter", 4, &Q4)
}

fn shift_entry(id: &'static str, width: usize, names: &'static [&'static str]) -> DigitalEntry {
    finish(id, |builder| {
        let data = builder.input("din");
        let reset = builder.input("reset");
        let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
        let zero = builder.constant(false);
        for (index, bit) in bits.iter().enumerate() {
            let source = if index == 0 { data } else { bits[index - 1] };
            let next = builder.mux(source, zero, reset);
            builder.dff_into(next, *bit);
            builder.output(names[index], *bit);
        }
        (vec!["din", "reset"], names.to_vec())
    })
}

fn shift8_entry() -> DigitalEntry {
    shift_entry("logic.counter.8bit_shift_register", 8, &Q8)
}

fn sipo_entry() -> DigitalEntry {
    shift_entry("logic.counter.serial_in_parallel_out", 8, &Q8)
}

fn level_entry(id: &'static str) -> DigitalEntry {
    finish(id, |builder| {
        let input = builder.input("a");
        let enable = builder.input("en");
        let output = builder.and(input, enable);
        builder.output("y", output);
        (vec!["a", "en"], vec!["y"])
    })
}

#[cfg(test)]
mod tests {
    use super::digital_entry;

    fn entry(id: &str) -> super::DigitalEntry {
        digital_entry(id).expect("build").expect("entry")
    }

    #[test]
    fn mux2_selects_input() {
        let mut entry = entry("logic.mux.2_to_1_multiplexer");
        entry.circuit.set_input("in1", true).expect("input");
        entry.circuit.set_input("sel", true).expect("input");
        entry.circuit.settle().expect("settle");
        assert_eq!(entry.circuit.output("y"), Some(true));
    }

    #[test]
    fn adder4_adds_three_and_five() {
        let mut entry = entry("logic.adder.4bit_adder");
        for label in ["a0", "a1"] {
            entry.circuit.set_input(label, true).expect("input");
        }
        for label in ["b0", "b2"] {
            entry.circuit.set_input(label, true).expect("input");
        }
        entry.circuit.settle().expect("settle");
        assert_eq!(entry.circuit.output("s3"), Some(true));
        assert_eq!(entry.circuit.output("s0"), Some(false));
        assert_eq!(entry.circuit.output("cout"), Some(false));
    }

    #[test]
    fn dff_captures_on_rising_clock() {
        let mut entry = entry("logic.flipflop.d_flipflop");
        entry.circuit.set_input("d", true).expect("input");
        entry.circuit.tick().expect("tick");
        assert_eq!(entry.circuit.output("q"), Some(false));
        entry.circuit.set_input("clk", true).expect("input");
        entry.circuit.tick().expect("tick");
        assert_eq!(entry.circuit.output("q"), Some(true));
    }

    #[test]
    fn counter_counts_four_bits() {
        let mut entry = entry("logic.counter.4bit_counter");
        for _ in 0..3 {
            entry.circuit.tick().expect("tick");
        }
        assert_eq!(entry.circuit.output("q0"), Some(true));
        assert_eq!(entry.circuit.output("q1"), Some(true));
        assert_eq!(entry.circuit.output("q2"), Some(false));
    }

    #[test]
    fn decade_counter_resets_after_nine() {
        let mut entry = entry("logic.counter.decade_counter");
        for _ in 0..10 {
            entry.circuit.tick().expect("tick");
        }
        for index in 0..4 {
            assert_eq!(entry.circuit.output(&format!("q{index}")), Some(false));
        }
    }

    #[test]
    fn mux8_selects_index_five() {
        let mut entry = entry("logic.mux.8_to_1_multiplexer");
        entry.circuit.set_input("in5", true).expect("input");
        for label in ["sel0", "sel2"] {
            entry.circuit.set_input(label, true).expect("input");
        }
        entry.circuit.settle().expect("settle");
        assert_eq!(entry.circuit.output("y"), Some(true));
    }

    #[test]
    fn decoder38_selects_index_five() {
        let mut entry = entry("logic.mux.3_to_8_decoder");
        for label in ["a0", "a2"] {
            entry.circuit.set_input(label, true).expect("input");
        }
        entry.circuit.settle().expect("settle");
        assert_eq!(entry.circuit.output("y5"), Some(true));
        assert_eq!(entry.circuit.output("y4"), Some(false));
    }

    #[test]
    fn priority_encoder_reports_highest_bit() {
        let mut entry = entry("logic.mux.priority_encoder");
        entry.circuit.set_input("d6", true).expect("input");
        entry.circuit.settle().expect("settle");
        assert_eq!(entry.circuit.output("y1"), Some(true));
        assert_eq!(entry.circuit.output("y2"), Some(true));
        assert_eq!(entry.circuit.output("y0"), Some(false));
        assert_eq!(entry.circuit.output("valid"), Some(true));
    }

    #[test]
    fn comparator_detects_greater_and_less() {
        let mut entry = entry("logic.adder.magnitude_comparator");
        for label in ["a0", "a2"] {
            entry.circuit.set_input(label, true).expect("input");
        }
        entry.circuit.set_input("b1", true).expect("input");
        entry.circuit.settle().expect("settle");
        assert_eq!(entry.circuit.output("gt"), Some(true));
        assert_eq!(entry.circuit.output("lt"), Some(false));
    }

    #[test]
    fn parity_checker_detects_error() {
        let mut entry = entry("logic.adder.parity_checker");
        for label in ["a0", "a1", "a2"] {
            entry.circuit.set_input(label, true).expect("input");
        }
        entry.circuit.set_input("parity", false).expect("input");
        entry.circuit.settle().expect("settle");
        assert_eq!(entry.circuit.output("error"), Some(true));
    }

    #[test]
    fn bus_buffer_is_gated_by_enable() {
        let mut entry = entry("logic.adder.bus_buffer");
        entry.circuit.set_input("a0", true).expect("input");
        entry.circuit.set_input("a2", true).expect("input");
        entry.circuit.set_input("en", true).expect("input");
        entry.circuit.settle().expect("settle");
        assert_eq!(entry.circuit.output("y0"), Some(true));
        assert_eq!(entry.circuit.output("y2"), Some(true));
    }

    #[test]
    fn shift_register_moves_data() {
        let mut entry = entry("logic.counter.4bit_shift_register");
        entry.circuit.set_input("din", true).expect("input");
        entry.circuit.tick().expect("tick");
        entry.circuit.set_input("din", false).expect("input");
        entry.circuit.tick().expect("tick");
        assert_eq!(entry.circuit.output("q0"), Some(false));
        assert_eq!(entry.circuit.output("q1"), Some(true));
    }
}
