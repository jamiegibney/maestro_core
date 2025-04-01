//! DC-offset filter.

use super::*;
use crate::dsp::{BiquadFilter, Effect};
use crate::prelude::*;

/// A wrapper around a highpass [`FirstOrderFilter`](crate::dsp::FirstOrderFilter)
/// tailored for filtering out 0.0 Hz (DC) signals.
#[derive(Clone, Default, Debug)]
pub struct DCFilter {
    filters: Vec<BiquadFilter>,
}

impl DCFilter {
    /// # Panics
    ///
    /// Panics if `num_filters == 0` or if `sample_rate` is negative.
    pub fn new(sample_rate: f64, num_filters: usize) -> Self {
        assert_ne!(num_filters, 0);
        assert!(sample_rate.is_sign_positive());

        Self {
            filters: vec![Self::create_filter(sample_rate); num_filters],
        }
    }

    /// # Panics
    ///
    /// Panics if `sample rate` is negative.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        assert!(sample_rate.is_sign_positive());

        self.filters
            .iter_mut()
            .for_each(|fil| fil.reset_sample_rate(sample_rate));
    }

    /// # Panics
    ///
    /// Panics if `num_filters == 0`.
    pub fn set_num_filters(&mut self, num_filters: usize) {
        assert_ne!(num_filters, 0);

        // safety: there will always be at least one filter in the vector
        let sr = self.filters[0].get_sample_rate();

        self.filters.resize(num_filters, Self::create_filter(sr));
    }

    fn create_filter(sample_rate: f64) -> BiquadFilter {
        let mut filter = BiquadFilter::new(sample_rate);
        filter.set_q(BUTTERWORTH_Q);
        filter.set_type(FilterType::Highpass);
        filter.set_freq(20.0);

        filter
    }
}

impl Effect for DCFilter {
    fn process_mono(&mut self, mut input: f64, _: usize) -> f64 {
        for filter in &mut self.filters {
            input = filter.process(input);
        }

        input
    }

    fn get_sample_rate(&self) -> f64 {
        self.filters
            .first()
            .expect("expected to have a filter present")
            .get_sample_rate()
    }

    fn get_identifier(&self) -> &str {
        "dc_filter"
    }
}
