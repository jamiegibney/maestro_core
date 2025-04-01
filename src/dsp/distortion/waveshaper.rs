//! Signal waveshaping.

use std::ops::RangeInclusive;

use super::*;

/// A waveshaper which dynamically accepts any transfer function and asymmetric
/// drive levels.
///
/// TODO: add asymmetric curve processing (currently only drive is applied
/// asymmetrically).
pub struct Waveshaper {
    curve: f64,
    // curve_lower: f64,
    curve_range: RangeInclusive<f64>,
    // asymmetric_curve: bool,
    drive: f64,
    drive_lower: f64,
    asymmetric: bool,

    xfer_function: Box<dyn Fn(f64, f64) -> f64 + Send>,
}

impl Waveshaper {
    /// Returns a new, initialised waveshaper.
    ///
    /// By default, this uses `xfer::s_curve` as its transfer function, its
    /// curve parameter range is `0.0` to `1.0`, and it operates symmetrically.
    ///
    /// See the `set_xfer_function()` method to provide a custom transfer
    /// function, and `set_curve_range()` to change its curve parameter range.
    #[must_use]
    pub fn new() -> Self {
        Self {
            curve: 0.0,
            // curve_lower: 0.0,
            curve_range: 0.0..=1.0,
            // asymmetric_curve: false,
            drive: 1.0,
            drive_lower: 1.0,
            asymmetric: false,

            xfer_function: Box::new(smooth_soft_clip),
        }
    }

    /// Processes a single sample through the waveshaper.
    #[must_use]
    pub fn process(&self, sample: f64) -> f64 {
        let xfer = &self.xfer_function;
        let drive = if sample.is_sign_negative() && self.asymmetric {
            self.drive_lower
        }
        else {
            self.drive
        };

        xfer(sample * drive, self.curve) / drive
    }

    /// Moves `function` into the waveshaper, which will then use it as its
    /// transfer function. The passed function must have two arguments of type
    /// `f64`, and return `f64`, to be accepted. The first argument refers to
    /// the function's input, the second its "modification" amount (such as
    /// curve tension).
    ///
    /// If the transfer function you want to use only has one argument, use the
    /// `set_xfer_function_single_argument()` method.
    ///
    /// # Notes
    ///
    /// The transfer function should follow these rules:
    ///
    /// - Its input should operate in the range `-1.0` to `1.0`,
    /// - Its second argument should accept the range `0.0` to `1.0`,
    /// - It *should prefer* to output values between `-1.0` and `1.0`.
    ///
    /// The waveshaper's curve parameter operates in the range of `0.0` to `1.0`
    /// by default. If you need to map the waveshaper's curve to the range your
    /// transfer function accepts, see the `set_curve_range()` method. An example
    /// of this may be if the transfer function has an "inverse" part in a different
    /// part of its range (e.g. if `0.0` to `1.0` is its "normal" range, and
    /// `0.0` to `1.0` is its "inverse" range).
    pub fn set_xfer_function<F>(&mut self, function: F)
    where
        F: Fn(f64, f64) -> f64 + Send + 'static,
    {
        self.xfer_function = Box::new(function);
    }

    /// If the transfer function you want to pass only has a single argument
    /// (such as the sine function, for example), use this function to pass it
    /// to the waveshaper.
    pub fn set_xfer_function_single_argument<F>(&mut self, function: F)
    where
        F: Fn(f64) -> f64 + Send + 'static,
    {
        let xfer = move |x: f64, _: f64| -> f64 { function(x) };
        self.xfer_function = Box::new(xfer);
    }

    /// If the transfer function you want to pass does not cover negative
    /// values, use this method to adapt it such that it processes positive
    /// and negative values symmetrically and pass it to the waveshaper.
    ///
    /// Asymmetric processing is still available after calling this method.
    pub fn set_xfer_function_positive_only<F>(&mut self, function: F)
    where
        F: Fn(f64, f64) -> f64 + Send + 'static,
    {
        let xfer = move |x: f64, d: f64| -> f64 {
            if x.is_sign_negative() {
                -function(-x, d)
            }
            else {
                function(x, d)
            }
        };
        self.xfer_function = Box::new(xfer);
    }

