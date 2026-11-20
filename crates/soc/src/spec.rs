//! Data-only MCU and SoC specifications.

use kernel::SimError;
use std::collections::BTreeSet;

/// Specification for one MCU or SoC.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McuSpec {
    /// Human-readable name.
    pub name: String,
    /// CPU core configuration.
    pub cpu: CpuSpec,
    /// Clock sources.
    pub clocks: Vec<ClockSpec>,
    /// Bus fabrics.
    pub buses: Vec<BusSpec>,
    /// Memory regions.
    pub memories: Vec<MemorySpec>,
    /// Peripheral instances.
    pub peripherals: Vec<PeripheralSpec>,
    /// Package pins.
    pub pins: Vec<PinSpec>,
}

impl McuSpec {
    /// Validates names, widths, and address collisions.
    pub fn validate(&self) -> Result<(), SimError> {
        if self.name.trim().is_empty() {
            return Err(invalid("MCU name must not be empty"));
        }
        self.cpu.validate()?;
        if self.clocks.is_empty() {
            return Err(invalid("MCU needs at least one clock"));
        }
        if self.memories.is_empty() {
            return Err(invalid("MCU needs at least one memory"));
        }
        validate_clocks(&self.clocks)?;
        validate_buses(&self.buses)?;
        validate_memories(&self.memories)?;
        validate_peripherals(&self.peripherals)?;
        validate_pins(&self.pins)
    }
}

/// CPU core specification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuSpec {
    /// Data width in bits.
    pub data_width: u8,
    /// Address width in bits.
    pub address_width: u8,
    /// Architectural register count.
    pub register_count: u8,
    /// ISA identifier.
    pub isa: String,
}

impl CpuSpec {
    /// Validates CPU widths and ISA name.
    pub fn validate(&self) -> Result<(), SimError> {
        if !matches!(self.data_width, 8 | 16 | 32 | 64) {
            return Err(invalid("CPU data width must be 8, 16, 32, or 64"));
        }
        if self.address_width == 0 || self.address_width > 64 {
            return Err(invalid("CPU address width must be 1..=64"));
        }
        if self.register_count == 0 {
            return Err(invalid("CPU needs at least one register"));
        }
        if self.isa.trim().is_empty() {
            return Err(invalid("CPU ISA must not be empty"));
        }
        Ok(())
    }
}

/// One clock source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockSpec {
    /// Clock name.
    pub name: String,
    /// Frequency in hertz.
    pub frequency_hz: u64,
}

/// Bus protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BusProtocol {
    /// Internal simple bus.
    Internal,
    /// APB.
    Apb,
    /// AHB-Lite.
    AhbLite,
    /// AXI-Lite.
    AxiLite,
    /// Wishbone.
    Wishbone,
    /// Other bus.
    Other,
}

/// One bus fabric.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BusSpec {
    /// Bus name.
    pub name: String,
    /// Bus protocol.
    pub protocol: BusProtocol,
    /// Data width.
    pub data_width: u8,
    /// Address width.
    pub address_width: u8,
}

/// Memory kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryKind {
    /// Volatile RAM.
    Ram,
    /// Mask ROM.
    Rom,
    /// Flash memory.
    Flash,
    /// EEPROM.
    Eeprom,
    /// FRAM.
    Fram,
    /// Other memory.
    Other,
}

/// One memory region.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemorySpec {
    /// Region name.
    pub name: String,
    /// Memory kind.
    pub kind: MemoryKind,
    /// Address bits.
    pub address_bits: u8,
    /// Data bits.
    pub data_bits: u8,
}

/// Peripheral kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeripheralKind {
    /// Timer or counter.
    Timer,
    /// UART or USART.
    Uart,
    /// SPI.
    Spi,
    /// I2C or TWI.
    I2c,
    /// GPIO bank.
    Gpio,
    /// ADC.
    Adc,
    /// DAC.
    Dac,
    /// PWM.
    Pwm,
    /// DMA controller.
    Dma,
    /// Interrupt controller.
    InterruptController,
    /// Other peripheral.
    Other,
}

/// One peripheral instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeripheralSpec {
    /// Instance name.
    pub name: String,
    /// Peripheral kind.
    pub kind: PeripheralKind,
    /// Base address.
    pub base_address: u64,
    /// Optional interrupt line.
    pub irq: Option<u16>,
}

/// One package pin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinSpec {
    /// Pin name.
    pub name: String,
    /// Function or alternate function.
    pub function: String,
}

fn validate_clocks(clocks: &[ClockSpec]) -> Result<(), SimError> {
    for clock in clocks {
        if clock.frequency_hz == 0 {
            return Err(invalid("clock frequency must be positive"));
        }
    }
    ensure_unique(clocks.iter().map(|clock| clock.name.as_str()), "clock")
}

