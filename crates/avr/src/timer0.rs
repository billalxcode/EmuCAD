//! Timer0 counter, prescaler, normal mode, and CTC.

use kernel::SimError;

/// Timer0 core with an 8-bit functional counter.
pub struct Timer0 {
    counter: u8,
    prescaler: u32,
    divider: u32,
    compare: u8,
    ctc: bool,
    pwm: bool,
    inverting: bool,
    pwm_output: bool,
    overflow: bool,
    compare_match: bool,
    overflow_count: u32,
    compare_count: u32,
    overflow_cycle: Option<u32>,
    compare_cycle: Option<u32>,
}

impl Timer0 {
    /// Builds Timer0 in normal mode with prescaler 1.
    pub fn new() -> Result<Self, SimError> {
        Ok(Self {
            counter: 0,
            prescaler: 0,
            divider: 1,
            compare: 0,
            ctc: false,
            pwm: false,
            inverting: false,
            pwm_output: false,
            overflow: false,
            compare_match: false,
            overflow_count: 0,
            compare_count: 0,
            overflow_cycle: None,
            compare_cycle: None,
        })
    }

    /// Sets the prescaler select value.
    pub fn set_prescaler(&mut self, select: u8) -> Result<(), SimError> {
        self.divider = match select {
            0 => 1,
            1 => 8,
            2 => 64,
            3 => 256,
            4 => 1024,
            _ => {
                return Err(SimError::InvalidNetlist(
                    "invalid Timer0 prescaler".to_string(),
                ));
            }
        };
        self.prescaler = 0;
        Ok(())
    }

    /// Sets the compare value.
    pub fn set_compare(&mut self, value: u8) {
        self.compare = value;
    }

    /// Enables or disables CTC mode.
    pub fn set_ctc(&mut self, enabled: bool) {
        self.ctc = enabled;
        self.pwm = false;
        if !enabled {
            self.pwm_output = false;
        }
    }

    /// Enables fast PWM and selects inverting or non-inverting output.
    pub fn set_pwm(&mut self, inverting: bool) {
        self.pwm = true;
        self.inverting = inverting;
        self.ctc = false;
    }

    /// Returns the PWM output compare A signal.
    #[must_use]
    pub const fn pwm_output(&self) -> bool {
        self.pwm_output
    }

    /// Advances the timer by CPU cycles.
    pub fn tick(&mut self, cycles: u32) -> Result<(), SimError> {
        self.overflow_cycle = None;
        self.compare_cycle = None;
        let initial = self.prescaler;
        self.prescaler = self.prescaler.saturating_add(cycles);
        let ticks = self.prescaler / self.divider;
        self.prescaler %= self.divider;
        for index in 0..ticks {
            let offset = (index + 1) * self.divider - initial;
            self.step(offset.min(cycles));
        }
        Ok(())
    }

    /// Returns the counter value.
    #[must_use]
    pub const fn counter(&self) -> u8 {
        self.counter
    }

    /// Takes and clears the overflow flag.
    pub fn take_overflow(&mut self) -> bool {
        let value = self.overflow;
        self.overflow = false;
        value
    }

    /// Returns the overflow flag without clearing it.
    #[must_use]
    pub const fn overflow_flag(&self) -> bool {
        self.overflow
    }

    /// Returns the compare flag without clearing it.
    #[must_use]
    pub const fn compare_flag(&self) -> bool {
        self.compare_match
    }

    /// Takes and clears the compare flag.
    pub fn take_compare(&mut self) -> bool {
        let value = self.compare_match;
        self.compare_match = false;
        value
    }

    /// Returns the number of overflow events since construction.
    #[must_use]
    pub const fn overflow_count(&self) -> u32 {
        self.overflow_count
    }

    /// Returns the number of compare events since construction.
    #[must_use]
    pub const fn compare_count(&self) -> u32 {
        self.compare_count
    }

    /// Returns the cycle offset of the first overflow in the last tick.
    #[must_use]
    pub const fn overflow_cycle(&self) -> Option<u32> {
        self.overflow_cycle
    }

    /// Returns the cycle offset of the first compare in the last tick.
    #[must_use]
    pub const fn compare_cycle(&self) -> Option<u32> {
        self.compare_cycle
    }

    /// Resets the counter, prescaler, and flags.
    ///
    /// Event counters are preserved so observers keep their history.
    pub fn reset(&mut self) -> Result<(), SimError> {
        self.counter = 0;
        self.prescaler = 0;
        self.overflow = false;
        self.compare_match = false;
        Ok(())
    }

