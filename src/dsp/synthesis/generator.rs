//! Generic enum for audio generators.

use super::*;

/// All the types of signal generators available.
#[derive(Debug, Clone, Copy)]
pub enum Generator {
    /// A basic sine wave generator.
    Sine(SineOsc),
    /// A basic triangle wave generator.
    Tri(TriOsc),
    /// A basic saw wave generator.
    Saw(Phasor),
    /// A basic square wave generator.
    Square(SquareOsc),
    /// A basic white noise generator.
    Noise,
}

impl Generator {
    pub fn process(&mut self) -> (f64, f64) {
        match self {
            Self::Sine(gen) => gen.process(),
            Self::Tri(gen) => gen.process(),
            Self::Saw(gen) => gen.process(),
            Self::Square(gen) => gen.process(),
            Self::Noise => (NoiseOsc::process(), NoiseOsc::process()),
        }
    }

    pub fn change_freq(&mut self, freq_hz: f64, sample_rate: f64) {
        match self {
            Self::Sine(gen) => gen.set_freq(freq_hz, sample_rate),
            Self::Tri(gen) => gen.set_freq(freq_hz, sample_rate),
            Self::Saw(gen) => gen.set_freq(freq_hz, sample_rate),
            Self::Square(gen) => gen.set_freq(freq_hz, sample_rate),
            Self::Noise => {}
        }
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::Sine(SineOsc::new(440.0, unsafe { SAMPLE_RATE }))
    }
}
