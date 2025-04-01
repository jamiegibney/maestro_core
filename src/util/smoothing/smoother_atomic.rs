//! Atomic value smoother.

#![allow(clippy::should_implement_trait)]
use super::ramp_atomic::RampAtomic;
use super::*;
use crate::prelude::*;

use SmoothingType as ST;

#[derive(Debug, Default)]
pub struct SmootherAtomic<T: SmoothableAtomic> {
    ramp: RampAtomic,
    start_value: AtomicF64,
    current_value: AtomicF64,
    target_value: T::Atomic,

    smoothing_type: ST,
}

impl<T: SmoothableAtomic> SmootherAtomic<T> {
    /// Creates a new `Smoother` with linear smoothing (see the
    /// [`set_smoothing_type()`][Self::set_smoothing_type()] method).
    pub fn new(duration_ms: f64, target_value: T, sample_rate: f64) -> Self {
        Self {
            ramp: RampAtomic::new(duration_ms, sample_rate),
            start_value: AtomicF64::new(target_value.to_f64()),
            target_value: target_value.atomic_new(),
            current_value: AtomicF64::new(target_value.to_f64()),

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
    pub fn next(&self) -> T {
        self.skip(1)
    }

    /// Skips `num_steps` steps, returning the new value. In effect, equivalent
    /// to calling the [`next()`][Self::next()] method `num_steps` times, but
    /// provides some internal optimizations.
    ///
    pub fn skip(&self, num_steps: u32) -> T {
        if self.is_active() {
            self.ramp.skip(num_steps);

            self.interpolated_value()
        }
        else {
            self.current_value()
        }
    }

    /// Computes the `block_len` next elements and places them into `block`.
    ///
    /// Progresses the `Smoother` by `block_len` steps.
    pub fn next_block(&self, block: &mut [T], block_len: usize) {
        debug_assert!(block_len <= block.len());
        self.next_block_exact(&mut block[..block_len]);
    }

    /// Computes a block of new elements and places them into `block`.
    ///
    /// Progresses the `Smoother` by `block.len()` steps.
    pub fn next_block_exact(&self, block: &mut [T]) {
        let (a, b) = (
            self.start_value.lr(),
            T::atomic_load(&self.target_value).to_f64(),
        );

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
    pub fn stop_in_place(&self) {
        T::atomic_store(
            &self.target_value,
            T::from_f64(self.current_value.lr()),
        );
        self.finish();
    }

    /// Forces the `Smoother` to finish smoothing and reach its target value
    /// immediately.
    pub fn finish(&self) {
        self.ramp.reset();
    }

    /// Returns the `Smoother`'s current value, i.e. the last value returned
    /// by its [`next()`][Self::next()] method.
    pub fn current_value(&self) -> T {
        T::from_f64(self.current_value.lr())
    }

    /// Sets the smoothing (interpolation) type of the `Smoother`. See the
    /// variants of `SmoothingType` for all the options.
    pub fn set_smoothing_type(&mut self, smoothing_type: SmoothingType) {
        self.smoothing_type = smoothing_type;
    }

    /// Sets the new target value of the `Smoother`. This will automatically
    /// set its starting value to the current value.
    pub fn set_target_value(&self, target_value: T) {
        if epsilon_eq(
            target_value.to_f64(),
            T::atomic_load(&self.target_value).to_f64(),
        ) {
            return;
        }

        self.start_value.sr(self.current_value.lr());
        T::atomic_store(&self.target_value, target_value);

        self.ramp.reset();
    }

    /// Sets the starting value of the `Smoother` (the value it is
    /// interpolating from).
    pub fn set_start_value(&self, start_value: T) {
        self.start_value.sr(start_value.to_f64());
    }

    /// Resets the `Smoother` to its default settings.
    pub fn reset(&self) {
        self.ramp.reset();

        self.start_value.sr(0.0);
        self.current_value.sr(0.0);
        T::atomic_store(&self.target_value, T::from_f64(0.0));
    }

    pub fn reset_to(&self, value: T) {
        self.ramp.reset_to(value.to_f64());
    }

    /// Resets the smoothing period of the `Smoother` in milliseconds.
    pub fn set_smoothing_period(&self, duration_ms: f64) {
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
    fn interpolated_value(&self) -> T {
        let (a, b, t) = (
            self.start_value.lr(),
            T::atomic_load(&self.target_value).to_f64(),
            self.ramp.current_value(),
        );

        let current_value = T::from_f64(match self.smoothing_type {
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

        self.current_value.sr(current_value.to_f64());

        current_value
    }

    fn map(&self, t: f64) -> T {
        let (a, b) = (
            self.start_value.lr(),
            T::atomic_load(&self.target_value).to_f64(),
        );

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

impl<T: SmoothableAtomic> Clone for SmootherAtomic<T> {
    fn clone(&self) -> Self {
        let t = T::from_f64(1.0).atomic_new();
        T::atomic_store(&t, T::atomic_load(&self.target_value));

        Self {
            ramp: self.ramp.clone(),
            start_value: AtomicF64::new(self.start_value.lr()),
            current_value: AtomicF64::new(self.current_value.lr()),
            target_value: t,
            smoothing_type: self.smoothing_type,
        }
    }
}
