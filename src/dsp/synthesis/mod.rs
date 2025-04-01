//! Module for signal generation.

use super::*;

pub mod basic;
pub mod generator;

pub use basic::*;

pub use generator::Generator;
pub use noise_osc::NoiseOsc;
pub use phasor::Phasor;
pub use sine::SineOsc;

/// A trait for audio generators.
pub trait GeneratorProcessor {
    /// Processes two stereo channels.
    fn process(&mut self) -> (f64, f64);

    /// Sets the frequency for the generator.
    ///
    /// The default implementation of this method is to panic if `freq_hz
    /// <= 0.0` or `sample_rate / 2.0 < freq_hz`.
    fn set_freq(&mut self, freq_hz: f64, sample_rate: f64) {
        assert!(
            0.0 < freq_hz && freq_hz <= sample_rate / 2.0,
            "out-of-range panic for default implementation of set_freq() in the GeneratorProcessor trait"
        );
    }
}
