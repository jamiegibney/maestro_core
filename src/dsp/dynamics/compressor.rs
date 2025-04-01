//! Module for signal compression (dynamics compression).

use super::*;

const DEFAULT_ATTACK_TIME_MS: f64 = 100.0;
const DEFAULT_RELEASE_TIME_MS: f64 = 100.0;

/// A simple dynamics compressor. Supports a variable knee width,
/// attack and release times, and ratio.
#[derive(Clone, Debug)]
pub struct Compressor {
    sample_rate: f64,

    threshold_db: f64,

    knee_width: f64,
    ratio: f64,

    envelope_filter: BallisticsFilter,
}

impl Compressor {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sample_rate,

            threshold_db: 0.0,

            knee_width: 0.0,

            ratio: 1.0,

            envelope_filter: BallisticsFilter::new(NUM_CHANNELS, sample_rate),
        }
    }

    /// Sets the compressor's threshold in decibels.
    ///
    /// # Panics
    ///
    /// Panics if `level_db` is greater than `0.0`.
    pub fn set_threshold_level_db(&mut self, level_db: f64) {
        debug_assert!(level_db <= 0.0);

        self.threshold_db = level_db;
    }

    /// Sets the ratio of the compressor. Any values over `100.0` are clamped to `100.0`.
    ///
    /// # Panics
    ///
    /// Panics if `ratio` is less than `1.0`.
    pub fn set_ratio(&mut self, mut ratio: f64) {
        debug_assert!(ratio >= 1.0);
        ratio = ratio.clamp(1.0, 100.0);

        self.ratio = ratio;
    }

    /// Sets the compressor's knee width.
    ///
    /// # Panics
    ///
    /// Panics if `width` is negative.
    pub fn set_knee_width(&mut self, width: f64) {
        debug_assert!(width.is_sign_positive());

        self.knee_width = width;
    }

    /// Sets the compressor's attack time in milliseconds.
    pub fn set_attack_time_ms(&mut self, time_ms: f64) {
        self.envelope_filter.set_attack_time_ms(time_ms);
    }

    /// Sets the compressor's release time in milliseconds.
    pub fn set_release_time_ms(&mut self, time_ms: f64) {
        self.envelope_filter.set_release_time_ms(time_ms);
    }

    /// Sets whether to use RMS level in the envelope calculation.
    pub fn use_rms(&mut self, use_rms: bool) {
        if use_rms {
            self.envelope_filter
                .set_level_type(BallisticsLevelType::Rms);
        }
        else {
            self.envelope_filter
                .set_level_type(BallisticsLevelType::Peak);
        }
    }

    /// Standard compression gain function with a rounded knee and, otherwise,
    /// a linear profile. This represents the *amount of gain to apply* for a
    /// given envelope level, not a scale.
    ///
    /// This function may be used to find the compressor's transfer function,
    /// which may be useful if you wish to draw it, for example.
    ///
    /// From *Audio Processes by David Creasey*.
    pub fn gain_function(&self, input: f64) -> f64 {
        let Self { threshold_db: thresh, knee_width: width, ratio, .. } = self;
        let half_width = width / 2.0;

        // below the knee
        if input <= (thresh - half_width) {
            0.0
        }
        // within the knee
        else if (thresh - half_width) < input
            && input <= (thresh + half_width)
        {
            (2.0 * width).recip()
                * (ratio.recip() - 1.0)
                * (input - thresh + half_width).powi(2)
        }
        // above the knee
        else {
            (ratio.recip() - 1.0) * (input - thresh)
        }
    }
}

impl Effect for Compressor {
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let (env_l, env_r) = self.envelope_filter.process_stereo(in_l, in_r);

        let gain_l = db_to_level(self.gain_function(level_to_db(env_l)));
        let gain_r = db_to_level(self.gain_function(level_to_db(env_r)));

        (gain_l * in_l, gain_r * in_r)
    }

    fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }
    
    fn get_identifier(&self) -> &str {
        "compressor"
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self::new(unsafe { SAMPLE_RATE })
    }
}
