//! ATmega328P package pin mapping.
//!
//! The table follows the 32-pin TQFP pinout from the Microchip
//! ATmega328P datasheet, section 5.3. Pin 29 is PC6/RESET and is kept
//! as an IO location; reset is an alternate function of that pin.

use kernel::SimError;

/// Signal carried by one package pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvrSignal {
    /// General-purpose IO location.
    Io {
        /// Port index: 0 is B, 1 is C, 2 is D.
        port: u8,
        /// Bit within the port.
        bit: u8,
    },
    /// Digital supply.
    Vcc,
    /// Ground.
    Gnd,
    /// Analog supply.
    Avcc,
    /// Analog reference.
    Aref,
    /// Analog input 6.
    Adc6,
    /// Analog input 7.
    Adc7,
}

/// Map from package pin number to signal.
#[derive(Clone, Copy, Default)]
pub struct AvrPackageMap {
    signals: [Option<AvrSignal>; 32],
}

impl AvrPackageMap {
    /// Builds an empty map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one package pin in the range 1..=32.
    pub fn add(&mut self, pin: u8, signal: AvrSignal) -> Result<(), SimError> {
        let index = pin_index(pin)?;
        if self.signals[index].is_some() {
            return Err(SimError::InvalidNetlist(format!(
                "ATmega328P package pin {pin} already mapped"
            )));
        }
        self.signals[index] = Some(signal);
        Ok(())
    }

    /// Returns the signal for one package pin.
    #[must_use]
    pub fn get(&self, pin: u8) -> Option<AvrSignal> {
        self.signals.get(pin_index(pin).ok()?).copied().flatten()
    }

    /// Returns the number of mapped package pins.
    #[must_use]
    pub fn len(&self) -> usize {
        self.signals.iter().flatten().count()
    }

    /// Returns true when no package pin is mapped.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Builds the standard ATmega328P TQFP-32 pin map.
    #[must_use]
    pub fn atmega328p_tqfp32() -> Self {
        let mut map = Self::default();
        for (index, signal) in TQFP32.iter().enumerate() {
            map.signals[index] = Some(*signal);
        }
        map
    }
}

fn pin_index(pin: u8) -> Result<usize, SimError> {
    if !(1..=32).contains(&pin) {
        return Err(SimError::InvalidNetlist(format!(
            "ATmega328P package pin out of range: {pin}"
        )));
    }
    Ok(usize::from(pin - 1))
}

const TQFP32: [AvrSignal; 32] = [
    AvrSignal::Io { port: 2, bit: 3 },
    AvrSignal::Io { port: 2, bit: 4 },
    AvrSignal::Gnd,
    AvrSignal::Vcc,
    AvrSignal::Gnd,
    AvrSignal::Vcc,
    AvrSignal::Io { port: 0, bit: 6 },
    AvrSignal::Io { port: 0, bit: 7 },
    AvrSignal::Io { port: 2, bit: 5 },
    AvrSignal::Io { port: 2, bit: 6 },
    AvrSignal::Io { port: 2, bit: 7 },
    AvrSignal::Io { port: 0, bit: 0 },
    AvrSignal::Io { port: 0, bit: 1 },
    AvrSignal::Io { port: 0, bit: 2 },
    AvrSignal::Io { port: 0, bit: 3 },
    AvrSignal::Io { port: 0, bit: 4 },
    AvrSignal::Io { port: 0, bit: 5 },
    AvrSignal::Avcc,
    AvrSignal::Adc6,
    AvrSignal::Aref,
    AvrSignal::Gnd,
    AvrSignal::Adc7,
    AvrSignal::Io { port: 1, bit: 0 },
    AvrSignal::Io { port: 1, bit: 1 },
    AvrSignal::Io { port: 1, bit: 2 },
    AvrSignal::Io { port: 1, bit: 3 },
    AvrSignal::Io { port: 1, bit: 4 },
    AvrSignal::Io { port: 1, bit: 5 },
    AvrSignal::Io { port: 1, bit: 6 },
    AvrSignal::Io { port: 2, bit: 0 },
    AvrSignal::Io { port: 2, bit: 1 },
    AvrSignal::Io { port: 2, bit: 2 },
];

#[cfg(test)]
mod tests {
    use super::{AvrPackageMap, AvrSignal};

    #[test]
    fn tqfp32_maps_io_and_power_pins() {
        let map = AvrPackageMap::atmega328p_tqfp32();
        assert_eq!(map.len(), 32);
        assert_eq!(map.get(1), Some(AvrSignal::Io { port: 2, bit: 3 }));
        assert_eq!(map.get(17), Some(AvrSignal::Io { port: 0, bit: 5 }));
        assert_eq!(map.get(29), Some(AvrSignal::Io { port: 1, bit: 6 }));
        assert_eq!(map.get(4), Some(AvrSignal::Vcc));
        assert_eq!(map.get(18), Some(AvrSignal::Avcc));
        assert_eq!(map.get(19), Some(AvrSignal::Adc6));
        assert_eq!(map.get(21), Some(AvrSignal::Gnd));
    }

    #[test]
    fn tqfp32_counts_signals() {
        let map = AvrPackageMap::atmega328p_tqfp32();
        let io = count(&map, |signal| matches!(signal, AvrSignal::Io { .. }));
        let vcc = count(&map, |signal| signal == AvrSignal::Vcc);
        let gnd = count(&map, |signal| signal == AvrSignal::Gnd);
        assert_eq!(io, 23);
        assert_eq!(vcc, 2);
        assert_eq!(gnd, 3);
    }

    fn count(map: &AvrPackageMap, predicate: impl Fn(AvrSignal) -> bool) -> usize {
        (1..=32)
            .filter_map(|pin| map.get(pin))
            .filter(|signal| predicate(*signal))
            .count()
    }

    #[test]
    fn package_map_rejects_invalid_and_duplicate_pins() {
        let mut map = AvrPackageMap::new();
        assert!(map.add(0, AvrSignal::Vcc).is_err());
        assert!(map.add(33, AvrSignal::Vcc).is_err());
        assert!(map.add(1, AvrSignal::Vcc).is_ok());
        assert!(map.add(1, AvrSignal::Gnd).is_err());
        assert_eq!(map.get(1), Some(AvrSignal::Vcc));
        assert!(!map.is_empty());
    }
}
