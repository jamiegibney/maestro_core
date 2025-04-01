//! Two-pole resonator filter.

use super::*;
use crate::{dsp::Effect, prelude::*};

#[derive(Clone, Debug)]
struct Coefs {
    b0: f64,
    a1: f64,
    a2: f64,
}

impl Coefs {
    pub fn identity() -> Self {
        Self {
            b0: 1.0,
            a1: 0.0,
            a2: 0.0,
        }
    }
}

/// A two-pole resonator.
///
/// [`Source`](https://www.dsprelated.com/freebooks/filters/Two_Pole.html)
#[derive(Clone, Debug)]
pub struct TwoPoleResonator {
    resonance: f64,
    coefs: Coefs,
    z1: f64,
    z2: f64,

    sample_rate: f64,
}

impl TwoPoleResonator {
    /// Creates a new filter with identity coefficients (i.e., any input is left
    /// unaltered).
    ///
    /// # Panics
    ///
    /// Panics if `sample_rate` is negative.
    pub fn new(sample_rate: f64) -> Self {
        assert!(sample_rate.is_sign_positive());
        Self {
            resonance: 0.0,
            coefs: Coefs::identity(),
            z1: 0.0,
            z2: 0.0,
            sample_rate,
        }
    }

    /// Sets the cutoff frequency of the filter in Hz.
    ///
    /// # Panics
    ///
    /// Panics if `cutoff_hz` is negative, or greater than half the internal sample
    /// rate.
    pub fn set_cutoff(&mut self, cutoff_hz: f64) {
        let sr = self.sample_rate;
        assert!(cutoff_hz.is_sign_positive() && cutoff_hz <= sr / 2.0);

        let theta = (TAU * cutoff_hz) / sr;
        let r = self.resonance;

        self.coefs.a1 = -2.0 * r * theta.cos();
        self.coefs.a2 = r * r;
    }

    /// `resonance` is clamped between `0.0` and `1.0`.
    pub fn set_resonance(&mut self, resonance: f64) {
        // self.resonance = resonance.clamp(0.0, 1.0);
        self.resonance = resonance;
    }

    /// Resets the internal sample rate of the filter.
    ///
    /// # Panics
    ///
    /// Panics if `sample_rate` is negative.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        assert!(sample_rate.is_sign_positive());
        self.sample_rate = sample_rate;
    }

    /// Returns the filter magnitude at `frequency_hz` Hz.
    ///
    /// # Panics
    ///
    /// Panics if `frequency_hz` is negative, or greater than the Nyquist rate.
    pub fn magnitude_response_at(&self, frequency_hz: f64) -> f64 {
        let sr = self.sample_rate;
        assert!(frequency_hz.is_sign_positive() && frequency_hz <= sr / 2.0);
        let Coefs { b0, a1, a2 } = self.coefs;

        let wt = frequency_hz * self.sample_rate.recip();
        let wt2 = wt + wt;

        let p1 = a2.mul_add(wt2.cos(), a1.mul_add(wt.cos(), 1.0));
        let p2 = (-a1).mul_add(wt.sin(), -a2 * wt2.sin());
        let den = p1.hypot(p2);

        b0 / den
    }

    /// Returns the phase response at `frequency_hz` Hz.
    ///
    /// # Panics
    ///
    /// Panics if `frequency_hz` is negative, or greater than the Nyquist rate.
    pub fn phase_response_at(&self, frequency_hz: f64) -> f64 {
        let sr = self.sample_rate;
        assert!(frequency_hz.is_sign_positive() && frequency_hz <= sr / 2.0);
        let Coefs { b0, a1, a2 } = self.coefs;
        debug_assert!(b0 > 0.0);

        let wt = frequency_hz * sr.recip();
        let wt2 = wt * wt;

        let num = (-a1).mul_add(wt.sin(), -a2 * wt2.sin());
        let den = a2.mul_add(wt2.cos(), a1.mul_add(wt.cos(), 1.0));

        -(num / den).atan()
    }

    /// Returns the internal sample rate of the filter.
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}

impl Filter for TwoPoleResonator {
    fn process(&mut self, sample: f64) -> f64 {
        let Coefs { b0, a1, a2 } = self.coefs;

        let output = sample.mul_add(b0, self.z2.mul_add(-a2, self.z1 * -a1));

        self.z2 = self.z1;
        self.z1 = output;

        output
    }
}

impl Effect for TwoPoleResonator {
    fn get_sample_rate(&self) -> f64 {
        self.sample_rate()
    }

    fn process_mono(&mut self, input: f64, _: usize) -> f64 {
        self.process(input)
    }

    fn get_identifier(&self) -> &str {
        "two_pole_resonator"
    }
}

impl Default for TwoPoleResonator {
    fn default() -> Self {
        Self {
            resonance: 0.0,
            coefs: Coefs::identity(),
            z1: 0.0,
            z2: 0.0,
            sample_rate: 0.0,
        }
    }
}
