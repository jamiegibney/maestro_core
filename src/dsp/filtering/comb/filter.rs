//! Base comb filter type.

use super::*;
use crate::util::interp::InterpolationType as InterpType;

#[derive(Debug, Clone, Default)]
pub struct CombFilter {
    // coefficients
    pub a0: f64,
    pub bd: f64,

    pub buffer: RingBuffer,

    pub freq: f64,
    pub gain_db: f64,
    pub positive_polarity: bool,
    pub interpolation: bool,

    pub sample_rate: f64,
}

impl CombFilter {
    #[must_use]
    pub fn new(interpolation: bool, sample_rate: f64) -> Self {
        // allocates 1 second
        let mut buffer = RingBuffer::new(sample_rate as usize, sample_rate);
        buffer.set_smoothing(SmoothingType::Cosine, 0.05);

        Self {
            a0: 1.0,
            bd: 0.0,

            buffer,

            freq: unsafe { TUNING_FREQ_HZ },
            gain_db: MINUS_INFINITY_DB,
            positive_polarity: true,
            interpolation,

            sample_rate,
        }
    }

    pub fn reset_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.buffer.resize(self.sample_rate as usize);
    }

    pub fn set_freq(&mut self, freq: f64) {
        self.freq = freq;
        self.assertions();
        self.set_delay_time();
    }

    pub fn set_gain_db(&mut self, gain: f64) {
        self.gain_db = gain;
        self.assertions();
    }

    pub fn set_positive_polarity(&mut self, polarity_should_be_positive: bool) {
        self.positive_polarity = polarity_should_be_positive;
    }

    pub fn set_interpolation(&mut self, interpolation_type: InterpType) {
        self.buffer.set_interpolation(interpolation_type);
    }

    fn set_delay_time(&mut self) {
        self.buffer.set_delay_time(1.0 / self.freq);
    }

    fn assertions(&self) {
        debug_assert!(
            1.0 <= self.freq && self.freq <= self.sample_rate / 2.0 && self.gain_db <= 0.0,
            "{}",
            self.freq
        );
    }
}
