//! Catalog summary and JSON export for the CLI.

use sim_device::{CatalogRecord, catalog_records};

/// Runs `emucad catalog`.
pub fn run(args: &[String]) -> Result<(), String> {
    match args {
        [] => summary(),
        [flag] if flag == "--json" => json(),
        _ => Err("usage: emucad catalog [--json]".to_string()),
    }
}

fn summary() -> Result<(), String> {
    let records = catalog_records();
    let solver = records.iter().filter(|record| record.solver_backed).count();
    println!(
        "catalog: {} entries, {} solver-backed, {} behavioral",
        records.len(),
        solver,
        records.len() - solver
    );
    Ok(())
}

fn json() -> Result<(), String> {
    println!("{}", json_document(&catalog_records()));
    Ok(())
}

fn json_document(records: &[CatalogRecord]) -> String {
    let solver = records.iter().filter(|record| record.solver_backed).count();
    format!(
        "{{\n  \"version\": \"{}\",\n  \"count\": {},\n  \"solver_backed\": {},\n  \"entries\": [\n{}\n  ]\n}}",
        env!("CARGO_PKG_VERSION"),
        records.len(),
        solver,
        entries_json(records)
    )
}

fn entries_json(records: &[CatalogRecord]) -> String {
    records
        .iter()
        .map(record_json)
        .collect::<Vec<_>>()
        .join(",\n")
}

fn record_json(record: &CatalogRecord) -> String {
    format!(
        "    {{\"id\":\"{}\",\"name\":\"{}\",\"family\":\"{}\",\"model_class\":\"{}\",\"device\":{},\"pins\":{},\"parameter\":\"{}\",\"default\":{},\"unit\":\"{}\",\"variant\":{},\"solver_backed\":{}}}",
        escape_json(record.id),
        escape_json(record.name),
        escape_json(record.family),
        escape_json(&record.model_class),
        option_json(record.device.as_deref()),
        record.pins,
        escape_json(record.parameter),
        number_json(record.default),
        escape_json(record.unit),
        record.variant,
        record.solver_backed
    )
}

fn option_json(value: Option<&str>) -> String {
    value.map_or_else(
        || "null".to_string(),
        |text| format!("\"{}\"", escape_json(text)),
    )
}

fn number_json(value: f64) -> String {
    if value.is_finite() {
        return value.to_string();
    }
    "null".to_string()
}

fn escape_json(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if control.is_control() => {
                output.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => output.push(other),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{escape_json, json_document, number_json, option_json, record_json, run};
    use sim_device::{CatalogRecord, catalog_records};

    #[test]
    fn json_contains_all_entries() {
        let document = json_document(&catalog_records());
        assert!(document.contains("\"count\": 500"));
        assert!(document.contains("\"id\":\"passive.resistor.ideal_resistor\""));
        assert!(document.ends_with("\n  ]\n}"));
    }

    #[test]
    fn json_delimiters_are_balanced() {
        let document = json_document(&catalog_records());
        assert_eq!(document.matches('{').count(), document.matches('}').count());
        assert_eq!(document.matches('[').count(), document.matches(']').count());
    }

    #[test]
    fn escape_handles_json_specials() {
        assert_eq!(escape_json("a\"b\\c\n\t"), "a\\\"b\\\\c\\n\\t");
        assert_eq!(escape_json("\u{0001}"), "\\u0001");
    }

    #[test]
    fn missing_device_is_null() {
        let record = catalog_records()
            .into_iter()
            .find(|record| !record.solver_backed)
            .expect("behavioral");
        assert!(record_json(&record).contains("\"device\":null"));
    }

    #[test]
    fn option_and_number_json_handle_edges() {
        assert_eq!(option_json(None), "null");
        assert_eq!(option_json(Some("resistor")), "\"resistor\"");
        assert_eq!(number_json(1000.0), "1000");
        assert_eq!(number_json(f64::NAN), "null");
    }

    #[test]
    fn record_json_keeps_solver_flag() {
        let record = CatalogRecord {
            id: "test",
            name: "test",
            family: "passive",
            model_class: "Resistor".to_string(),
            device: Some("resistor".to_string()),
            pins: 2,
            parameter: "resistance",
            default: 1000.0,
            unit: "ohm",
            variant: 0,
            solver_backed: true,
        };
        let text = record_json(&record);
        assert!(text.contains("\"solver_backed\":true"));
        assert!(text.contains("\"device\":\"resistor\""));
    }

    #[test]
    fn run_rejects_unknown_option() {
        assert!(run(&["--yaml".to_string()]).is_err());
    }
}
