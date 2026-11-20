//! Digital bit vectors with explicit width.

use super::blocks::read_bits;
use super::circuit::{DigitalBuilder, DigitalCircuit, NetId};
use kernel::SimError;

const MAX_WIDTH: usize = 64;

/// A little-endian vector of digital nets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DigitalVector {
    bits: Vec<NetId>,
}

impl DigitalVector {
    /// Builds a vector around existing nets.
    pub fn new(bits: Vec<NetId>) -> Result<Self, SimError> {
        validate_width(bits.len())?;
        Ok(Self { bits })
    }

    /// Allocates a zeroed vector with the requested width.
    pub fn allocate(builder: &mut DigitalBuilder, width: usize) -> Result<Self, SimError> {
        validate_width(width)?;
        let bits = (0..width).map(|_| builder.net()).collect();
        Ok(Self { bits })
    }

    /// Returns the vector width in bits.
    #[must_use]
    pub fn width(&self) -> usize {
        self.bits.len()
    }

    /// Returns the nets from least significant to most significant.
    #[must_use]
    pub fn bits(&self) -> &[NetId] {
        &self.bits
    }

    /// Returns a slice of the vector.
    pub fn slice(&self, start: usize, width: usize) -> Result<Self, SimError> {
        validate_width(width)?;
        let end = start.checked_add(width).ok_or_else(range_error)?;
        if end > self.width() {
            return Err(range_error());
        }
        Self::new(self.bits[start..end].to_vec())
    }

    /// Concatenates vectors from least to most significant.
    pub fn concat(parts: &[Self]) -> Result<Self, SimError> {
        if parts.is_empty() {
            return Err(SimError::InvalidNetlist(
                "digital vector needs at least one part".to_string(),
            ));
        }
        let width = parts.iter().map(Self::width).sum();
        validate_width(width)?;
        let bits = parts
            .iter()
            .flat_map(|part| part.bits.iter().copied())
            .collect();
        Ok(Self { bits })
    }

    /// Extends the vector to a larger width with zero bits.
    pub fn zero_extend(
        &self,
        builder: &mut DigitalBuilder,
        width: usize,
    ) -> Result<Self, SimError> {
        validate_extension(self.width(), width)?;
        let zero = builder.constant(false);
        let mut bits = self.bits.clone();
        bits.resize(width, zero);
        Ok(Self { bits })
    }

    /// Extends the vector to a larger width with its sign bit.
    pub fn sign_extend(
        &self,
        builder: &mut DigitalBuilder,
        width: usize,
    ) -> Result<Self, SimError> {
        validate_extension(self.width(), width)?;
        let Some(&sign) = self.bits.last() else {
            return Err(SimError::InvalidNetlist("empty digital vector".to_string()));
        };
        let mut bits = self.bits.clone();
        for _ in self.width()..width {
            let bit = builder.net();
            builder.buf_into(sign, bit);
            bits.push(bit);
        }
        Ok(Self { bits })
    }

    /// Reads the vector as an unsigned integer.
    #[must_use]
    pub fn read(&self, circuit: &DigitalCircuit) -> u64 {
        read_bits(circuit, &self.bits)
    }
}

fn validate_width(width: usize) -> Result<(), SimError> {
    if width == 0 || width > MAX_WIDTH {
        return Err(SimError::InvalidNetlist(format!(
            "digital vector width must be 1..={MAX_WIDTH}"
        )));
    }
    Ok(())
}

fn validate_extension(current: usize, width: usize) -> Result<(), SimError> {
    validate_width(width)?;
    if width < current {
        return Err(SimError::InvalidNetlist(
            "digital vector extension cannot shrink".to_string(),
        ));
    }
    Ok(())
}

fn range_error() -> SimError {
    SimError::InvalidNetlist("digital vector slice out of range".to_string())
}

#[cfg(test)]
mod tests {
    use super::DigitalVector;
    use crate::DigitalBuilder;

    #[test]
    fn allocates_and_reads_zeroed_vector() {
        let mut builder = DigitalBuilder::new();
        let vector = DigitalVector::allocate(&mut builder, 8).expect("vector");
        let circuit = builder.build();
        assert_eq!(vector.width(), 8);
        assert_eq!(vector.read(&circuit), 0);
    }

    #[test]
    fn slice_and_concat_preserve_order() {
        let mut builder = DigitalBuilder::new();
        let vector = DigitalVector::allocate(&mut builder, 8).expect("vector");
        let low = vector.slice(0, 4).expect("low");
        let high = vector.slice(4, 4).expect("high");
        let joined = DigitalVector::concat(&[low, high]).expect("concat");
        assert_eq!(joined.bits(), vector.bits());
    }

    #[test]
    fn extensions_keep_low_bits_and_fill_high_bits() {
        let mut builder = DigitalBuilder::new();
        let vector = DigitalVector::allocate(&mut builder, 4).expect("vector");
        let zero = vector.zero_extend(&mut builder, 8).expect("zero");
        let sign = vector.sign_extend(&mut builder, 8).expect("sign");
        let mut circuit = builder.build();
        circuit.set_net(vector.bits()[3], true);
        circuit.settle().expect("settle");
        assert_eq!(zero.read(&circuit), 0x08);
        assert_eq!(sign.read(&circuit), 0xF8);
    }

    #[test]
    fn rejects_invalid_widths_and_ranges() {
        let mut builder = DigitalBuilder::new();
        assert!(DigitalVector::allocate(&mut builder, 0).is_err());
        assert!(DigitalVector::allocate(&mut builder, 65).is_err());
        let vector = DigitalVector::allocate(&mut builder, 4).expect("vector");
        assert!(vector.slice(2, 4).is_err());
        assert!(vector.zero_extend(&mut builder, 2).is_err());
        assert!(DigitalVector::concat(&[]).is_err());
    }
}
