//! `emucad digital` command.

use sim_cells::catalog_digital::{DigitalEntry, digital_entry};

/// Runs `emucad digital`.
pub fn run(args: &[String]) -> Result<(), String> {
    let id = args
        .first()
        .ok_or_else(|| "usage: emucad digital <id> [name=0|1] [--ticks N]".to_string())?;
    let entry = build(id)?;
    let inputs = parse_inputs(&args[1..])?;
    let ticks = parse_ticks(&args[1..])?;
    let outputs = simulate(entry, &inputs, ticks)?;
    println!("{}: {}", id, format_outputs(&outputs));
    Ok(())
}

fn build(id: &str) -> Result<DigitalEntry, String> {
    digital_entry(id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("digital entry is not supported: {id}"))
}

fn simulate(
    mut entry: DigitalEntry,
    inputs: &[(String, bool)],
    ticks: usize,
) -> Result<Vec<(&'static str, bool)>, String> {
    for (name, value) in inputs {
        entry
            .circuit
            .set_input(name, *value)
            .map_err(|error| error.to_string())?;
    }
    if ticks == 0 {
        entry.circuit.settle().map_err(|error| error.to_string())?;
    } else {
        for _ in 0..ticks {
            entry.circuit.tick().map_err(|error| error.to_string())?;
        }
    }
    Ok(entry
        .outputs
        .iter()
        .map(|name| (*name, entry.circuit.output(name).unwrap_or(false)))
        .collect())
}

fn parse_inputs(args: &[String]) -> Result<Vec<(String, bool)>, String> {
    let mut inputs = Vec::new();
    let mut skip = false;
    for arg in args {
        if skip {
            skip = false;
            continue;
        }
        if arg == "--ticks" {
            skip = true;
            continue;
        }
        inputs.push(parse_assignment(arg)?);
    }
    Ok(inputs)
}

fn parse_assignment(text: &str) -> Result<(String, bool), String> {
    let (name, value) = text
        .split_once('=')
        .ok_or_else(|| format!("expected name=0|1: {text}"))?;
    let value = match value {
        "0" | "false" => false,
        "1" | "true" => true,
        _ => return Err(format!("expected 0 or 1: {text}")),
    };
    Ok((name.to_string(), value))
}

fn parse_ticks(args: &[String]) -> Result<usize, String> {
    let position = args.iter().position(|arg| arg == "--ticks");
    let Some(position) = position else {
        return Ok(0);
    };
    args.get(position + 1)
        .ok_or_else(|| "missing value after --ticks".to_string())?
        .parse()
        .map_err(|_| "ticks must be a positive integer".to_string())
}

fn format_outputs(outputs: &[(&str, bool)]) -> String {
    outputs
        .iter()
        .map(|(name, value)| format!("{name}={}", u8::from(*value)))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{format_outputs, parse_assignment, parse_ticks, simulate};
    use sim_cells::catalog_digital::digital_entry;

    #[test]
    fn assignment_parses_binary_values() {
        assert_eq!(
            parse_assignment("sel=1").expect("assign"),
            ("sel".to_string(), true)
        );
        assert_eq!(
            parse_assignment("sel=false").expect("assign"),
            ("sel".to_string(), false)
        );
        assert!(parse_assignment("sel=2").is_err());
    }

    #[test]
    fn ticks_require_a_value() {
        assert_eq!(
            parse_ticks(&["--ticks".to_string(), "3".to_string()]).expect("ticks"),
            3
        );
        assert!(parse_ticks(&["--ticks".to_string()]).is_err());
    }

    #[test]
    fn adder4_simulates_three_plus_five() {
        let entry = digital_entry("logic.adder.4bit_adder")
            .expect("build")
            .expect("entry");
        let inputs = vec![
            ("a0".to_string(), true),
            ("a1".to_string(), true),
            ("b0".to_string(), true),
            ("b2".to_string(), true),
        ];
        let outputs = simulate(entry, &inputs, 0).expect("simulate");
        let text = format_outputs(&outputs);
        assert!(text.contains("s3=1"), "{text}");
        assert!(text.contains("s0=0"), "{text}");
        assert!(text.contains("cout=0"), "{text}");
    }
}
