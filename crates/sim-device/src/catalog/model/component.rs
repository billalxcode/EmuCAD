use super::*;

impl ComponentSpec {
    /// Validates catalog invariants for this entry.
    pub fn validate(&self) -> Result<(), CatalogError> {
        self.validate_text()?;
        self.validate_values()?;
        self.validate_class()
    }

    fn validate_text(&self) -> Result<(), CatalogError> {
        if self.id.is_empty() || !self.id.is_ascii() {
            return Err(self.error("id must be non-empty ASCII"));
        }
        if self.name.is_empty() || !self.name.is_ascii() {
            return Err(self.error("name must be non-empty ASCII"));
        }
        if self.parameter.is_empty() || !self.parameter.is_ascii() {
            return Err(self.error("parameter must be non-empty ASCII"));
        }
        if self.unit.is_empty() || !self.unit.is_ascii() {
            return Err(self.error("unit must be non-empty ASCII"));
        }
        Ok(())
    }

    fn validate_values(&self) -> Result<(), CatalogError> {
        if self.pins == 0 {
            return Err(self.error("pins must be positive"));
        }
        if !self.default.is_finite() {
            return Err(self.error("default must be finite"));
        }
        if self.class == ModelClass::LogicGate && self.variant > 8 {
            return Err(self.error("logic gate variant must be 0 to 8"));
        }
        Ok(())
    }

    fn validate_class(&self) -> Result<(), CatalogError> {
        if self.class.principle().is_empty() {
            return Err(self.error("principle must be non-empty"));
        }
        if self.class.family().source().is_empty() {
            return Err(self.error("source must be non-empty"));
        }
        Ok(())
    }

    fn error(&self, reason: &'static str) -> CatalogError {
        CatalogError {
            id: self.id,
            reason,
        }
    }
}