    /// `set_xfer_function_single_argument()` and `set_xfer_function_positive_only()`
    /// merged into one method.
    pub fn set_xfer_function_single_argument_positive_only<F>(
        &mut self,
        function: F,
    ) where
        F: Fn(f64) -> f64 + Send + 'static,
    {
        let xfer = move |x: f64, _: f64| -> f64 {
            if x.is_sign_negative() {
                -function(-x)
            }
            else {
                function(x)
            }
        };
        self.xfer_function = Box::new(xfer);
    }

    /// Sets the drive of the waveshaper. If asymmetric distortion is enabled,
    /// this is only used for positive parts of the signal.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `drive` is outside the range of `0.0` to `1.0`.
    pub fn set_drive(&mut self, drive: f64) {
        debug_assert!((0.0..=1.0).contains(&drive));
        self.drive = drive;
    }

    /// Sets the drive of the waveshaper for negative parts of the signal; only
    /// used if asymmetric distortion is enabled.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `drive` is outside the range of `0.0` to `1.0`.
    pub fn set_drive_lower(&mut self, drive: f64) {
        debug_assert!((0.0..=1.0).contains(&drive));
        self.drive_lower = drive;
    }

    /// Sets the curve of the waveshaper.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `curve` is outside the range of `0.0` to `1.0`.
    pub fn set_curve(&mut self, curve: f64) {
        self.curve =
            curve.clamp(*self.curve_range.start(), *self.curve_range.end());
    }

    /* /// Sets the curve of the waveshaper for negative parts of the signal; only
    /// used if asymmetric distortion is enabled.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `curve` is outside the range of `0.0` to `1.0`.
    pub fn set_curve_lower(&mut self, curve: f64) {
        debug_assert!(self.curve_range.contains(&curve));
        self.curve_lower =
            curve.clamp(*self.curve_range.start(), *self.curve_range.end());
    } */

    /// Sets the range for the waveshaper's `curve` parameters to use. The parameters
    /// are clamped to this range, and if a value which exceeds this range is passed
    /// to either the `set_curve()` or `set_curve_lower()` methods, they will panic
    /// in debug mode.
    ///
    /// The intended purpose of this function is to mutate the range which you will
    /// normally pass to the waveshaper, for whatever reason you need.
    ///
    /// # Example
    ///
    /// ```
    /// let mut ws = Waveshaper::new();
    ///
    /// // by default, this will panic (in debug mode)
    /// // ws.set_curve(-1.0);
    ///
    /// ws.set_curve_range(-1.0..=1.0);
    ///
    /// ws.set_curve(-1.0);
    /// ws.set_curve_lower(-0.1234);
    /// ```
    pub fn set_curve_range(&mut self, range: RangeInclusive<f64>) {
        self.curve_range = range;
    }

    /// Sets whether the waveshaper separately applies drive to the positive and
    /// negative parts of the waveform.
    pub fn set_asymmetric(&mut self, asymmetric: bool) {
        self.asymmetric = asymmetric;
    }
}

impl Default for Waveshaper {
    fn default() -> Self {
        Self::new()
    }
}

/// Smooth soft saturation function. `input` is clamped between `-1.0` and `1.0`,
/// and `c` is clamped between `0.0` and `1.0`. Outputs in the range `-1.0` to `1.0`.
///
/// Raising `c` will increase the amount of saturation.
///
/// From <https://www.musicdsp.org/en/latest/Effects/42-soft-saturation.html>
pub fn smooth_soft_clip(mut input: f64, mut c: f64) -> f64 {
    input = input.clamp(-1.0, 1.0);
    c = 1.0 - c.clamp(0.0, 1.0);

    let abs = input.abs();
    let sign = if input.is_sign_positive() { 1.0 } else { -1.0 };

    if abs > 1.0 {
        (c + 1.0) / 2.0 * sign
    }
    else if abs > c {
        c + (abs - c) / (1.0 + ((abs + c) / (1.0 - c)).powi(2)) * sign
    }
    else {
        input
    }
}
