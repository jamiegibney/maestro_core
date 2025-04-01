//! One-pole lowpass filter.

use crate::dsp::Effect;
use crate::prelude::*;

/// Source: https://www.musicdsp.org/en/latest/Effects/169-compressor.html
#[derive(Clone, Debug)]
pub struct OnePoleLowpass {
    a0: f64,
    b1: f64,

    old: f64,

    sample_rate: f64,
}

impl OnePoleLowpass {
    /// Returns a new `OnePoleLowpass` filter with identity coefficients (i.e., the input
    /// is unaltered).
    pub fn new(sample_rate: f64) -> Self {
        OnePoleLowpass {
            a0: 1.0,
            b1: 0.0,
            old: 0.0,
            sample_rate,
        }
    }

    /// Sets the cutoff frequency of the filter in Hz.
    ///
    /// # See also
    ///
    /// [`set_cutoff_time()`](Self::set_cutoff_time)
    /// [`set_cutoff_time_samples()`](Self::set_cutoff_time_samples)
    pub fn set_cutoff_freq(&mut self, freq_hz: f64) {
        let sr = self.sample_rate;
        assert!(freq_hz.is_sign_positive() && freq_hz <= sr / 2.0);

        let c = 2.0 - (TAU * freq_hz / sr).cos();

        self.b1 = (c * c - 1.0).sqrt() - c;
        self.a0 = 1.0 + self.b1;
    }

    /// Sets the cutoff frequency based on a time value in samples. Useful for averaged
    /// level measurement similar to RMS.
    ///
    /// `time_samples` is the time window in samples (`window * sample_rate`), and
    /// `speed` controls the rate of change. `9.0` is a common value for `speed`.
    ///
    /// Based on *Audio Processes by David Creasey*.
    ///
    /// # See also
    ///
    /// [`set_cutoff_time()`](Self::set_cutoff_time)
    /// [`set_cutoff_freq()`](Self::set_cutoff_freq)
    pub fn set_cutoff_time_samples(&mut self, time_samples: f64, speed: f64) {
        let g = speed.powf(-(time_samples.recip()));

        self.a0 = 1.0 - g;
        self.b1 = g;
    }

    /// Sets the cutoff frequency based on a time value in milliseconds. Useful for
    /// averaged level measurement similar to RMS.
    ///
    /// `time_ms` is the time window in milliseconds, and `speed` controls the rate
    /// of change. `9.0` is a common value for `speed`.
    ///
    /// # See also
    ///
    /// [`set_cutoff_time_samples()`](Self::set_cutoff_time_samples)
    /// [`set_cutoff_freq()`](Self::set_cutoff_freq)
    pub fn set_cutoff_time(&mut self, time_ms: f64, speed: f64) {
        let samples = self.sample_rate * time_ms * 0.001;
        self.set_cutoff_time_samples(samples, speed);
    }

    pub fn reset(&mut self) {
        self.old = 0.0;
    }
}

impl Effect for OnePoleLowpass {
    fn process_mono(&mut self, input: f64, _: usize) -> f64 {
        self.old = self.a0 * input - self.b1 * self.old;
        self.old
    }

    fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn get_identifier(&self) -> &str {
        "one_pole_lowpass"
    }
}
