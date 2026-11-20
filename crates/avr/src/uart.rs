//! Behavioral AVR USART transmitter and receiver model.

use std::collections::VecDeque;

const TX_ENABLE: u8 = 1 << 3;
const RX_ENABLE: u8 = 1 << 4;
#[cfg(test)]
const FRAME_BITS: u32 = 10;
const NORMAL_MODE_FACTOR: u32 = 16;
const DOUBLE_SPEED_FACTOR: u32 = 8;
const NORMAL_SAMPLES: u32 = 16;
const DOUBLE_SAMPLES: u32 = 8;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum RxLineState {
    #[default]
    Idle,
    Start,
    Data,
    Parity,
    Stop,
}

#[derive(Clone, Copy, Default)]
struct RxState {
    data: u8,
    rxc: bool,
    active: bool,
    shift: u8,
    bits: u8,
    line: bool,
    line_active: bool,
    line_elapsed: u32,
    line_state: RxLineState,
    line_timer: u32,
    line_shift: u8,
    line_bits: u8,
    line_ones: u8,
    line_parity_ok: bool,
    count: u32,
    ninth: bool,
    line_ninth: bool,
}

/// Behavioral model of the AVR USART.
pub struct Uart {
    ucsr_a: u8,
    ucsr_b: u8,
    ucsr_c: u8,
    ubrr: u16,
    tx_data: u8,
    tx_bit_cycles: u32,
    tx_frame_cycles: u32,
    tx_edge_cycle: Option<u32>,
    tx_elapsed: u32,
    tx_active: bool,
    tx_queue: VecDeque<u8>,
    udre: bool,
    txc: bool,
    rx: RxState,
}

