//! Oversampling types and implementation.

use super::*;
use crate::prelude::*;

mod block;
mod lanczos;
mod lanczos_stage;

pub use block::OversamplingBuffer;
pub use lanczos::Lanczos3Oversampler as Oversampler;
