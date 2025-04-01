//! Global utility functions — these are publicly re-exported in `prelude.rs`.

use crate::settings::{SAMPLE_RATE, TUNING_FREQ_HZ};
use nannou::prelude::{DVec2, Vec2};
use std::f64::consts::PI;
use std::sync::atomic::Ordering::Relaxed;

pub mod atomic_ops;
pub mod general;
pub mod interp;
pub mod smoothing;
pub mod thread_pool;
pub mod timer;
pub mod window;
pub mod xfer;

pub use interp::InterpolationType as InterpType;

pub use atomic_ops::AtomicOps;
pub use general::*;
pub use interp::{ilerp, lerp};
pub use smoothing::*;
pub use thread_pool::ThreadPool;
pub use xfer::SmoothingType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_freq_conversion() {
        let e6 = 88.0;
        let freq = note_to_freq(e6);
        assert!(within_tolerance(freq, 1318.51, 0.001));
        assert!(within_tolerance(freq_to_note(freq), e6, f64::EPSILON));
    }

    #[test]
    fn test_amplitude_conversion() {
        let level = 0.5;
        let db = level_to_db(level);
        assert!(within_tolerance(db, -6.020_599_913_279_624, f64::EPSILON));
        assert!(within_tolerance(db_to_level(db), level, f64::EPSILON));
    }
}
