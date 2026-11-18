//! `emucad digital` command.

use cells::catalog_digital::{DigitalConfig, DigitalEntry, digital_entry_with_config};

struct Request {
    inputs: Vec<(String, bool)>,
    ticks: usize,
    config: DigitalConfig,
}

/// Runs `emucad digital`.
pub fn run(args: &[String]) -> Result<(), String> {
    let id = args.first().ok_or_else(|| {
        "usage: emucad digital <id> [name=0|1] [--ticks N] [--pattern V] [--mask V]".to_string()
    })?;
    let request = parse_request(&args[1..])?;
    let entry = build(id, request.config)?;
    let outputs = simulate(entry, &request.inputs, request.ticks)?;
    println!("{}: {}", id, format_outputs(&outputs));
    Ok(())
}

fn build(id: &str, config: DigitalConfig) -> Result<DigitalEntry, String> {
    digital_entry_with_config(id, config)
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

fn parse_request(args: &[String]) -> Result<Request, String> {
    let mut request = Request {
        inputs: Vec::new(),
        ticks: 0,
        config: DigitalConfig::default(),
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--ticks" => {
                request.ticks = value_at(args, index + 1, "ticks")?
                    .parse()
                    .map_err(|_| "ticks must be a positive integer".to_string())?;
                index += 2;
            }
            "--pattern" => {
                request.config.pattern = parse_number(value_at(args, index + 1, "pattern")?)? as u8;
                index += 2;
            }
            "--mask" => {
                request.config.mask = parse_number(value_at(args, index + 1, "mask")?)? as u16;
                index += 2;
            }
            "--fsm" => {
                request.config.fsm = parse_number(value_at(args, index + 1, "fsm")?)? as u16;
                index += 2;
            }
            value if value.starts_with("--") => {
                return Err(format!("unknown option: {value}"));
            }
            value => {
                request.inputs.push(parse_assignment(value)?);
                index += 1;
            }
        }
    }
    Ok(request)
}

fn value_at<'a>(args: &'a [String], index: usize, name: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value after --{name}"))
}

fn parse_number(text: &str) -> Result<u32, String> {
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|_| format!("invalid number: {text}"))
    } else {
        text.parse().map_err(|_| format!("invalid number: {text}"))
    }
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

fn format_outputs(outputs: &[(&str, bool)]) -> String {
    outputs
        .iter()
        .map(|(name, value)| format!("{name}={}", u8::from(*value)))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{format_outputs, parse_assignment, parse_request, simulate};
    use cells::catalog_digital::digital_entry_with_config;

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
    fn request_parses_ticks_and_config() {
        let args = ["--ticks", "3", "--pattern", "0x0f", "--mask", "8", "a0=1"]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        let request = parse_request(&args).expect("request");
        assert_eq!(request.ticks, 3);
        assert_eq!(request.config.pattern, 15);
        assert_eq!(request.config.mask, 8);
        assert_eq!(request.inputs, vec![("a0".to_string(), true)]);
    }

    #[test]
    fn lut_and_mask_simulates() {
        let config = cells::catalog_digital::DigitalConfig {
            pattern: 0,
            mask: 0x0008,
            fsm: 0x0605,
        };
        let entry = digital_entry_with_config("logic.programmable.lookup_table", config)
            .expect("build")
            .expect("entry");
        let inputs = vec![("a0".to_string(), true), ("a1".to_string(), true)];
        let outputs = simulate(entry, &inputs, 0).expect("simulate");
        assert!(format_outputs(&outputs).contains("y=1"));
    }

    #[test]
    fn adder4_simulates_three_plus_five() {
        let entry = digital_entry_with_config(
            "logic.adder.4bit_adder",
            cells::catalog_digital::DigitalConfig::default(),
        )
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
