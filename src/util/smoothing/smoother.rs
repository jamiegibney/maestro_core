//! Non-atomic value smoother.

#![allow(clippy::should_implement_trait)]
use super::ramp::Ramp;
use super::*;
use crate::prelude::*;

use SmoothingType as ST;

#[derive(Debug, Default, Clone)]
pub struct Smoother<T: Smoothable> {
    ramp: Ramp,
    start_value: T,
    target_value: T,
    current_value: T,

    smoothing_type: ST,
}

impl<T: Smoothable> Smoother<T> {
    /// Creates a new `Smoother` with linear smoothing (see the
    /// [`set_smoothing_type()`][Self::set_smoothing_type()] method).
    pub fn new(duration_ms: f64, target_value: T, sample_rate: f64) -> Self {
        Self {
            ramp: Ramp::new(duration_ms, sample_rate),
            start_value: T::from_f64(0.0),
            current_value: target_value,
            target_value,

            smoothing_type: SmoothingType::default(),
        }
    }

    /// Creates a smoother with `smoothing_type` smoothing.
    pub fn with_smoothing_type(
        mut self,
        smoothing_type: SmoothingType,
    ) -> Self {
        self.set_smoothing_type(smoothing_type);
        self
    }

    /// Yields the `Smoother`'s next smoothed value, intended to be called
    /// per sample. If you need a large block of samples, see the
    /// [`next_block()`][Self::next_block()] method. If you need to skip a
    /// certain number of samples, see the [`skip()`][Self::skip()] method.
    /// Both methods provide optimizations over calling this method N times.
    pub fn next(&mut self) -> T {
        self.skip(1)
    }

    /// Skips `num_steps` steps, returning the new value. In effect, equivalent
    /// to calling the [`next()`][Self::next()] method `num_steps` times, but
    /// provides some internal optimizations.
    ///
    pub fn skip(&mut self, num_steps: u32) -> T {
        if !self.is_active() {
            return self.current_value();
        }

        self.ramp.skip(num_steps);
        self.interpolated_value()
    }

    /// Computes the `block_len` next elements and places them into `block`.
    ///
    /// Progresses the `Smoother` by `block_len` steps.
    pub fn next_block(&mut self, block: &mut [T], block_len: usize) {
        debug_assert!(block_len <= block.len());
        self.next_block_exact(&mut block[..block_len]);
    }

    /// Computes a block of new elements and places them into `block`.
    ///
    /// Progresses the `Smoother` by `block.len()` steps.
    pub fn next_block_exact(&mut self, block: &mut [T]) {
        let (a, b) = (self.start_value.to_f64(), self.target_value.to_f64());

        self.ramp.next_block_exact_mapped(block, |t: f64| {
            T::from_f64(match self.smoothing_type {
                ST::Linear => lerp(a, b, t),
                ST::Cosine => interp::cosine(a, b, t),
                ST::SineTop => lerp(a, b, xfer::sine_upper(t)),
                ST::SineBottom => lerp(a, b, xfer::sine_lower(t)),
                ST::CurveNormal(c) => lerp(a, b, xfer::s_curve(t, c)),
                ST::CurveLinearStart(c) => {
                    lerp(a, b, xfer::s_curve_linear_centre(t, c))
                }
                ST::CurveRounder(c) => lerp(a, b, xfer::s_curve_round(t, c)),
            })
        });
    }

    /// Stops the `Smoother` in-place, holding its current value. Any calls
    /// to the [`next()`][Self::next()] method (or its variants) will have
    /// no effect, and will return the current value.
    pub fn stop_in_place(&mut self) {
        self.target_value = self.current_value;
        self.finish();
    }

    /// Forces the `Smoother` to finish smoothing and reach its target value
    /// immediately.
    pub fn finish(&mut self) {
        self.ramp.reset();
    }

