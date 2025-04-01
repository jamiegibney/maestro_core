//! Second-order biquad filter form supporting various filter types.

#![allow(unused, clippy::must_use_candidate)]
mod filter;
use super::{Filter, FilterType};
use crate::util;

pub use filter::*;