impl Uart {
    /// Builds a USART in its reset state with an empty transmitter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ucsr_a: 0,
            ucsr_b: 0,
            ucsr_c: 0x06,
            ubrr: 0,
            tx_data: 0,
            tx_bit_cycles: 0,
            tx_frame_cycles: 0,
            tx_edge_cycle: None,
            tx_elapsed: 0,
            tx_active: false,
            tx_queue: VecDeque::new(),
            udre: true,
            txc: false,
            rx: rx_reset(),
        }
    }

    /// Writes a control register: 0 = UCSR0A, 1 = UCSR0B, 2 = UCSR0C.
    pub fn write_control(&mut self, register: u8, value: u8) {
        match register {
            0 => self.ucsr_a = value,
            1 => self.ucsr_b = value,
            2 => self.ucsr_c = value,
            _ => {}
        }
    }

    /// Stores the 12-bit UBRR baud divider.
    pub fn write_baud(&mut self, low: u8, high: u8) {
        self.ubrr = u16::from(low) | ((u16::from(high) & 0x0F) << 8);
    }

    /// Loads one byte when the transmitter is enabled and idle.
    pub fn write_tx(&mut self, value: u8) {
        if self.ucsr_b & TX_ENABLE == 0 {
            return;
        }
        if self.tx_active {
            return;
        }
        self.tx_data = value;
        self.udre = false;
        self.txc = false;
        self.tx_elapsed = 0;
        self.tx_bit_cycles = self.cycles_per_bit();
        self.tx_frame_cycles = self.tx_bit_cycles.saturating_mul(self.frame_bits());
        self.tx_active = true;
        self.tx_edge_cycle = Some(0);
    }

    /// Returns the last received byte without clearing RXC.
    #[must_use]
    pub const fn rx_byte(&self) -> u8 {
        self.rx.data
    }

    /// Returns the received byte and clears RXC.
    pub fn read_rx(&mut self) -> u8 {
        let value = self.rx.data;
        self.rx.rxc = false;
        value
    }

    /// Records an externally received byte and sets RXC.
    pub fn receive(&mut self, value: u8) {
        self.complete_rx(value);
    }

    /// Returns the number of received bytes since construction.
    #[must_use]
    pub const fn rx_count(&self) -> u32 {
        self.rx.count
    }

    /// Returns the ninth received data bit.
    #[must_use]
    pub const fn rx_ninth(&self) -> bool {
        self.rx.ninth
    }

    fn complete_rx(&mut self, value: u8) {
        self.rx.data = value;
        self.rx.ninth = false;
        self.rx.rxc = true;
        self.rx.count = self.rx.count.wrapping_add(1);
    }

    /// Shifts one received serial bit into the receiver.
    pub fn receive_bit(&mut self, level: bool) {
        if !self.rx.active {
            self.start_rx(level);
            return;
        }
        if self.rx.bits >= 8 {
            return;
        }
        self.shift_rx(level);
    }

    /// Sets the sampled RX line level for timed 8N1 reception.
    pub fn set_rx_line(&mut self, level: bool) {
        self.rx.line = level;
        self.rx.line_active = true;
    }

    /// Returns true when the timed RX line receiver is active.
    #[must_use]
    pub const fn rx_line_active(&self) -> bool {
        self.rx.line_active
    }

    fn start_rx(&mut self, level: bool) {
        if level {
            return;
        }
        self.rx.active = true;
        self.rx.bits = 0;
        self.rx.shift = 0;
    }

    fn shift_rx(&mut self, level: bool) {
        if level {
            self.rx.shift |= 1 << self.rx.bits;
        }
        self.rx.bits += 1;
        if self.rx.bits == 8 {
            self.complete_rx(self.rx.shift);
            self.rx.active = false;
        }
    }

    /// Returns true while a frame is being transmitted.
    #[must_use]
    pub const fn tx_active(&self) -> bool {
        self.tx_active
    }

    /// Advances the transmitter shift timing by CPU cycles.
    pub fn tick(&mut self, cycles: u32) {
        if cycles == 0 {
            return;
        }
        let bit_cycles = self.tx_bit_cycles.max(1);
        let previous_line = self.tx_line();
        let elapsed = self.tx_elapsed;
        self.advance_tx(cycles);
        if self.tx_line() != previous_line {
            let offset = tx_edge_offset(elapsed, cycles, bit_cycles);
            self.tx_edge_cycle.get_or_insert(offset);
        }
        self.advance_rx_line(cycles);
    }

    /// Takes the cycle offset of the first TX edge since the last call.
    pub fn take_tx_edge_cycle(&mut self) -> Option<u32> {
        self.tx_edge_cycle.take()
    }

    /// Returns true when the TX data register is empty (UDRE).
    #[must_use]
    pub fn tx_ready(&self) -> bool {
        self.udre
    }

    /// Returns the last byte loaded into the transmitter.
    #[must_use]
    pub const fn tx_byte(&self) -> u8 {
        self.tx_data
    }

    /// Takes the next fully transmitted byte.
    pub fn take_tx_byte(&mut self) -> Option<u8> {
        self.tx_queue.pop_front()
    }

    /// Returns true when the transmitter is complete (TXC).
    #[must_use]
    pub fn tx_complete(&self) -> bool {
        self.txc
    }

    /// Returns true when a received byte is available (RXC).
    #[must_use]
    pub fn rx_ready(&self) -> bool {
        self.rx.rxc
    }

    /// Takes and clears TXC.
    pub fn take_tx_complete(&mut self) -> bool {
        let value = self.txc;
        self.txc = false;
        value
    }

    /// Takes and clears RXC.
    pub fn take_rx_ready(&mut self) -> bool {
        let value = self.rx.rxc;
        self.rx.rxc = false;
        value
    }

    /// Returns the TX line level; the idle level is high.
    #[must_use]
    pub fn tx_line(&self) -> bool {
        if !self.tx_active {
            return true;
        }
        let position = self.tx_elapsed / self.tx_bit_cycles;
        self.tx_level(position)
    }

    fn tx_level(&self, position: u32) -> bool {
        if position == 0 {
            return false;
        }
        let data_bits = self.data_bits();
        if position <= data_bits {
            if position <= 8 {
                return (self.tx_data >> (position - 1)) & 1 != 0;
            }
            return self.ucsr_b & 0x01 != 0;
        }
        if self.parity_enabled() && position == data_bits + 1 {
            return self.tx_parity();
        }
        true
    }

    fn parity_enabled(&self) -> bool {
        self.ucsr_c & 0x30 != 0
    }

    fn parity_value(&self, ones: u32) -> bool {
        let odd = self.ucsr_c & 0x30 == 0x30;
        if odd { ones % 2 == 0 } else { ones % 2 == 1 }
    }

    fn tx_parity(&self) -> bool {
        let mut ones = (0..self.data_bits().min(8))
            .filter(|bit| (self.tx_data >> bit) & 1 != 0)
            .count() as u32;
        if self.data_bits() == 9 && self.ucsr_b & 0x01 != 0 {
            ones += 1;
        }
        self.parity_value(ones)
    }

    /// Returns true while a frame is being shifted out.
    #[must_use]
    pub fn tx_activity(&self) -> bool {
        self.tx_active
    }

    fn advance_rx_line(&mut self, cycles: u32) {
        if !self.rx.line_active || self.ucsr_b & RX_ENABLE == 0 {
            return;
        }
        let period = self.rx_sample_period();
        self.rx.line_elapsed = self.rx.line_elapsed.saturating_add(cycles);
        while self.rx.line_elapsed >= period {
            self.rx.line_elapsed -= period;
            self.sample_rx_line();
        }
    }

    fn rx_sample_period(&self) -> u32 {
        let samples = if self.ucsr_a & 0x02 != 0 {
            DOUBLE_SAMPLES
        } else {
            NORMAL_SAMPLES
        };
        (self.cycles_per_bit() / samples).max(1)
    }

    fn sample_rx_line(&mut self) {
        let samples = if self.ucsr_a & 0x02 != 0 {
            DOUBLE_SAMPLES
        } else {
            NORMAL_SAMPLES
        };
        match self.rx.line_state {
            RxLineState::Idle => self.rx_idle(),
            RxLineState::Start => self.rx_start(samples),
            RxLineState::Data => self.rx_line_data(samples),
            RxLineState::Parity => self.rx_line_parity(samples),
            RxLineState::Stop => self.rx_stop(samples),
        }
    }

    fn rx_idle(&mut self) {
        if self.rx.line {
            return;
        }
        self.rx.line_state = RxLineState::Start;
        self.rx.line_timer = 0;
    }

    fn rx_start(&mut self, samples: u32) {
        self.rx.line_timer += 1;
        if self.rx.line_timer < samples / 2 {
            return;
        }
        if self.rx.line {
            self.rx.line_state = RxLineState::Idle;
            return;
        }
        self.rx.line_state = RxLineState::Data;
        self.rx.line_timer = 0;
        self.rx.line_bits = 0;
        self.rx.line_shift = 0;
        self.rx.line_ones = 0;
        self.rx.line_parity_ok = true;
        self.rx.line_ninth = false;
    }

    fn rx_line_data(&mut self, samples: u32) {
        self.rx.line_timer += 1;
        if self.rx.line_timer < samples {
            return;
        }
        self.rx.line_timer = 0;
        self.rx_line_bit();
        self.rx.line_bits += 1;
        if self.rx.line_bits == self.data_bits().min(9) as u8 {
            self.rx.line_state = if self.parity_enabled() {
                RxLineState::Parity
            } else {
                RxLineState::Stop
            };
        }
    }

    fn rx_line_bit(&mut self) {
        if !self.rx.line {
            return;
        }
        if self.rx.line_bits < 8 {
            self.rx.line_shift |= 1 << self.rx.line_bits;
        } else {
            self.rx.line_ninth = true;
        }
        self.rx.line_ones ^= 1;
    }

    fn rx_line_parity(&mut self, samples: u32) {
        self.rx.line_timer += 1;
        if self.rx.line_timer < samples {
            return;
        }
        self.rx.line_timer = 0;
        self.rx.line_parity_ok = self.rx.line == self.parity_value(u32::from(self.rx.line_ones));
        self.rx.line_state = RxLineState::Stop;
    }

    fn rx_stop(&mut self, samples: u32) {
        self.rx.line_timer += 1;
        if self.rx.line_timer < samples {
            return;
        }
        if self.rx.line && self.rx.line_parity_ok {
            self.complete_rx(self.rx.line_shift);
            self.rx.ninth = self.rx.line_ninth;
        }
        self.rx.line_state = RxLineState::Idle;
    }

    fn advance_tx(&mut self, cycles: u32) {
        if !self.tx_active {
            return;
        }
        let elapsed = self.tx_elapsed.saturating_add(cycles);
        if elapsed < self.tx_frame_cycles {
            self.tx_elapsed = elapsed;
            return;
        }
        self.finish_tx();
    }

    fn finish_tx(&mut self) {
        self.tx_active = false;
        self.tx_elapsed = 0;
        self.tx_bit_cycles = 0;
        self.tx_frame_cycles = 0;
        self.tx_queue.push_back(self.tx_data);
        self.udre = true;
        self.txc = true;
    }

    fn cycles_per_bit(&self) -> u32 {
        let factor = if self.ucsr_a & 0x02 != 0 {
            DOUBLE_SPEED_FACTOR
        } else {
            NORMAL_MODE_FACTOR
        };
        (u32::from(self.ubrr) + 1) * factor
    }

    fn data_bits(&self) -> u32 {
        let size = (((self.ucsr_b >> 2) & 1) << 2) | ((self.ucsr_c >> 1) & 0x03);
        match size {
            0 => 5,
            1 => 6,
            2 => 7,
            3 => 8,
            7 => 9,
            _ => 8,
        }
    }

    fn frame_bits(&self) -> u32 {
        let parity = u32::from(self.ucsr_c & 0x30 != 0);
        let stop = if self.ucsr_c & 0x08 != 0 { 2 } else { 1 };
        1 + self.data_bits() + parity + stop
    }
}

