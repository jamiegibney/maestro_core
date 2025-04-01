//! Revised biquad filter using the [direct form 1](https://en.wikipedia.org/wiki/Digital_biquad_filter#Transposed_direct_forms:~:text=%5Bedit%5D-,Direct%20form,-1%5Bedit).
//!
//! Coefficient equations taken from the 
//! [Audio EQ Cookbook by Robert Bristow-Johnson](https://www.w3.org/TR/audio-eq-cookbook/).

#![allow(clippy::module_name_repetitions)]
use super::*;
use crate::prelude::*;
use std::f64::consts::{FRAC_1_SQRT_2, PI, TAU};
use util::{db_to_level, level_to_db};
use FilterType as FT;

/// Filter coefficients.
#[derive(Debug, Clone, Copy)]
struct Coefs {
    /// INPUT side
    a0: f64,
    /// INPUT side
    a1: f64,
    /// INPUT side
    a2: f64,
    /// OUTPUT side
    b0: f64,
    /// OUTPUT side
    b1: f64,
    /// OUTPUT side
    b2: f64,
}

impl Coefs {
    /// A coefficient state which leaves the input signal totally unaffected.
    fn identity() -> Self {
        Self { a0: 1.0, a1: 0.0, a2: 0.0, b0: 1.0, b1: 0.0, b2: 0.0 }
    }
}

impl Default for Coefs {
    fn default() -> Self {
        Self::identity()
    }
}

/// A struct which covers the parameters used by biquad filters.
#[derive(Debug, Clone, Copy)]
pub struct BiquadParams {
    pub freq: f64,
    pub gain: f64,
    pub q: f64,
    pub filter_type: FilterType,
}

impl Default for BiquadParams {
    fn default() -> Self {
        Self {
            freq: 1.0,
            q: FRAC_1_SQRT_2,
            filter_type: FT::default(),
            gain: 0.0,
        }
    }
}

/// A biquad filter implementation, which offers all of the filter types
/// available in `FilterType`.
/// This is based on the [transposed direct form 1](https://en.wikipedia.org/wiki/Digital_biquad_filter#Transposed_direct_forms:~:text=%5Bedit%5D-,Direct%20form,-1%5Bedit)
/// implementation.
///
/// Its parameters are stored internally as a `BiquadParams`, which can be passed
/// to the `set_params()` method to mutate the filter's state. There are also
/// separate methods for each of the filter's parameters for finer control.
/// Neither of these options differ in performance.
///
/// # Performance
///
/// The filter will lazily update its coefficients, i.e. only when a parameter
/// is updated (even if the value is the same) will it recompute its coefficients.
/// Consider avoiding unnecessary calls to the `set_q()`, `set_type()`,
/// `set_gain()`, `set_freq()`, `reset_sample_rate()`, or `set_params()`
/// methods where possible to leverage the lazy behaviour.
///
/// Note that certain filter types do not use all parameters which can be
/// passed to the filter. These values are ignored during processing, but
/// updating them will still signal the filter to recompute.
#[derive(Debug, Clone, Default)]
pub struct BiquadFilter {
    coefs: Coefs,
    delayed_in: (f64, f64),
    delayed_out: (f64, f64),

    params: BiquadParams,
    sample_rate: f64,

    needs_recompute: bool,
}

impl Filter for BiquadFilter {
    /// Processes a single sample of the filter and returns the new sample.
    ///
    /// Note that this filter will lazily update its coefficients; if there is
    /// no parameter change between calls to this method, only the sample output
    /// is computed — not the filter coefficients. In other words, this method
    /// will compute much faster if there is no parameter change between calls.
    fn process(&mut self, sample: f64) -> f64 {
        let Coefs { a0, a1, a2, b1, b2, b0 } = self.coefs;

        if self.needs_recompute {
            match self.params.filter_type {
                FT::Peak => self.set_peak_coefs(),
                FT::Lowpass => self.set_lowpass_coefs(),
                FT::Highpass => self.set_highpass_coefs(),
                FT::Lowshelf => self.set_lowshelf_coefs(),
                FT::Highshelf => self.set_highshelf_coefs(),
                FT::Bandpass => self.set_bandpass_coefs(),
                FT::Notch => self.set_notch_coefs(),
                FT::Allpass => self.set_allpass_coefs(),
            };

            self.needs_recompute = false;
        }

        let bottom_sum = self.delayed_in.1 * b2 + self.delayed_out.1 * -a2;
        let middle_sum = self.delayed_in.0 * b1 + self.delayed_out.0 * -a1;
        let output = bottom_sum + middle_sum + (sample * b0);

        self.delayed_in = (sample, self.delayed_in.0);
        self.delayed_out = (output, self.delayed_out.0);

        // let output = a0.mul_add(sample, z1);
        //
        // self.delayed = (
        //     a1.mul_add(sample, output * -b1) + z2,
        //     a2.mul_add(sample, output * -b2),
        // );
        //
        output
    }
}

