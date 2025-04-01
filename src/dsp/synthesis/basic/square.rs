//! Square wave generator.

use super::*;

/// Basic non-antialiased square wave oscillator.
#[derive(Debug, Clone, Copy)]
pub struct SquareOsc {
    phase: f64,
    phase_increment: f64,
}

impl SquareOsc {
    pub fn new(freq_hz: f64, sample_rate: f64) -> Self {
        debug_assert!(0.0 < freq_hz && freq_hz <= sample_rate / 2.0);

        Self {
            phase: 0.0,
            phase_increment: freq_hz / sample_rate,
        }
    }

    fn increment_phase(&mut self) {
        self.phase += self.phase_increment;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }
}

impl GeneratorProcessor for SquareOsc {
    /// Creates two, identical square wave samples.
    fn process(&mut self) -> (f64, f64) {
        self.increment_phase();
        let out = if self.phase < 0.5 { 1.0 } else { -1.0 };

        (out, out)
    }

    /// Sets the frequency of the square wave oscillator.
    fn set_freq(&mut self, freq_hz: f64, sample_rate: f64) {
        self.phase_increment = freq_hz / sample_rate;
    }
}
