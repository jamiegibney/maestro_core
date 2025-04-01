//! Primitive white noise oscillator.

use super::*;
use crate::prelude::*;

/// A white noise oscillator.
#[derive(Debug, Clone, Copy)]
pub struct NoiseOsc;

impl NoiseOsc {
    /// Produces a single noise sample at 0.0 dBFS.
    pub fn process() -> f64 {
        random_f64().mul_add(2.0, -1.0)
    }
}