// NOTE: the mul_add() method is used a lot here as it may improve performance on
// some systems and only involves one rounding error. as the majority of the code
// for computing filter coefficients is designed to focus on correctness and
// performance, I opted to use mul_add() at the cost of some readability.

impl BiquadFilter {
    /// Creates a new, initialised `Filter`, set to the default `Peak` filter type.
    #[must_use]
    pub fn new(sample_rate: f64) -> Self {
        Self { sample_rate, ..Self::default() }
    }

    /// "Suspends" the filter, leaving any processed signal totally unaltered.
    ///
    /// See `force_recompute()` if you need to "resume" the filter's processing
    /// after a call to this method.
    ///
    /// Alternatively, adjusting any of the filter parameters via the `set_q()`,
    /// `set_type()`, `set_gain()`, `set_freq()`, `reset_sample_rate()`, or
    /// `set_params()` methods will also "resume" the filter's processing.
    ///
    /// Note that this function does not alter its filter parameters.
    ///
    /// Note that this function acts instantaneously, and does not attempt to
    /// prevent clicking or signal discontinuities.
    pub fn suspend(&mut self) {
        self.coefs = Coefs::identity();
        self.needs_recompute = false;
    }

    /// Resets the sample rate of the filter.
    ///
    /// # Safety
    ///
    /// This can be called whilst the filter is actively processing and its
    /// coefficients will update upon the next call to the `process()` method,
    /// but the audio output is not guaranteed to be safe.
    pub fn reset_sample_rate(&mut self, new_sample_rate: f64) {
        self.sample_rate = new_sample_rate;
        self.needs_recompute = true;
    }

    /// Forces the filter to recompute its coefficients on the next call of the
    /// `process()` method.
    ///
    /// This can be used to "resume" the filter's processing after a call to
    /// `suspend()`.
    pub fn force_recompute(&mut self) {
        self.needs_recompute = true;
    }

    /// Sets the parameters of the filter all at once.
    ///
    /// # Panics
    ///
    /// This function will panic in debug mode if the parameter's filter
    /// and/or q value is negative, or if the frequency is over half of
    /// the sample rate.
    pub fn set_params(&mut self, params: &BiquadParams) {
        self.params = *params;
        self.needs_recompute = true;
        self.assertions();
    }

    /// Sets the frequency of the filter.
    ///
    /// # Panics
    ///
    /// This function will panic in debug mode if `freq` is negative.
    /// It will also panic in debug mode if the frequency is over half of
    /// the sample rate.
    pub fn set_freq(&mut self, freq: f64) {
        self.params.freq = freq;
        self.needs_recompute = true;
        self.assertions();
    }

    /// Sets the gain of the filter.
    pub fn set_gain(&mut self, gain: f64) {
        self.params.gain = gain;
        self.needs_recompute = true;
        self.assertions();
    }

    /// Sets the Q of the filter.
    ///
    /// # Panics
    /// This function will panic in debug mode if `Q` is negative.
    pub fn set_q(&mut self, q: f64) {
        self.params.q = q;
        self.needs_recompute = true;
        self.assertions();
    }

    /// Sets the filter type of the filter.
    ///
    /// # Note
    ///
    /// Note that the shelving filters are not yet implemented.
    pub fn set_type(&mut self, filter_type: FilterType) {
        self.params.filter_type = filter_type;
        self.needs_recompute = true;
        self.assertions();
    }

    /// Returns the half-power points (-3 dB gain) of the bandpass/notch filter.
    ///
    /// # Panics
    ///
    /// This function will panic if the current filter type is not `Bandpass` or `Notch`.
    pub fn bp_notch_half_power_points(&self) -> (f64, f64) {
        debug_assert!(matches!(
            self.params.filter_type,
            FT::Notch | FT::Bandpass
        ));
        let BiquadParams { freq, q, .. } = self.params;
        let f_min =
            (freq / (2.0 * q)) * (4.0f64.mul_add(q.powi(2), 1.0).sqrt() - 1.0);
        let f_max = f_min + (freq / q);

        (f_min, f_max)
    }

