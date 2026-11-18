//! Wall-clock pacing for the AVR core.

use std::time::{Duration, Instant};

/// Nominal ATmega328P clock.
pub(crate) const CLOCK_HZ: f64 = 16_000_000.0;
const SLEEP_MARGIN: Duration = Duration::from_millis(1);

/// Paces simulated cycles against a monotonic clock.
pub(crate) struct Pacer {
    start: Instant,
    origin: u64,
}

impl Pacer {
    pub(crate) fn new(origin: u64) -> Self {
        Self {
            start: Instant::now(),
            origin,
        }
    }

    pub(crate) fn wait(&self, cycles: u64) {
        let target = target_elapsed(cycles.saturating_sub(self.origin));
        let elapsed = self.start.elapsed();
        if target <= elapsed {
            return;
        }
        let remaining = target - elapsed;
        if remaining > SLEEP_MARGIN {
            std::thread::sleep(remaining - SLEEP_MARGIN);
        }
        while self.start.elapsed() < target {
            std::hint::spin_loop();
        }
    }
}

fn target_elapsed(cycles: u64) -> Duration {
    Duration::from_secs_f64(cycles as f64 / CLOCK_HZ)
}

#[cfg(test)]
mod tests {
    use super::{Pacer, target_elapsed};
    use std::time::Duration;

    #[test]
    fn target_matches_sixteen_megahertz() {
        assert_eq!(target_elapsed(16_000_000), Duration::from_secs(1));
        assert_eq!(target_elapsed(8_000_000), Duration::from_millis(500));
    }

    #[test]
    fn wait_before_deadline_is_immediate() {
        let pacer = Pacer::new(0);
        pacer.wait(0);
        pacer.wait(1);
    }
}
