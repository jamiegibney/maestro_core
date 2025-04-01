//! Primitive, non-anti-aliased oscillator types.

use super::*;

pub mod noise_osc;
pub mod phasor;
pub mod sine;
pub mod square;
pub mod tri;

pub use noise_osc::NoiseOsc;
pub use phasor::Phasor;
pub use sine::SineOsc;
pub use square::SquareOsc;
pub use tri::TriOsc;