    /// Returns the `Smoother`'s current value, i.e. the last value returned
    /// by its [`next()`][Self::next()] method.
    pub fn current_value(&self) -> T {
        self.current_value
    }

    /// Returns the current target value of the smoother.
    pub fn target_value(&self) -> T {
        self.target_value
    }

    /// Sets the smoothing (interpolation) type of the `Smoother`. See the
    /// variants of `SmoothingType` for all the options.
    pub fn set_smoothing_type(&mut self, smoothing_type: SmoothingType) {
        self.smoothing_type = smoothing_type;
    }

    /// Sets the new target value of the `Smoother`. This will automatically
    /// set its starting value to the current value.
    pub fn set_target_value(&mut self, target_value: T) {
        self.target_value = target_value;

        if self.is_active() {
            let interp = ilerp(
                self.start_value.to_f64(),
                target_value.to_f64(),
                self.current_value.to_f64(),
            );

            self.ramp.reset_to(interp);
        }
        else {
            self.ramp.reset();
        }

        self.start_value = self.current_value;
    }

    /// Sets the starting value of the `Smoother` (the value it is
    /// interpolating from).
    pub fn set_start_value(&mut self, start_value: T) {
        self.start_value = start_value;
    }

    /// Resets the `Smoother` to its default settings.
    pub fn reset(&mut self) {
        self.ramp.reset();
        self.start_value = T::from_f64(0.0);
        self.target_value = T::from_f64(0.0);
        self.current_value = T::from_f64(0.0);
    }

    pub fn reset_to(&mut self, value: T) {
        self.ramp.reset_to(value.to_f64());
    }

    /// Resets the smoothing period of the `Smoother` in milliseconds.
    pub fn set_smoothing_period(&mut self, duration_ms: f64) {
        self.ramp.set_duration(duration_ms);
    }

    pub fn reset_sample_rate(&mut self, sample_rate: f64) {
        self.ramp.reset_sample_rate(sample_rate);
    }

    /// Returns whether the `Smoother` is actively smoothing or not.
    pub fn is_active(&self) -> bool {
        self.ramp.is_active()
    }

    /// Computes the interpolated value based on the current `SmoothingType`.
    fn interpolated_value(&mut self) -> T {
        let (a, b, t) = (
            self.start_value.to_f64(),
            self.target_value.to_f64(),
            self.ramp.current_value(),
        );

        self.current_value = T::from_f64(match self.smoothing_type {
            ST::Linear => interp::lerp(a, b, t),
            ST::Cosine => interp::cosine(a, b, t),
            ST::SineTop => interp::lerp(a, b, xfer::sine_upper(t)),
            ST::SineBottom => interp::lerp(a, b, xfer::sine_lower(t)),
            ST::CurveNormal(tension) => {
                interp::lerp(a, b, xfer::s_curve(t, tension))
            }
            ST::CurveLinearStart(tension) => {
                interp::lerp(a, b, xfer::s_curve_linear_centre(t, tension))
            }
            ST::CurveRounder(tension) => {
                interp::lerp(a, b, xfer::s_curve_round(t, tension))
            }
        });

        self.current_value
    }

    fn map(&self, t: f64) -> T {
        let (a, b) = (self.start_value.to_f64(), self.target_value.to_f64());

        T::from_f64(match self.smoothing_type {
            ST::Linear => interp::lerp(a, b, t),
            ST::Cosine => interp::cosine(a, b, t),
            ST::SineTop => interp::lerp(a, b, xfer::sine_upper(t)),
            ST::SineBottom => interp::lerp(a, b, xfer::sine_lower(t)),
            ST::CurveNormal(tension) => {
                interp::lerp(a, b, xfer::s_curve(t, tension))
            }
            ST::CurveLinearStart(tension) => {
                interp::lerp(a, b, xfer::s_curve_linear_centre(t, tension))
            }
            ST::CurveRounder(tension) => {
                interp::lerp(a, b, xfer::s_curve_round(t, tension))
            }
        })
    }
}
