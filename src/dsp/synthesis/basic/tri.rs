//! A triangle wave generator.

use super::Phasor;
use super::*;

/// Basic non-antialiased triangle wave oscillator.
///
/// [Formula source](https://www.desmos.com/calculator/dzdtwqrnto)
#[derive(Debug, Clone, Copy)]
pub struct TriOsc {
    phasor: Phasor,
}

impl TriOsc {
    pub fn new(freq_hz: f64, sample_rate: f64) -> Self {
        Self { phasor: Phasor::new(freq_hz, sample_rate) }
    }
}

impl GeneratorProcessor for TriOsc {
    /// Creates two, identical triangle wave samples.
    fn process(&mut self) -> (f64, f64) {
        let p = self.phasor.period_length_samples();
        let x = self.phasor.next();

        let out = (x.abs() - 0.5) * 2.0;

        (out, out)
    }

    /// Sets the frequency of the triangle wave oscillator.
    fn set_freq(&mut self, freq_hz: f64, sample_rate: f64) {
        self.phasor.set_freq(freq_hz, sample_rate);
    }
}
