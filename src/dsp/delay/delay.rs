//! Audio delay tap.

use super::{Effect, RingBuffer};
use crate::prelude::*;

pub const DEFAULT_DELAY_SMOOTHING: SmoothingType = SmoothingType::Cosine;

/// An audio delay tap.
///
/// This is essentially a wrapper around a [`RingBuffer`](super::RingBuffer)
/// which abstracts away some parameters, such as interpolation and smoothing.
#[derive(Clone, Debug, Default)]
pub struct Delay {
    buffer: RingBuffer,
    feedback_amount: f64,
}

impl Delay {
    /// Creates a new `Delay` which can provide up to
    pub fn new(max_delay_time_secs: f64, sample_rate: f64) -> Self {
        let size_samples = (max_delay_time_secs * sample_rate) as usize;

        Self {
            buffer: RingBuffer::new(size_samples, sample_rate)
                .with_interpolation(InterpType::DefaultCubic)
                .with_smoothing(DEFAULT_DELAY_SMOOTHING, 0.1),
            feedback_amount: 0.0,
        }
    }

    /// Returns `Self` with a delay time of `delay_secs`.
    pub fn with_delay_time(mut self, delay_secs: f64) -> Self {
        self.set_delay_time(delay_secs);
        self
    }

    /// Returns `Self` with a delay time of `delay_samples`.
    pub fn with_delay_time_samples(mut self, delay_samples: f64) -> Self {
        self.set_delay_time_samples(delay_samples);
        self
    }

    /// Sets the delay time of the tap in seconds.
    ///
    /// # Panics
    ///
    /// Panics if `delay_secs` is greater than the maximum delay time possible
    /// with the internal buffer. Query the maximum delay time with the
    /// [`max_delay_time_secs()`](Self::max_delay_time_secs) method.
    pub fn set_delay_time(&mut self, delay_secs: f64) {
        assert!(delay_secs <= self.buffer.max_delay_secs());
        self.buffer.set_delay_time(delay_secs);
    }

    /// Sets the delay time of the tap in samples.
    ///
    /// # Panics
    ///
    /// Panics if `delay_samples` is greater than the maximum delay time
    /// possible with the internal buffer.
    pub fn set_delay_time_samples(&mut self, delay_samples: f64) {
        let sr = self.buffer.get_sample_rate();
        self.set_delay_time(delay_samples / sr);
    }

    pub fn set_feedback_amount(&mut self, feedback: f64) {
        self.feedback_amount = feedback.clamp(0.0, 1.0);
    }

    /// Returns the maximum delay time of the `Delay` in seconds — accounting
    /// for the sample rate.
    pub fn max_delay_time_secs(&self) -> f64 {
        self.buffer.max_delay_secs()
    }

    /// Sets the time it takes for the internal delay time to reach the requested target.
    pub fn set_smoothing_time(&mut self, smoothing_time_secs: f64) {
        self.buffer
            .set_smoothing(DEFAULT_DELAY_SMOOTHING, smoothing_time_secs);
    }

    /// Sets a new sample rate for the `Delay`.
    ///
    /// # Panics
    ///
    /// Panics if `sample_rate` is negative.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.buffer.set_sample_rate(sample_rate);
    }
}

impl Effect for Delay {
    fn process_mono(&mut self, input: f64, _: usize) -> f64 {
        let output = self.buffer.read();
        self.buffer
            .push(output.mul_add(self.feedback_amount, input));

        output
    }

    fn get_sample_rate(&self) -> f64 {
        self.buffer.get_sample_rate()
    }

    fn get_identifier(&self) -> &str {
        "delay"
    }
}
