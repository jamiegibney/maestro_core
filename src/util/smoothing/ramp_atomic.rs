//! Atomic linear segment generator.

use super::smoothable_types::SmoothableAtomic;
use crate::prelude::*;
use atomic_float::AtomicF64;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// The constant target for `Ramp`.
const RAMP_TARGET: f64 = 1.0;

/// A linear segment generator ("ramp") which smooths between `0.0` and `1.0`.
/// Used as the internal system for `SmootherAtomic`.
#[derive(Debug, Default)]
pub struct RampAtomic {
    /// The number of smoothing steps remaining until the target is reached.
    steps_remaining: AtomicU32,

    /// The step increment for each step, which should be called each sample.
    step_size: AtomicF64,

    /// The smoothed value for the current sample.
    current_value: AtomicF64,

    /// The duration of smoothing in milliseconds.
    duration_ms: AtomicF64,

    sample_rate: f64,
}

impl RampAtomic {
    /// Returns a new `Ramp` with the provided duration time in milliseconds.
    pub fn new(duration_ms: f64, sample_rate: f64) -> Self {
        let mut s = Self {
            duration_ms: AtomicF64::new(duration_ms),
            sample_rate,
            steps_remaining: AtomicU32::new(0),
            step_size: AtomicF64::new(0.0),
            current_value: AtomicF64::new(0.0),
        };
        s.setup();
        s
    }

    /// Yields the next sample's smoothed value.
    pub fn next(&mut self) -> f64 {
        self.skip(1)
    }

    /// Skips `num_steps` samples, returning the new value.
    pub fn skip(&self, num_steps: u32) -> f64 {
        if num_steps == 0 {
            return self.current_value();
        }

        let steps_remaining = self.steps_remaining.lr();
        let step_size = self.step_size.lr();

        if steps_remaining == 0 {
            return RAMP_TARGET;
        }

        if steps_remaining <= num_steps {
            self.steps_remaining.sr(0);
            self.current_value.sr(RAMP_TARGET);
        }
        else {
            self.current_value
                .fetch_add(step_size * num_steps as f64, Relaxed);
            self.steps_remaining.fetch_sub(num_steps, Relaxed);
        }

        self.current_value.lr()
    }

    /// Returns the current value in the `Ramp`, i.e. the last value returned
    /// by the [`next()`][Self::next()] method.
    pub fn current_value(&self) -> f64 {
        self.current_value.lr()
    }

    /// Fills `block` with the next `block_len` smoothed values. Progresses
    /// the `Ramp`.
    pub fn next_block(&self, block: &mut [f64], block_len: usize) {
        self.next_block_exact(&mut block[..block_len]);
    }

    /// Fills block with filled samples. Progresses the `Ramp` by `block.len()`
    /// values.
    pub fn next_block_exact(&self, block: &mut [f64]) {
        let Self { steps_remaining, step_size, current_value, .. } = self;

        let step_size = self.step_size.lr();

        let steps_remaining = self.steps_remaining.lr() as usize;
        let num_smoothed_values = block.len().min(steps_remaining);

        if num_smoothed_values == 0 {
            block.fill(RAMP_TARGET);
            return;
        }

        let filler =
            || self.current_value.fetch_add(step_size, Relaxed) + step_size;

        if num_smoothed_values == steps_remaining {
            block[..num_smoothed_values - 1].fill_with(filler);

            self.current_value.sr(RAMP_TARGET);
            block[num_smoothed_values - 1] = RAMP_TARGET;
        }
        else {
            block[..num_smoothed_values].fill_with(filler);
        }

        block[num_smoothed_values..].fill(RAMP_TARGET);

        self.current_value.sr(RAMP_TARGET);
        self.steps_remaining
            .fetch_sub(num_smoothed_values as u32, Relaxed);
    }

    /// Same as the [`next_block`][Self::next_block()] method, but applies
    /// a mapping function to each element (should map `0.0` to `1.0` to
    /// the desired range).
    pub fn next_block_mapped<T, F>(
        &self,
        block: &mut [T],
        _block_len: usize,
        function: F,
    ) where
        F: FnMut(f64) -> T,
        T: SmoothableAtomic,
    {
        self.next_block_exact_mapped(block, function);
    }

