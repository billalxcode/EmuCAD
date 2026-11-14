use super::*;

impl Cpu {
    pub(super) fn load_arithmetic(
        &mut self,
        destination: u8,
        source: u8,
        immediate_value: u8,
        use_immediate: bool,
    ) -> Result<(), SimError> {
        self.set_byte("a", self.register(destination))?;
        self.set_byte(
            "b",
            if use_immediate {
                0
            } else {
                self.register(source)
            },
        )?;
        self.set_byte("k", immediate_value)?;
        self.set_bool("imm", use_immediate)
    }

    pub(super) fn update_full_flags(&mut self) {
        let flags = self.arithmetic_flags();
        self.sreg.carry = flags.0;
        self.sreg.zero = flags.1;
        self.sreg.negative = flags.2;
        self.sreg.overflow = flags.3;
        self.sreg.sign = flags.4;
        self.sreg.half_carry = flags.5;
    }

    pub(super) fn update_partial_flags(&mut self) {
        let flags = self.arithmetic_flags();
        self.sreg.zero = flags.1;
        self.sreg.negative = flags.2;
        self.sreg.overflow = flags.3;
        self.sreg.sign = flags.4;
    }

    pub(super) fn update_logic_flags(&mut self, mnemonic: Mnemonic) {
        let unit = &self.datapath.logic;
        self.sreg.zero = self.circuit.value(unit.zero);
        self.sreg.negative = self.circuit.value(unit.negative);
        self.sreg.overflow = false;
        self.sreg.sign = self.circuit.value(unit.sign);
        if mnemonic == Mnemonic::Com {
            self.sreg.carry = true;
        }
    }

    pub(super) fn update_shift_flags(&mut self) {
        let unit = &self.datapath.shift;
        self.sreg.carry = self.circuit.value(unit.carry);
        self.sreg.zero = self.circuit.value(unit.zero);
        self.sreg.negative = self.circuit.value(unit.negative);
        self.sreg.overflow = self.circuit.value(unit.overflow);
        self.sreg.sign = self.circuit.value(unit.sign);
    }

    pub(super) fn arithmetic_flags(&self) -> (bool, bool, bool, bool, bool, bool) {
        let unit = &self.datapath.arithmetic;
        (
            self.circuit.value(unit.carry),
            self.circuit.value(unit.zero),
            self.circuit.value(unit.negative),
            self.circuit.value(unit.overflow),
            self.circuit.value(unit.sign),
            self.circuit.value(unit.half_carry),
        )
    }
}
