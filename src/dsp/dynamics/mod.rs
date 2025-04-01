//! Module for dynamics processors.

use super::filtering::simple::ballistics::{
    BallisticsFilter, BallisticsLevelType,
};
use super::*;

pub mod adsr;
pub mod compressor;

pub use compressor::Compressor;
