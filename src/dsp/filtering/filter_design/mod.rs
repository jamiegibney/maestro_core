//! Filter design methods intended for oversampling. All based on the JUCE implementations.
//!
//! Unused in this project.

use crate::prelude::*;
use realfft::num_complex::ComplexFloat;
use std::rc::Rc;

pub mod coefficients;
pub mod design;

pub use coefficients::*;
pub use design::*;
