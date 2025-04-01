//! Infinite impulse response comb filter.

use super::filter::CombFilter;
use crate::dsp::Filter;
use crate::prelude::*;

/// A IIR (Infinite Impulse Response) comb filter.
///
/// Supports frequencies as low as 10 Hz.
#[derive(Clone, Default)]
pub struct IirCombFilter {
    filter: CombFilter,
    internal_filters: Vec<Box<dyn Filter>>,
}

impl Filter for IirCombFilter {
    /// Processes a single sample of the comb filter, returning the new sample.
    fn process(&mut self, mut sample: f64) -> f64 {
        sample *= self.filter.a0;
        let mut output = self.filter.buffer.read().mul_add(self.filter.bd, sample);

        for filter in &mut self.internal_filters {
            output = filter.process(output);
        }

        self.filter.buffer.push(output);
        output
    }
}

impl IirCombFilter {
    /// Creates a new, initialized filter with an internal buffer holding
    /// one second of samples.
    pub fn with_interpolation(interpolation: bool, sample_rate: f64) -> Self {
        Self {
            filter: CombFilter::new(interpolation, sample_rate),
            internal_filters: vec![],
        }
    }

    /// Use this if you change the sample rate to reallocate the internal buffer.
    pub fn reset_sample_rate(&mut self, sample_rate: f64) {
        self.filter.reset_sample_rate(sample_rate);
    }

    /// Sets the frequency of the comb filter. Must be between 10 Hz and half
    /// the sample rate.
    ///
    /// # Panics
    ///
    /// Panics if `freq` is less than 10 or greater than half of the sample rate.
    pub fn set_freq(&mut self, freq: f64) {
        self.filter.set_freq(freq);
    }

    /// Sets the gain of the comb filter.
    ///
    /// # Panics
    ///
    /// Panics if the gain is greater than 0 dB.
    pub fn set_gain_db(&mut self, gain_db: f64) {
        self.filter.set_gain_db(gain_db);

        let level = db_to_level(gain_db);
        let polarity = if self.filter.positive_polarity {
            1.0
        } else {
            -1.0
        };
        self.filter.bd = level * polarity;
        self.filter.a0 = 1.0 - self.filter.bd.abs();
    }

    /// Sets the polarity of the comb filter.
    pub fn set_positive_polarity(&mut self, polarity_should_be_positive: bool) {
        self.filter
            .set_positive_polarity(polarity_should_be_positive);
    }

    /// Sets whether the comb filter should interpolate between samples.
    pub fn set_interpolation(&mut self, interpolation_type: InterpType) {
        self.filter.set_interpolation(interpolation_type);
    }

    /// Moves `filters` into the comb filter, which will process them in its
    /// `process()` method. The state of the filters you pass into the comb filter
    /// should be set before moving them into this method.
    ///
    /// The processing chain follows the order of this vector, i.e. element `0`
    /// is processed first.
    pub fn set_internal_filters(&mut self, filters: Vec<Box<dyn Filter>>) {
        self.internal_filters = filters;
    }
}