impl Default for Uart {
    fn default() -> Self {
        Self::new()
    }
}

fn tx_edge_offset(elapsed: u32, cycles: u32, bit_cycles: u32) -> u32 {
    let next = ((elapsed / bit_cycles) + 1) * bit_cycles;
    next.saturating_sub(elapsed).min(cycles)
}

fn rx_reset() -> RxState {
    RxState {
        line: true,
        ..RxState::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{FRAME_BITS, NORMAL_MODE_FACTOR, Uart};

    fn enabled_uart() -> Uart {
        let mut uart = Uart::new();
        uart.write_control(1, 0x18);
        uart
    }

    #[test]
    fn disabled_transmitter_ignores_write() {
        let mut uart = Uart::new();
        uart.write_tx(0x55);
        assert!(uart.tx_ready());
        assert!(!uart.tx_activity());
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert!(uart.tx_line());
        assert!(!uart.tx_complete());
    }

    #[test]
    fn control_register_b_enables_transmitter() {
        let mut uart = Uart::new();
        uart.write_control(1, 0x08);
        uart.write_tx(0x55);
        assert!(uart.tx_activity());
        assert!(!uart.tx_ready());
    }

    #[test]
    fn control_register_c_does_not_enable_transmitter() {
        let mut uart = Uart::new();
        uart.write_control(2, 0x08);
        uart.write_tx(0x55);
        assert!(!uart.tx_activity());
    }

    #[test]
    fn transmitter_takes_ten_bit_times() {
        let mut uart = enabled_uart();
        uart.write_tx(0xA5);
        assert!(!uart.tx_ready());
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS - 1);
        assert!(uart.tx_activity());
        assert!(!uart.tx_complete());
        uart.tick(1);
        assert!(!uart.tx_activity());
        assert!(uart.tx_complete());
        assert!(uart.tx_ready());
        assert_eq!(uart.tx_byte(), 0xA5);
    }

    #[test]
    fn tx_complete_flag_can_be_taken() {
        let mut uart = enabled_uart();
        uart.write_tx(0x00);
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert!(uart.take_tx_complete());
        assert!(!uart.tx_complete());
        assert!(!uart.take_tx_complete());
    }

    #[test]
    fn completed_bytes_are_queued_in_order() {
        let mut uart = enabled_uart();
        uart.write_tx(b'H');
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        uart.write_tx(b'i');
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert_eq!(uart.take_tx_byte(), Some(b'H'));
        assert_eq!(uart.take_tx_byte(), Some(b'i'));
        assert_eq!(uart.take_tx_byte(), None);
    }

    #[test]
    fn baud_divider_changes_frame_duration() {
        let mut uart = enabled_uart();
        uart.write_baud(1, 0);
        uart.write_tx(0x00);
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert!(uart.tx_activity());
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert!(uart.tx_complete());
    }

    #[test]
    fn baud_high_register_contributes_high_bits() {
        let mut uart = enabled_uart();
        uart.write_baud(0, 1);
        uart.write_tx(0x00);
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert!(uart.tx_activity());
    }

    #[test]
    fn tx_line_follows_frame_bits() {
        let mut uart = enabled_uart();
        assert!(uart.tx_line());
        uart.write_tx(0x01);
        assert!(!uart.tx_line());
        uart.tick(NORMAL_MODE_FACTOR);
        assert!(uart.tx_line());
        uart.tick(NORMAL_MODE_FACTOR);
        assert!(!uart.tx_line());
        uart.tick(NORMAL_MODE_FACTOR * 7);
        assert!(uart.tx_line());
        assert!(uart.tx_activity());
        uart.tick(NORMAL_MODE_FACTOR);
        assert!(!uart.tx_activity());
        assert!(uart.tx_line());
    }

    #[test]
    fn tx_edge_offsets_track_start_and_bit_boundary() {
        let mut uart = enabled_uart();
        uart.write_tx(0x01);
        assert_eq!(uart.take_tx_edge_cycle(), Some(0));
        uart.tick(NORMAL_MODE_FACTOR);
        assert_eq!(uart.take_tx_edge_cycle(), Some(NORMAL_MODE_FACTOR));
        uart.tick(NORMAL_MODE_FACTOR);
        assert_eq!(uart.take_tx_edge_cycle(), Some(NORMAL_MODE_FACTOR));
    }

    #[test]
    fn receive_sets_rxc_and_read_clears_it() {
        let mut uart = Uart::new();
        assert!(!uart.rx_ready());
        uart.receive(0x5A);
        assert!(uart.rx_ready());
        assert_eq!(uart.read_rx(), 0x5A);
        assert!(!uart.rx_ready());
    }

    #[test]
    fn take_rx_ready_clears_rxc_and_keeps_data() {
        let mut uart = Uart::new();
        uart.receive(0x33);
        assert!(uart.take_rx_ready());
        assert!(!uart.rx_ready());
        assert_eq!(uart.read_rx(), 0x33);
    }

    #[test]
    fn rx_shift_collects_eight_data_bits() {
        let mut uart = Uart::new();
        uart.receive_bit(false);
        for bit in 0..8 {
            uart.receive_bit((0xA5 >> bit) & 1 != 0);
        }
        assert!(uart.rx_ready());
        assert_eq!(uart.read_rx(), 0xA5);
    }

    #[test]
    fn source_is_ascii() {
        assert!(include_str!("uart.rs").is_ascii());
    }
    #[test]
    fn double_speed_halves_frame_time() {
        let mut uart = enabled_uart();
        uart.write_control(0, 0x02);
        uart.write_baud(1, 0);
        uart.write_tx(0x00);
        uart.tick(159);
        assert!(uart.tx_activity());
        uart.tick(1);
        assert!(!uart.tx_activity());
    }

    #[test]
    fn parity_adds_one_frame_bit() {
        let mut uart = enabled_uart();
        uart.write_control(2, 0x36);
        uart.write_tx(0x00);
        uart.tick(175);
        assert!(uart.tx_activity());
        uart.tick(1);
        assert!(!uart.tx_activity());
    }

    #[test]
    fn two_stop_bits_add_one_frame_bit() {
        let mut uart = enabled_uart();
        uart.write_control(2, 0x0E);
        uart.write_tx(0x00);
        uart.tick(175);
        assert!(uart.tx_activity());
        uart.tick(1);
        assert!(!uart.tx_activity());
    }

    #[test]
    fn nine_data_bits_add_one_frame_bit() {
        let mut uart = enabled_uart();
        uart.write_control(1, 0x0C);
        uart.write_tx(0x00);
        uart.tick(175);
        assert!(uart.tx_activity());
        uart.tick(1);
        assert!(!uart.tx_activity());
    }
    #[test]
    fn parity_bit_is_transmitted_in_even_mode() {
        let mut uart = enabled_uart();
        uart.write_control(2, 0x26);
        uart.write_tx(0x01);
        uart.tick(NORMAL_MODE_FACTOR * 9);
        assert!(uart.tx_line());
        uart.tick(NORMAL_MODE_FACTOR * 2);
        uart.write_tx(0x03);
        uart.tick(NORMAL_MODE_FACTOR * 9);
        assert!(!uart.tx_line());
    }

    #[test]
    fn line_receiver_checks_even_parity() {
        let mut uart = enabled_uart();
        uart.write_control(2, 0x26);
        send_line_frame(&mut uart, 0x01, true);
        assert!(uart.rx_ready());
        assert_eq!(uart.read_rx(), 0x01);
        send_line_frame(&mut uart, 0x01, false);
        assert!(!uart.rx_ready());
    }

    fn send_line_frame(uart: &mut Uart, value: u8, parity: bool) {
        uart.set_rx_line(false);
        uart.tick(NORMAL_MODE_FACTOR);
        for bit in 0..8 {
            uart.set_rx_line((value >> bit) & 1 != 0);
            uart.tick(NORMAL_MODE_FACTOR);
        }
        uart.set_rx_line(parity);
        uart.tick(NORMAL_MODE_FACTOR);
        uart.set_rx_line(true);
        uart.tick(NORMAL_MODE_FACTOR);
    }

    #[test]
    fn nine_bit_transmitter_uses_txb8() {
        let mut uart = Uart::new();
        uart.write_control(1, 0x0D);
        uart.write_control(2, 0x06);
        uart.write_tx(0x01);
        uart.tick(NORMAL_MODE_FACTOR * 9);
        assert!(uart.tx_line());
        uart.write_control(1, 0x0C);
        assert!(!uart.tx_line());
    }

    #[test]
    fn line_receiver_decodes_nine_data_bits() {
        let mut uart = enabled_uart();
        uart.write_control(1, 0x1C);
        uart.set_rx_line(true);
        uart.tick(NORMAL_MODE_FACTOR);
        uart.set_rx_line(false);
        uart.tick(NORMAL_MODE_FACTOR);
        for bit in 0..8 {
            uart.set_rx_line((0x5A >> bit) & 1 != 0);
            uart.tick(NORMAL_MODE_FACTOR);
        }
        uart.set_rx_line(true);
        uart.tick(NORMAL_MODE_FACTOR);
        uart.set_rx_line(true);
        uart.tick(NORMAL_MODE_FACTOR);
        assert!(uart.rx_ready());
        assert_eq!(uart.read_rx(), 0x5A);
        assert!(uart.rx_ninth());
    }

    #[test]
    fn line_receiver_decodes_8n1_frame() {
        let mut uart = enabled_uart();
        uart.set_rx_line(true);
        uart.tick(NORMAL_MODE_FACTOR);
        uart.set_rx_line(false);
        uart.tick(NORMAL_MODE_FACTOR);
        for bit in 0..8 {
            uart.set_rx_line((0x5A >> bit) & 1 != 0);
            uart.tick(NORMAL_MODE_FACTOR);
        }
        uart.set_rx_line(true);
        uart.tick(NORMAL_MODE_FACTOR);
        assert!(uart.rx_ready());
        assert_eq!(uart.read_rx(), 0x5A);
    }
}
