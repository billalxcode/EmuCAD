//! `emucad` command-line entry point.

mod bench;
#[cfg(unix)]
mod bootloader;
mod catalog;
mod digital;
mod monitor;
mod pacing;
#[cfg(unix)]
mod serial;
#[cfg(not(unix))]
mod serial {
    pub fn run(_args: &[String]) -> Result<(), String> {
        Err("serial mode is only supported on Unix".to_string())
    }

    pub fn run_usb(_args: &[String]) -> Result<(), String> {
        Err("USB mode is only supported on Unix".to_string())
    }
}

use monitor::{Monitor, parse_number, parse_word};
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(message) = run(&args) {
        eprintln!("error: {message}");
        eprintln!("{}", usage());
    }
}

fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0].starts_with("--") {
        return serial::run(args);
    }
    let (command, words) = args.split_first().ok_or_else(usage)?;
    dispatch(command, words)
}

fn dispatch(command: &str, words: &[String]) -> Result<(), String> {
    match command {
        "disasm" => show_disassembly(words),
        "run" => run_hex(words),
        "monitor" => {
            let mut monitor = Monitor::new(parse_program(words)?)?;
            repl(&mut monitor)
        }
        "bench" => bench::run(words),
        "catalog" => catalog::run(words),
        "digital" => digital::run(words),
        "serial" => serial::run(words),
        "usb" => serial::run_usb(words),
        _ => Err(usage()),
    }
}

fn run_hex(args: &[String]) -> Result<(), String> {
    let path = args.first().ok_or_else(|| "missing HEX path".to_string())?;
    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let realtime = args.iter().any(|arg| arg == "--realtime");
    let limit = match args.iter().skip(1).find(|arg| !arg.starts_with("--")) {
        Some(raw) => parse_number(raw)?,
        None => 100_000,
    };
    let mut monitor = Monitor::from_hex(&text)?;
    let result = if realtime {
        monitor.run_realtime(limit)?
    } else {
        monitor.run(limit)?
    };
    println!("{result}");
    Ok(())
}

fn show_disassembly(words: &[String]) -> Result<(), String> {
    let monitor = Monitor::new(parse_program(words)?)?;
    println!("{}", monitor.disassemble(0, words.len())?);
    Ok(())
}

fn parse_program(words: &[String]) -> Result<Vec<u16>, String> {
    if words.is_empty() {
        return Err("program is empty".to_string());
    }
    words.iter().map(|word| parse_word(word)).collect()
}

fn repl(monitor: &mut Monitor) -> Result<(), String> {
    let stdin = io::stdin();
    loop {
        print!("avr> ");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut line = String::new();
        if stdin
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            return Ok(());
        }
        if !execute(monitor, line.trim())? {
            return Ok(());
        }
    }
}

fn execute(monitor: &mut Monitor, line: &str) -> Result<bool, String> {
    let mut parts = line.split_whitespace();
    let Some(command) = parts.next() else {
        return Ok(true);
    };
    let args: Vec<&str> = parts.collect();
    match command {
        "quit" | "q" => return Ok(false),
        "step" => println!("{}", monitor.step()?),
        "run" => println!("{}", monitor.run(number(&args, 0, 100)?)?),
        "regs" => println!("{}", monitor.registers()),
        "sreg" => println!("{}", monitor.sreg()),
        "pc" => println!("0x{:04X} cycles={}", monitor.pc(), monitor.cycles()),
        "disasm" => run_disasm(monitor, &args)?,
        "mem" => run_mem(monitor, &args)?,
        "io" => println!("{}", monitor.io(address(&args, 0)? as u8)),
        _ => println!("unknown command: {command}"),
    }
    Ok(true)
}

fn run_disasm(monitor: &Monitor, args: &[&str]) -> Result<(), String> {
    let address = address(args, 0)?;
    let count = number(args, 1, 1)?;
    println!("{}", monitor.disassemble(address, count)?);
    Ok(())
}

fn run_mem(monitor: &Monitor, args: &[&str]) -> Result<(), String> {
    let address = address(args, 0)?;
    let length = number(args, 1, 16)?;
    println!("{}", monitor.memory(address, length));
    Ok(())
}

fn address(args: &[&str], index: usize) -> Result<u16, String> {
    Ok(number(args, index, 0)? as u16)
}

fn number(args: &[&str], index: usize, default: usize) -> Result<usize, String> {
    args.get(index).map_or(Ok(default), |raw| parse_number(raw))
}

fn usage() -> String {
    "usage: emucad [--link path] [--socket path] | emucad monitor <hex words...> | emucad disasm <hex words...> | emucad run <file.hex> [steps] [--realtime] | emucad bench [iterations] [nodes] | emucad catalog [--json] | emucad digital <id> [name=0|1] [--ticks N] | emucad serial [--link path] [--socket path] | emucad usb [--socket path]".to_string()
}
