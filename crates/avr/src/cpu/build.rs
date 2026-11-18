use super::*;

pub(super) fn build_datapath(builder: &mut DigitalBuilder) -> Result<Datapath, SimError> {
    Ok(Datapath {
        gpio: build_gpio(builder)?,
        timers: build_io()?,
        serial: Serial::new(),
        uart: Uart::new(),
    })
}

pub(super) fn build_io() -> Result<Timers, SimError> {
    Ok(Timers {
        timer0: Timer0::new()?,
        timer1: Timer1::new()?,
        timer2: Timer2::new()?,
    })
}

pub(super) fn build_gpio(builder: &mut DigitalBuilder) -> Result<Gpio, SimError> {
    Ok(Gpio {
        b: GpioPort::new(builder, "pb")?,
        c: GpioPort::new(builder, "pc")?,
        d: GpioPort::new(builder, "pd")?,
    })
}
