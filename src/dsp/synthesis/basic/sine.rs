//! A sine wave generator.

use super::*;
use std::f64::consts::TAU;

/// Basic non-anti-aliased sine wave oscillator.
#[derive(Debug, Clone, Copy)]
pub struct SineOsc {
    phase: f64,
    phase_increment: f64,
}

impl SineOsc {
    pub fn new(freq_hz: f64, sample_rate: f64) -> Self {
        debug_assert!(0.0 < freq_hz && freq_hz <= sample_rate / 2.0);
        let phase_increment = freq_hz / sample_rate * TAU;

        Self {
            phase: 0.0,
            phase_increment,
        }
    }

    fn increment_phase(&mut self) {
        self.phase += self.phase_increment;

        if self.phase >= TAU {
            self.phase -= TAU;
        }
    }
}

impl GeneratorProcessor for SineOsc {
    /// Produces two identical sine wave samples.
    fn process(&mut self) -> (f64, f64) {
        let out = self.phase.sin();

        self.increment_phase();

        (out, out)
    }

    /// Sets the frequency of the sine wave oscillator.
    fn set_freq(&mut self, freq_hz: f64, sample_rate: f64) {
        debug_assert!(0.0 < freq_hz && freq_hz <= sample_rate / 2.0);
        self.phase_increment = freq_hz / sample_rate * TAU;
    }
}
