use super::*;

pub(super) fn build_datapath(builder: &mut DigitalBuilder) -> Result<Datapath, SimError> {
    let nets = build_alu_nets(builder);
    let selected = select_operand(builder, &nets.second, &nets.immediate, nets.use_immediate)?;
    let units = build_units(builder, &nets, &selected)?;
    let gpio = build_gpio(builder)?;
    let (bank, timers) = build_io(builder)?;
    let serial = Serial::new();
    let uart = Uart::new();
    Ok(Datapath {
        bank,
        arithmetic: units.arithmetic,
        logic: units.logic,
        shift: units.shift,
        multiply: units.multiply,
        gpio,
        timers,
        serial,
        uart,
    })
}

pub(super) fn build_io(builder: &mut DigitalBuilder) -> Result<(RegisterBank, Timers), SimError> {
    Ok((
        build_bank(builder)?,
        Timers {
            timer0: Timer0::new()?,
            timer1: Timer1::new()?,
            timer2: Timer2::new()?,
        },
    ))
}

pub(super) fn build_units(
    builder: &mut DigitalBuilder,
    nets: &AluNets,
    selected: &[NetId],
) -> Result<Units, SimError> {
    let arithmetic = arithmetic_unit(builder, &nets.first, selected, nets.subtract, nets.carry)?;
    let logic_op = named_inputs(builder, "lop", 2);
    let logic = logic_unit(builder, &nets.first, selected, &logic_op)?;
    let shift_op = named_inputs(builder, "sop", 2);
    let shift = shift_unit(builder, &nets.first, nets.carry, &shift_op)?;
    let multiply = multiply_unit(builder, &nets.first, &nets.second)?;
    Ok(Units {
        arithmetic,
        logic,
        shift,
        multiply,
    })
}

pub(super) fn build_gpio(builder: &mut DigitalBuilder) -> Result<Gpio, SimError> {
    Ok(Gpio {
        b: GpioPort::new(builder, "pb")?,
        c: GpioPort::new(builder, "pc")?,
        d: GpioPort::new(builder, "pd")?,
    })
}

pub(super) fn build_alu_nets(builder: &mut DigitalBuilder) -> AluNets {
    AluNets {
        first: named_inputs(builder, "a", 8),
        second: named_inputs(builder, "b", 8),
        immediate: named_inputs(builder, "k", 8),
        use_immediate: builder.input("imm"),
        subtract: builder.input("sub"),
        carry: builder.input("cin"),
    }
}

pub(super) fn build_bank(builder: &mut DigitalBuilder) -> Result<RegisterBank, SimError> {
    let ports = RegisterPorts {
        write_address: named_inputs(builder, "wa", 5),
        write_data: named_inputs(builder, "wd", 8),
        write_enable: builder.input("we"),
        read_address_a: named_inputs(builder, "ra", 5),
        read_address_b: named_inputs(builder, "rb", 5),
    };
    RegisterBank::new(builder, &ports)
}
