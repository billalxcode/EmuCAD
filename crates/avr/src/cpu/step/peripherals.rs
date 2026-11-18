use super::*;

impl Cpu {
    /// Returns the Timer0 PWM output compare A signal.
    #[must_use]
    pub const fn pwm_a(&self) -> bool {
        self.datapath.timers.timer0.pwm_output()
    }
}

impl Cpu {
    /// Returns the Timer0 core for configuration.
    pub fn timer0(&mut self) -> &mut Timer0 {
        &mut self.datapath.timers.timer0
    }

    /// Selects the Timer0 clock source through TCCR0B.
    pub fn set_timer0_prescaler(&mut self, select: u8) -> Result<(), SimError> {
        if !(1..=5).contains(&select) {
            return Err(SimError::InvalidNetlist(
                "invalid Timer0 clock select".to_string(),
            ));
        }
        self.io[0x25] = select;
        self.apply_timer0_clock()
    }
}

impl Cpu {
    /// Returns the Timer1 core for configuration.
    pub fn timer1(&mut self) -> &mut Timer1 {
        &mut self.datapath.timers.timer1
    }

    /// Returns the Timer2 core for configuration.
    pub fn timer2(&mut self) -> &mut Timer2 {
        &mut self.datapath.timers.timer2
    }

    /// Returns the UART core for configuration.
    pub fn uart(&mut self) -> &mut Uart {
        &mut self.datapath.uart
    }

    /// Returns the SPI core for configuration.
    pub fn spi(&mut self) -> &mut Spi {
        &mut self.datapath.serial.spi
    }

    /// Returns the TWI core for configuration.
    pub fn twi(&mut self) -> &mut Twi {
        &mut self.datapath.serial.twi
    }

    /// Returns the ADC core for configuration.
    pub fn adc(&mut self) -> &mut Adc {
        &mut self.datapath.serial.adc
    }

    /// Requests SPI interrupt when SPIF is set.
    pub fn poll_spi(&mut self) -> Result<(), SimError> {
        if self.datapath.serial.spi.spif() {
            self.request_interrupt(17)?;
        }
        Ok(())
    }

    /// Requests TWI interrupt when TWINT is set.
    pub fn poll_twi(&mut self) -> Result<(), SimError> {
        if self.datapath.serial.twi.twint() {
            self.request_interrupt(24)?;
        }
        Ok(())
    }

    /// Requests ADC interrupt when ADIF is set.
    pub fn poll_adc(&mut self) -> Result<(), SimError> {
        if self.datapath.serial.adc.flag() {
            self.request_interrupt(21)?;
        }
        Ok(())
    }

    /// Requests UART interrupts for the provided flag mask.
    pub fn poll_uart(&mut self, mask: u8) -> Result<(), SimError> {
        let uart = &self.datapath.uart;
        let rx = uart.rx_ready();
        let tx = uart.tx_complete();
        let udre = uart.tx_ready();
        self.update_uart_interrupt(18, rx, mask & 0x01 != 0)?;
        self.update_uart_interrupt(20, tx, mask & 0x02 != 0)?;
        self.update_uart_interrupt(19, udre, mask & 0x04 != 0)
    }

    pub(crate) fn update_uart_interrupt(
        &mut self,
        index: u8,
        flag: bool,
        enabled: bool,
    ) -> Result<(), SimError> {
        if flag && enabled {
            return self.request_interrupt(index);
        }
        self.pending &= !(1u32 << index);
        Ok(())
    }

    /// Requests Timer1 interrupts for the provided flag mask.
    pub fn poll_timer1(&mut self, mask: u8) -> Result<(), SimError> {
        let timer = &self.datapath.timers.timer1;
        let capture = timer.capture_flag();
        let compare_a = timer.compare_a_flag();
        let compare_b = timer.compare_b_flag();
        let overflow = timer.overflow_flag();
        if capture && mask & 0x01 != 0 {
            self.request_interrupt(10)?;
        }
        if compare_a && mask & 0x02 != 0 {
            self.request_interrupt(11)?;
        }
        if compare_b && mask & 0x04 != 0 {
            self.request_interrupt(12)?;
        }
        if overflow && mask & 0x08 != 0 {
            self.request_interrupt(13)?;
        }
        Ok(())
    }

    /// Requests Timer2 interrupts for the provided flag mask.
    pub fn poll_timer2(&mut self, mask: u8) -> Result<(), SimError> {
        let timer = &self.datapath.timers.timer2;
        let overflow = timer.overflow_flag();
        let compare = timer.compare_flag();
        if overflow && mask & 0x01 != 0 {
            self.request_interrupt(9)?;
        }
        if compare && mask & 0x02 != 0 {
            self.request_interrupt(7)?;
        }
        Ok(())
    }

    /// Returns the stack pointer.
    #[must_use]
    pub const fn sp(&self) -> u16 {
        self.sp
    }
}