fn validate_buses(buses: &[BusSpec]) -> Result<(), SimError> {
    for bus in buses {
        if bus.data_width == 0 || bus.data_width > 64 {
            return Err(invalid("bus data width must be 1..=64"));
        }
        if bus.address_width == 0 || bus.address_width > 64 {
            return Err(invalid("bus address width must be 1..=64"));
        }
    }
    ensure_unique(buses.iter().map(|bus| bus.name.as_str()), "bus")
}

fn validate_memories(memories: &[MemorySpec]) -> Result<(), SimError> {
    for memory in memories {
        if memory.address_bits == 0 || memory.address_bits > 64 {
            return Err(invalid("memory address width must be 1..=64"));
        }
        if memory.data_bits == 0 || memory.data_bits > 64 {
            return Err(invalid("memory data width must be 1..=64"));
        }
    }
    ensure_unique(memories.iter().map(|memory| memory.name.as_str()), "memory")
}

fn validate_peripherals(peripherals: &[PeripheralSpec]) -> Result<(), SimError> {
    let mut addresses = BTreeSet::new();
    for peripheral in peripherals {
        if !addresses.insert(peripheral.base_address) {
            return Err(invalid("peripheral base addresses must be unique"));
        }
    }
    ensure_unique(
        peripherals
            .iter()
            .map(|peripheral| peripheral.name.as_str()),
        "peripheral",
    )
}

fn validate_pins(pins: &[PinSpec]) -> Result<(), SimError> {
    ensure_unique(pins.iter().map(|pin| pin.name.as_str()), "pin")
}

fn ensure_unique<'a>(names: impl Iterator<Item = &'a str>, label: &str) -> Result<(), SimError> {
    let mut seen = BTreeSet::new();
    for name in names {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(invalid(&format!("{label} name must not be empty")));
        }
        if !seen.insert(trimmed) {
            return Err(invalid(&format!("duplicate {label} name: {trimmed}")));
        }
    }
    Ok(())
}

fn invalid(message: &str) -> SimError {
    SimError::InvalidNetlist(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        BusProtocol, BusSpec, ClockSpec, CpuSpec, McuSpec, MemoryKind, MemorySpec, PeripheralKind,
        PeripheralSpec, PinSpec,
    };

    fn minimal() -> McuSpec {
        McuSpec {
            name: "test-mcu".to_string(),
            cpu: cpu(),
            clocks: vec![clock()],
            buses: vec![bus()],
            memories: vec![memory()],
            peripherals: vec![peripheral()],
            pins: vec![pin()],
        }
    }

    fn cpu() -> CpuSpec {
        CpuSpec {
            data_width: 32,
            address_width: 32,
            register_count: 32,
            isa: "rv32i".to_string(),
        }
    }

    fn clock() -> ClockSpec {
        ClockSpec {
            name: "core".to_string(),
            frequency_hz: 16_000_000,
        }
    }

    fn bus() -> BusSpec {
        BusSpec {
            name: "apb".to_string(),
            protocol: BusProtocol::Apb,
            data_width: 32,
            address_width: 32,
        }
    }

    fn memory() -> MemorySpec {
        MemorySpec {
            name: "sram".to_string(),
            kind: MemoryKind::Ram,
            address_bits: 12,
            data_bits: 32,
        }
    }

    fn peripheral() -> PeripheralSpec {
        PeripheralSpec {
            name: "timer0".to_string(),
            kind: PeripheralKind::Timer,
            base_address: 0x1000,
            irq: Some(1),
        }
    }

    fn pin() -> PinSpec {
        PinSpec {
            name: "PA0".to_string(),
            function: "gpio".to_string(),
        }
    }

    #[test]
    fn minimal_spec_is_valid() {
        assert!(minimal().validate().is_ok());
    }

    #[test]
    fn rejects_invalid_cpu_width() {
        let mut spec = minimal();
        spec.cpu.data_width = 7;
        assert!(spec.validate().is_err());
    }

    #[test]
    fn rejects_empty_clock_list() {
        let mut spec = minimal();
        spec.clocks.clear();
        assert!(spec.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_bus_name() {
        let mut spec = minimal();
        spec.buses.push(spec.buses[0].clone());
        assert!(spec.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_memory_name() {
        let mut spec = minimal();
        spec.memories.push(spec.memories[0].clone());
        assert!(spec.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_peripheral_address() {
        let mut spec = minimal();
        let mut duplicate = spec.peripherals[0].clone();
        duplicate.name = "timer1".to_string();
        spec.peripherals.push(duplicate);
        assert!(spec.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_pin_name() {
        let mut spec = minimal();
        spec.pins.push(spec.pins[0].clone());
        assert!(spec.validate().is_err());
    }
}
