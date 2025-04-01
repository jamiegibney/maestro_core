//! A phasor (saw wave) generator.

use super::*;

/// Non-anti-aliased phasor generator (AKA saw or sawtooth wave oscillator).
#[derive(Debug, Clone, Copy)]
pub struct Phasor {
    /// The phase value.
    pub phase: f64,
    /// The phase increment based on the current frequency.
    pub phase_increment: f64,
}

impl Phasor {
    pub fn new(freq_hz: f64, sample_rate: f64) -> Self {
        debug_assert!(0.0 < freq_hz && freq_hz <= sample_rate / 2.0);
        let phase_increment = freq_hz / sample_rate;

        Self {
            phase: 0.0,
            phase_increment,
        }
    }

    /// Produces the next sample value, useful for types which use `Phasor` to
    /// track their own phase.
    #[allow(clippy::should_implement_trait)] // hush clippy
    pub fn next(&mut self) -> f64 {
        let out = self.phase.mul_add(2.0, -1.0);
        self.increment_phase();
        out
    }

    /// Increments the phase of the `Phasor` by one sample.
    pub fn increment_phase(&mut self) {
        self.phase += self.phase_increment;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }

    /// Returns the length of one period at the `Phasor`'s current frequency.
    pub fn period_length_samples(&self) -> f64 {
        self.phase_increment.recip()
    }
}

impl GeneratorProcessor for Phasor {
    /// Processes two identical phasor samples.
    fn process(&mut self) -> (f64, f64) {
        let out = self.next();
        (out, out)
    }

    /// Sets the frequency of the phasor oscillator.
    fn set_freq(&mut self, freq_hz: f64, sample_rate: f64) {
        debug_assert!(0.0 < freq_hz && freq_hz <= sample_rate / 2.0);
        self.phase_increment = freq_hz / sample_rate;
    }
}
