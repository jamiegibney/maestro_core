//! Digital signal processors and utilities.

use crate::prelude::*;

pub mod delay;
pub mod distortion;
pub mod dynamics;
pub mod filtering;
pub mod modulation;
pub mod fx;
pub mod oversampling;
pub mod spectral;
pub mod synthesis;
pub mod util;

pub use delay::{Delay, RingBuffer, StereoDelay};
pub use distortion::Waveshaper;
pub use dynamics::adsr::{AdsrEnvelope, AdsrParameters};
pub use dynamics::Compressor;
pub use filtering::{
    biquad::{BiquadFilter, BiquadParams},
    comb::{FirCombFilter, IirCombFilter},
    first_order::FirstOrderFilter,
    lrf::LinkwitzRileyFilter,
    resonator::{
        resonator_bank::{ResoBankData, ResonatorBank, ResonatorBankParams},
        two_pole_resonator::TwoPoleResonator,
    },
    simple::{dc_filter::DCFilter, one_pole_lowpass::OnePoleLowpass},
    svf::StateVariableFilter,
    Filter, FilterType, BUTTERWORTH_Q,
};
pub use oversampling::{Oversampler, OversamplingBuffer};
pub use spectral::{
    spectral_filter::{mask::SpectralMask, SpectralFilter},
    StftHelper,
};
pub use synthesis::Generator;
pub use util::*;
