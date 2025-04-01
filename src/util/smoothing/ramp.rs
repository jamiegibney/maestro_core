//! Non-atomic linear segment generator.

use super::smoothable_types::Smoothable;
use crate::prelude::*;

/// The constant target for `Ramp`.
const RAMP_TARGET: f64 = 1.0;

/// A linear segment generator ("ramp") which smooths between `0.0` and `1.0`.
/// Used as the internal system for `Smoother`.
///
/// For thread-safe operations, use `SmootherAtomic`.
#[derive(Debug, Clone, Copy)]
pub struct Ramp {
    /// The number of smoothing steps remaining until the target is reached.
    steps_remaining: i32,

    /// The step increment for each step, which should be called each sample.
    step_size: f64,

    /// The smoothed value for the current sample.
    current_value: f64,

    /// The duration of smoothing in milliseconds.
    duration_ms: f64,

    sample_rate: f64,
}

impl Ramp {
    /// Returns a new `Ramp` with the provided duration time in milliseconds.
    pub fn new(duration_ms: f64, sample_rate: f64) -> Self {
        let mut s = Self {
            duration_ms,
            sample_rate,
            steps_remaining: 0,
            step_size: 0.0,
            current_value: 0.0,
        };
        s.setup();
        s
    }

    /// Yields the next sample's smoothed value.
    pub fn next(&mut self) -> f64 {
        self.skip(1)
    }

    /// Skips `num_steps` samples, returning the new value.
    pub fn skip(&mut self, num_steps: u32) -> f64 {
        debug_assert_ne!(num_steps, 0);

        let Self { steps_remaining, step_size, current_value, .. } = self;

        if *steps_remaining <= 0 {
            return RAMP_TARGET;
        }

        if *steps_remaining <= num_steps as i32 {
            *steps_remaining = 0;
            *current_value = RAMP_TARGET;
        }
        else {
            *current_value += *step_size * num_steps as f64;
            *steps_remaining -= num_steps as i32;
        }

        *current_value
    }

    /// Returns the current value in the `Ramp`, i.e. the last value returned
    /// by the [`next()`][Self::next()] method.
    pub const fn current_value(&self) -> f64 {
        self.current_value
    }

    /// Fills `block` with the next `block_len` smoothed values. Progresses
    /// the `Ramp`.
    pub fn next_block(&mut self, block: &mut [f64], block_len: usize) {
        self.next_block_exact(&mut block[..block_len]);
    }

    /// Fills block with filled samples. Progresses the `Ramp` by `block.len()`
    /// values.
    pub fn next_block_exact(&mut self, block: &mut [f64]) {
        let Self { steps_remaining, step_size, current_value, .. } = self;

        let steps_remaining = *steps_remaining as usize;
        let num_smoothed_values = block.len().min(steps_remaining);

        if num_smoothed_values == 0 {
            block.fill(RAMP_TARGET);
            return;
        }

        let filler = || {
            *current_value += *step_size;
            *current_value
        };

        if num_smoothed_values == steps_remaining {
            block[..num_smoothed_values - 1].fill_with(filler);

            *current_value = RAMP_TARGET;
            block[num_smoothed_values - 1] = RAMP_TARGET;
        }
        else {
            block[..num_smoothed_values].fill_with(filler);
        }

        block[num_smoothed_values..].fill(RAMP_TARGET);

        *current_value = RAMP_TARGET;
        self.steps_remaining -= num_smoothed_values as i32;
    }

    /// Same as the [`next_block`][Self::next_block()] method, but applies
    /// a mapping function to each element (should map `0.0` to `1.0` to
    /// the desired range).
    pub fn next_block_mapped<T, F>(
        &mut self,
        block: &mut [T],
        _block_len: usize,
        function: F,
    ) where
        F: FnMut(f64) -> T,
        T: Smoothable,
    {
        self.next_block_exact_mapped(block, function);
    }

    /// Same as the [`next_block_exaxt`][Self::next_block_exact()] method,
    /// but applies a mapping function to each element (should map `0.0`
    /// to `1.0` to the desired range).
    pub fn next_block_exact_mapped<T, F>(
        &mut self,
        block: &mut [T],
        mut mapping_function: F,
    ) where
        F: FnMut(f64) -> T,
        T: Smoothable,
    {
        let steps_remaining = self.steps_remaining as usize;
        let num_smoothed_values = block.len().min(steps_remaining);

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
                    self.current_value += self.step_size;
                    *x = mapping_function(self.current_value);
                });

            self.current_value = RAMP_TARGET;
            block[num_smoothed_values - 1] = mapping_function(RAMP_TARGET);
        }
        else {
            block.iter_mut().take(num_smoothed_values).for_each(|x| {
                self.current_value += self.step_size;
                *x = mapping_function(self.current_value);
            });
        }

        block
            .iter_mut()
            .skip(num_smoothed_values)
            .for_each(|x| *x = mapping_function(RAMP_TARGET));

        self.steps_remaining -= num_smoothed_values as i32;
    }

    /// Resets the `Ramp` to the provided value, and recomputes its
    /// step size/remaining count.
    pub fn reset_to(&mut self, value: f64) {
        self.current_value = value;
        self.setup();
    }

    /// Resets the `Ramp`, which sets its current value to `0.0` and
    /// recomputes its step size/remaining count.
    pub fn reset(&mut self) {
        self.current_value = 0.0;
        self.setup();
    }

    pub fn reset_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    /// Resets the duration of the `Ramp` in milliseconds.
    pub fn set_duration(&mut self, duration_ms: f64) {
        self.duration_ms = duration_ms;
        self.setup();
    }

    /// Returns how many steps the `Ramp` has remaining.
    pub const fn steps_remaining(&self) -> u32 {
        self.steps_remaining as u32
    }

    /// Returns whether the `Ramp` is actively smoothing or not.
    pub const fn is_active(&self) -> bool {
        self.steps_remaining > 0
    }

    /// Sets the `Ramp`'s internal step size and step count.
    fn setup(&mut self) {
        let steps_remaining = self.duration_samples();
        self.steps_remaining = steps_remaining as i32;

        self.step_size =
            if steps_remaining > 0 { self.compute_step_size() } else { 0.0 };
    }

    /// Computes the total number of steps required to reach the target value
    /// (i.e. the duration as samples).
    fn duration_samples(&self) -> u32 {
        (self.sample_rate * self.duration_ms / 1000.0).round() as u32
    }

    /// Computes the size of each step.
    fn compute_step_size(&self) -> f64 {
        (RAMP_TARGET - self.current_value) / (self.steps_remaining as f64)
    }
}

impl Default for Ramp {
    fn default() -> Self {
        Self::new(0.1, unsafe { SAMPLE_RATE })
    }
}
