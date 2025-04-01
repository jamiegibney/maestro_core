//! Ballistics filter, used for dynamics.

use crate::dsp::Effect;
use crate::prelude::*;
use BallisticsLevelType as LT;

#[derive(Clone, Debug)]
pub enum BallisticsLevelType {
    Peak,
    Rms,
}

/// A filter for measuring attack and release ballistics, most useful for envelope
/// following.
///
/// Based on the JUCE implementation.
#[derive(Clone, Debug)]
pub struct BallisticsFilter {
    /// A buffer for storing the last set of output samples.
    y_old: Vec<f64>,

    /// The "constant time envelope" attack level.
    cte_attack: f64,
    /// The "constant time envelope" release level.
    cte_release: f64,

    /// The level calculation type.
    level_type: LT,

    /// The internal sample rate.
    sample_rate: f64,
}

impl BallisticsFilter {
    /// Creates a new `BallisticsFilter` which can store `num_channels` samples.
    pub fn new(num_channels: usize, sample_rate: f64) -> Self {
        Self {
            y_old: vec![0.0; num_channels],

            cte_attack: 0.0,
            cte_release: 0.0,

            level_type: LT::Peak,
            sample_rate,
        }
    }

    /// Resets the internal buffer to `0.0`.
    pub fn reset(&mut self, value: f64) {
        self.y_old.iter_mut().for_each(|x| *x = value);
    }

    /// Sets the attack time of the filter in milliseconds.
    ///
    /// Values less than `0.001` ms (`1.0` µs) are automatically snapped to `0.0`.
    pub fn set_attack_time_ms(&mut self, time_ms: f64) {
        assert!(time_ms.is_sign_positive());
        self.cte_attack = self.calculate_cte(time_ms);
    }

    /// Sets the release time of the filter in milliseconds.
    ///
    /// Values less than `0.001` ms (`1.0` µs) are automatically snapped to `0.0`.
    pub fn set_release_time_ms(&mut self, time_ms: f64) {
        assert!(time_ms.is_sign_positive());
        self.cte_release = self.calculate_cte(time_ms);
    }

    /// Sets the level calculation type for the filter to use (either `Peak` or `RMS`
    /// values).
    ///
    /// Both types yield positive values, but `RMS` may give more weight to larger
    /// input values. It is, however, more expensive due to the squaring and square
    /// root calculation needed for each sample.
    pub fn set_level_type(&mut self, level_calculation_type: BallisticsLevelType) {
        self.level_type = level_calculation_type;
    }

    /// Sets the number of channels for the filter to store internally. Only one sample
    /// is stored per channel.
    ///
    /// # Safety
    ///
    /// This function may reallocate, so should not be used in a real-time context.
    pub fn set_num_channels(&mut self, num_channels: usize) {
        self.y_old.resize(num_channels, 0.0);
    }

    /// Calculates the constant time envelope ("CTE") value for the given period.
    ///
    /// Values less than `0.001` ms (`1.0` µs) are automatically snapped to `0.0`.
    fn calculate_cte(&self, time_ms: f64) -> f64 {
        if time_ms < 0.001 {
            0.0
        } else {
            ((-TAU * 1000.0 / self.sample_rate) / time_ms).exp()
        }
    }
}

impl Effect for BallisticsFilter {
    fn process_stereo(&mut self, mut in_l: f64, mut in_r: f64) -> (f64, f64) {
        const CH_L: usize = 0;
        const CH_R: usize = 1;

        // ready the input sample based on the type of calculation
        match self.level_type {
            LT::Peak => {
                // peak measurement does not enforce positive values, so abs is used
                in_l = in_l.abs();
                in_r = in_r.abs();
            }
            LT::Rms => {
                // squaring these values ensures they are positive
                in_l *= in_l;
                in_r *= in_r;
            }
        };

        // obtain the correct CTE values
        let cte_l = if in_l > self.y_old[CH_L] {
            self.cte_attack
        } else {
            self.cte_release
        };
        let cte_r = if in_r > self.y_old[CH_R] {
            self.cte_attack
        } else {
            self.cte_release
        };

        // process the samples
        let out_l = in_l + cte_l * (self.y_old[CH_L] - in_l);
        let out_r = in_r + cte_r * (self.y_old[CH_R] - in_r);

        // store them for the next call
        self.y_old[CH_L] = out_l;
        self.y_old[CH_R] = out_r;

        // output the correct sample values
        match self.level_type {
            LT::Peak => (out_l, out_r),
            LT::Rms => (out_l.sqrt(), out_r.sqrt()),
        }
    }

    fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn get_identifier(&self) -> &str {
        "ballistics_filter"
    }
}