    fn step(&mut self, offset: u32) {
        let previous = self.counter;
        self.counter = self.counter.wrapping_add(1);
        self.update_pwm();
        if previous == 0xFF {
            self.overflow = true;
            self.overflow_count = self.overflow_count.wrapping_add(1);
            self.overflow_cycle.get_or_insert(offset);
        }
        if self.counter == self.compare {
            self.compare_match = true;
            self.compare_count = self.compare_count.wrapping_add(1);
            self.compare_cycle.get_or_insert(offset);
            if self.ctc {
                self.counter = 0;
            }
        }
    }

    fn update_pwm(&mut self) {
        if !self.pwm {
            self.pwm_output = false;
            return;
        }
        self.pwm_output = if self.inverting {
            self.counter >= self.compare
        } else {
            self.counter < self.compare
        };
    }
}

#[cfg(test)]
mod tests {
    use super::Timer0;

    fn timer() -> Timer0 {
        Timer0::new().expect("timer")
    }

    #[test]
    fn prescaler_divides_clock() {
        let mut timer = timer();
        timer.set_prescaler(1).expect("prescaler");
        timer.tick(7).expect("tick");
        assert_eq!(timer.counter(), 0);
        timer.tick(1).expect("tick");
        assert_eq!(timer.counter(), 1);
        timer.tick(8).expect("tick");
        assert_eq!(timer.counter(), 2);
    }

    #[test]
    fn counter_wraps_and_sets_overflow() {
        let mut timer = timer();
        timer.tick(256).expect("tick");
        assert_eq!(timer.counter(), 0);
        assert!(timer.take_overflow());
    }

    #[test]
    fn compare_sets_flag_in_normal_mode() {
        let mut timer = timer();
        timer.set_compare(3);
        timer.tick(4).expect("tick");
        assert_eq!(timer.counter(), 4);
        assert!(timer.take_compare());
    }

    #[test]
    fn ctc_resets_counter_on_compare() {
        let mut timer = timer();
        timer.set_compare(3);
        timer.set_ctc(true);
        timer.tick(3).expect("tick");
        assert_eq!(timer.counter(), 0);
        assert!(timer.take_compare());
    }

    #[test]
    fn reset_clears_counter_and_flags() {
        let mut timer = timer();
        timer.set_compare(3);
        timer.tick(4).expect("tick");
        timer.reset().expect("reset");
        assert_eq!(timer.counter(), 0);
        assert!(!timer.take_compare());
    }

    #[test]
    fn event_counters_track_repeated_events() {
        let mut compare = timer();
        compare.set_compare(3);
        compare.set_ctc(true);
        compare.tick(8).expect("tick");
        assert_eq!(compare.compare_count(), 2);
        let mut overflow = timer();
        overflow.set_compare(200);
        overflow.tick(256).expect("tick");
        assert_eq!(overflow.overflow_count(), 1);
    }

    #[test]
    fn event_cycle_offsets_are_recorded() {
        let mut compare = timer();
        compare.set_compare(1);
        compare.tick(2).expect("tick");
        assert_eq!(compare.compare_cycle(), Some(1));
        let mut overflow = timer();
        overflow.tick(256).expect("tick");
        assert_eq!(overflow.overflow_cycle(), Some(256));
    }

    #[test]
    fn prescaler_shifts_event_cycle_offset() {
        let mut timer = timer();
        timer.set_prescaler(1).expect("prescaler");
        timer.set_compare(1);
        timer.tick(8).expect("tick");
        assert_eq!(timer.compare_cycle(), Some(8));
    }

    #[test]
    fn invalid_prescaler_is_rejected() {
        let mut timer = timer();
        assert!(timer.set_prescaler(5).is_err());
    }

    #[test]
    fn fast_pwm_tracks_compare_value() {
        let mut timer = timer();
        timer.set_compare(2);
        timer.set_pwm(false);
        timer.tick(1).expect("tick");
        assert!(timer.pwm_output());
        timer.tick(1).expect("tick");
        assert!(!timer.pwm_output());
        timer.tick(254).expect("tick");
        assert_eq!(timer.counter(), 0);
        assert!(timer.pwm_output());
    }

    #[test]
    fn inverting_pwm_is_complementary() {
        let mut timer = timer();
        timer.set_compare(2);
        timer.set_pwm(true);
        timer.tick(1).expect("tick");
        assert!(!timer.pwm_output());
        timer.tick(1).expect("tick");
        assert!(timer.pwm_output());
    }
}
