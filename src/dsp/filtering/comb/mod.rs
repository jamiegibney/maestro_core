//! FIR (finite impulse response) and IIR (infinite impulse response) comb filter forms.

#![allow(clippy::must_use_candidate)]
mod filter;
mod fir;
mod iir;

use crate::dsp::*;

pub use fir::FirCombFilter;
pub use iir::IirCombFilter;

#[cfg(test)]
mod tests {
    // use super::*;
}