    /// Returns the bandwidth of the bandpass/notch filter.
    ///
    /// # Panics
    ///
    /// This function will panic if the current filter type is not `Bandpass` or `Notch`.
    pub fn bp_notch_bandwidth(&self) -> f64 {
        debug_assert!(matches!(
            self.params.filter_type,
            FT::Notch | FT::Bandpass
        ));

        self.params.freq / self.params.q
    }

    pub fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }

    /// Returns the magnitude response at `freq` Hz in decibels.
    ///
    /// Method found in the comments at <https://www.musicdsp.org/en/latest/Analysis/186-frequency-response-from-biquad-coefficients.html>.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `freq <= 0.0 || freq > sample_rate / 2`.
    pub fn response_at(&self, freq: f64) -> f64 {
        // debug_assert!(0.0 < freq && freq <= self.sample_rate / 2.0);
        let Coefs { a1, a2, b0, b1, b2, .. } = self.coefs;
        // I realised that, as I'm already normalising all coefficients by a0
        // (for low/high pass/shelves), that it needs to be 1.0 here as it sort of
        // normalises itself.
        let a0 = 1.0;

        // sin²(w / 2)
        let phi = (freq * 0.5).sin().powi(2);

        // the part with the b coefficients
        let a = (b0 + b1 + b2).powi(2);
        let b = 4.0 * b1.mul_add(b2, b0.mul_add(b1, 4.0 * b0 * b2)) * phi;
        let c = 16.0 * b0 * b2 * phi.powi(2);
        let b_part = 10.0 * (a - b + c).log10();

        // the part with the a coefficients
        let a = (a0 + a1 + a2).powi(2);
        let b = 4.0 * a1.mul_add(a2, a0.mul_add(a1, 4.0 * a0 * a2)) * phi;
        let c = 16.0 * a0 * a2 * phi.powi(2);
        let a_part = 10.0 * (a - b + c).log10();

        b_part - a_part
    }

    /* PRIVATE METHODS */

    /// Sets the filter coefficients for a peak filter.
    fn set_peak_coefs(&mut self) {
        let phi = self.get_phi();
        let alpha = self.get_alpha(phi);

        let Coefs { a0, a1, a2, b1, b2, b0 } = &mut self.coefs;
        let BiquadParams { freq, gain, q, filter_type } = self.params;
        let sr = &self.sample_rate;

        let amp = 10.0f64.powf(gain / 40.0);
        let cos_phi = phi.cos();

        *a0 = 1.0 + alpha / amp;

        *b0 = (1.0 + alpha * amp) / *a0;
        *b1 = (-2.0 * cos_phi) / *a0;
        *b2 = (1.0 - alpha * amp) / *a0; 
        *a1 = *b1;
        *a2 = (1.0 - alpha / amp) / *a0;
    }

    /// Sets the filter coefficients for a lowpass filter.
    fn set_lowpass_coefs(&mut self) {
        let phi = self.get_phi();
        let alpha = self.get_alpha(phi);
        let cos_phi = phi.cos();

        let Coefs { a0, a1, a2, b1, b2, b0 } = &mut self.coefs;
        let BiquadParams { freq, q, .. } = self.params;
        let sr = self.sample_rate;

        *a0 = 1.0 + alpha;

        *b0 = ((1.0 - cos_phi) * 0.5) / *a0;
        *b1 = (1.0 - cos_phi) / *a0;
        *b2 = *b0;

        *a1 = (-2.0 * cos_phi) / *a0;
        *a2 = (1.0 - alpha) / *a0;
    }

    /// Sets the filter coefficients for a highpass filter.
    fn set_highpass_coefs(&mut self) {
        let phi = self.get_phi();
        let alpha = self.get_alpha(phi);
        let cos_phi = phi.cos();

        let Coefs { a0, a1, a2, b1, b2, b0 } = &mut self.coefs;
        let BiquadParams { freq, q, .. } = self.params;
        let sr = self.sample_rate;

        *a0 = 1.0 + alpha;

        *b0 = ((1.0 + cos_phi) * 0.5) / *a0;
        *b1 = (-(1.0 + cos_phi)) / *a0;
        *b2 = *b0;

        *a1 = (-2.0 * cos_phi) / *a0;
        *a2 = (1.0 - alpha) / *a0;
    }

    /// Sets the filter coefficients for a lowshelf filter.
    fn set_lowshelf_coefs(&mut self) {
        let phi = self.get_phi();
        let alpha = self.get_alpha(phi);
        let cos_phi = phi.cos();

        let Coefs { a0, a1, a2, b1, b2, b0 } = &mut self.coefs;
        let BiquadParams { freq, q, gain, .. } = self.params;
        let sr = self.sample_rate;

        let amp = 10.0f64.powf(gain / 40.0);
        let root_amp_2 = 2.0 * amp.sqrt() * alpha;

        *a0 = (amp + 1.0) + (amp - 1.0) * cos_phi + root_amp_2;

        *b0 = amp * ((amp + 1.0) - (amp - 1.0) * cos_phi + root_amp_2) / *a0;
        *b1 = 2.0 * amp * ((amp - 1.0) - (amp + 1.0) * cos_phi) / *a0;
        *b2 = amp * ((amp + 1.0) - (amp - 1.0) * cos_phi - root_amp_2) / *a0;

        *a1 = -2.0 * ((amp - 1.0) + (amp + 1.0) * cos_phi) / *a0;
        *a2 = ((amp + 1.0) + (amp - 1.0) * cos_phi - root_amp_2) / *a0;
    }

    /// Sets the filter coefficients for a highshelf filter.
    fn set_highshelf_coefs(&mut self) {
        let phi = self.get_phi();
        let alpha = self.get_alpha(phi);
        let cos_phi = phi.cos();

        let Coefs { a0, a1, a2, b1, b2, b0 } = &mut self.coefs;
        let BiquadParams { freq, q, gain, .. } = self.params;
        let sr = self.sample_rate;

        let amp = 10.0f64.powf(gain / 40.0);
        let ra_2a = 2.0 * amp.sqrt() * alpha;

        *a0 = (amp + 1.0) - (amp - 1.0) * cos_phi + ra_2a;

        *b0 = (amp * ((amp + 1.0) + (amp - 1.0) * cos_phi + ra_2a)) / *a0;
        *b1 = (-2.0 * amp * ((amp - 1.0) + (amp + 1.0) * cos_phi)) / *a0;
        *b2 = (amp * ((amp + 1.0) + (amp - 1.0) * cos_phi - ra_2a)) / *a0;

        *a1 = (2.0 * ((amp - 1.0) - (amp + 1.0) * cos_phi)) / *a0;
        *a2 = ((amp + 1.0) - (amp - 1.0) * cos_phi - ra_2a) / *a0;
    }

    /// Sets the filter coefficients for a bandpass filter.
    fn set_bandpass_coefs(&mut self) {
        self.bandpass_notch_b_coefs();
        let Coefs { a0, a1, a2, b1, b2, .. } = &mut self.coefs;

        *a0 = (1.0 - *b2) / 2.0;
        *a1 = 0.0;
        *a2 = -(*a0);
    }

    /// Sets the filter coefficients for a notch filter.
    fn set_notch_coefs(&mut self) {
        self.bandpass_notch_b_coefs();
        let Coefs { a0, a1, a2, b1, b2, .. } = &mut self.coefs;

        *a0 = (1.0 + *b2) / 2.0;
        *a1 = *b1;
        *a2 = *a0;
    }

    /// Sets common coefficients for notch and bandpass designs.
    fn bandpass_notch_b_coefs(&mut self) {
        let Coefs { a0, a1, a2, b1, b2, .. } = &mut self.coefs;
        let BiquadParams { freq, q, .. } = self.params;
        let sr = &self.sample_rate;

        let phi = (TAU * freq) / sr;

        *b2 = (PI / 4.0) - (phi / (2.0 * q)).tan();
        *b1 = -(1.0 + *b2) * phi.cos();
    }

    /// Sets the filter coefficients for a allpass filter.
    fn set_allpass_coefs(&mut self) {
        let Coefs { a0, a1, a2, b1, b2, .. } = &mut self.coefs;
        let BiquadParams { freq, q, .. } = self.params;
        let sr = &self.sample_rate;

        let phi = (TAU * freq) / sr;

        *b2 = (phi / 2.0).mul_add(-q, PI / 4.0);
        *b1 = -(1.0 + *b2) * phi.cos();
        *a0 = *b2;
        *a1 = *b1;
        *a2 = 1.0;
    }

    /// Convenience method for obtaining the value of "phi".
    fn get_phi(&self) -> f64 {
        TAU * (self.params.freq / self.sample_rate)
    }

    /// Convenience method for obtaining the value of "alpha".
    fn get_alpha(&self, phi: f64) -> f64 {
        (phi.sin()) / 2.0 * self.params.q
    }

    /// Debug assertions used whenever a parameter is changed.
    fn assertions(&self) {
        let BiquadParams { freq, q, filter_type, .. } = self.params;
        let sr = self.sample_rate;

        // general assertions
        debug_assert!(
            freq.is_sign_positive() && q.is_sign_positive() && freq <= sr / 2.0
        );
    }
}
