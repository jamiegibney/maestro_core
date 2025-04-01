//! State variable filter, based on the TPT (Topology-Preserving Transform) design.

use super::*;
use crate::dsp::Effect;
use crate::prelude::*;

pub mod filter;

pub use filter::StateVariableFilter;

#[cfg(test)]
mod tests {
    use super::*;
    //
}