    /// Same as the [`next_block_exaxt`][Self::next_block_exact()] method,
    /// but applies a mapping function to each element (should map `0.0`
    /// to `1.0` to the desired range).
    pub fn next_block_exact_mapped<T, F>(
        &self,
        block: &mut [T],
        mut mapping_function: F,
    ) where
        F: FnMut(f64) -> T,
        T: SmoothableAtomic,
    {
        let steps_remaining = self.steps_remaining.lr() as usize;
        let num_smoothed_values = block.len().min(steps_remaining);
        let step_size = self.step_size.lr();

        if num_smoothed_values == 0 {
            block
                .iter_mut()
                .for_each(|x| *x = mapping_function(RAMP_TARGET));

            return;
        }

        if num_smoothed_values == steps_remaining {
            block
                .iter_mut()
                .take(num_smoothed_values - 1)
                .for_each(|x| {
                    self.current_value.fetch_add(step_size, Relaxed);
                    *x = mapping_function(self.current_value.lr());
                });

            self.current_value.sr(RAMP_TARGET);
            block[num_smoothed_values - 1] = mapping_function(RAMP_TARGET);
        }
        else {
            block.iter_mut().take(num_smoothed_values).for_each(|x| {
                self.current_value.fetch_add(step_size, Relaxed);
                *x = mapping_function(self.current_value.lr());
            });
        }

        block
            .iter_mut()
            .skip(num_smoothed_values)
            .for_each(|x| *x = mapping_function(RAMP_TARGET));

        self.steps_remaining
            .fetch_sub(num_smoothed_values as u32, Relaxed);
    }

    /// Resets the `Ramp` to the provided value, and recomputes its
    /// step size/remaining count.
    pub fn reset_to(&self, value: f64) {
        self.current_value.sr(value.clamp(0.0, 1.0));
        self.setup();
    }

    /// Resets the `Ramp`, which sets its current value to `0.0` and
    /// recomputes its step size/remaining count.
    pub fn reset(&self) {
        self.current_value.sr(0.0);
        self.setup();
        self.compute_step_size();
    }

    /// Resets the ramp's internal sample rate.
    pub fn reset_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    /// Resets the duration of the `Ramp` in milliseconds.
    pub fn set_duration(&self, duration_ms: f64) {
        self.duration_ms.sr(duration_ms);
        self.setup();
    }

    /// Returns how many steps the `Ramp` has remaining.
    pub fn steps_remaining(&self) -> u32 {
        self.steps_remaining.lr()
    }

    /// Returns whether the `Ramp` is actively smoothing or not.
    pub fn is_active(&self) -> bool {
        self.steps_remaining.lr() > 0
    }

    /// Sets the `Ramp`'s internal step size and step count.
    fn setup(&self) {
        let steps_remaining = self.duration_samples();
        self.steps_remaining.sr(steps_remaining);
    }

    /// Computes the total number of steps required to reach the target value
    /// (i.e. the duration as samples).
    fn duration_samples(&self) -> u32 {
        (self.sample_rate * self.duration_ms.lr() / 1000.0).round() as u32
    }

    /// Computes the size of each step.
    fn compute_step_size(&self) {
        self.step_size.sr(if self.steps_remaining() > 0 {
            (RAMP_TARGET - self.current_value.lr())
                / (self.steps_remaining.lr() as f64)
        }
        else {
            0.0
        });
    }
}

impl Clone for RampAtomic {
    fn clone(&self) -> Self {
        Self {
            steps_remaining: AtomicU32::new(self.steps_remaining.lr()),
            step_size: AtomicF64::new(self.step_size.lr()),
            current_value: AtomicF64::new(self.current_value.lr()),
            duration_ms: AtomicF64::new(self.duration_ms.lr()),
            sample_rate: self.sample_rate,
        }
    }
}
