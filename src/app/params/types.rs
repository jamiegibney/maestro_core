//! All the custom UI parameter types.

use super::*;
use bytemuck::NoUninit;
use std::fmt::{Display, Formatter, Result};

/// The current algorithm used by the spectral filter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GenerativeAlgo {
    #[default]
    /// A perlin noise contour-line generator.
    Contours,
    /// A [SmoothLife](https://arxiv.org/abs/1111.1567) simulation.
    SmoothLife,
    /// A Voronoi noise generator.
    Voronoi,
}

impl Display for GenerativeAlgo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Contours => write!(f, "Contours"),
            Self::SmoothLife => write!(f, "Smooth Life"),
            Self::Voronoi => write!(f, "Voronoi"),
        }
    }
}

unsafe impl NoUninit for GenerativeAlgo {}

// *** //

#[derive(Clone, Copy, Debug, Default)]
pub enum SmoothLifePreset {
    #[default]
    Jitter,
    Slime,
    Corrupt,
}

impl Display for SmoothLifePreset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Jitter => write!(f, "Jitter"),
            Self::Slime => write!(f, "Slime"),
            Self::Corrupt => write!(f, "Corrupt"),
        }
    }
}

unsafe impl NoUninit for SmoothLifePreset {}

// *** //

#[derive(Clone, Copy, Debug, Default)]
pub enum SpectrogramView {
    #[default]
    /// Draw both the pre- and post-FX spectrograms.
    PrePost,
    /// Only draw the pre-FX spectrogram.
    PreOnly,
    /// Only draw the post-FX spectrogram.
    PostOnly,
}

impl Display for SpectrogramView {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::PrePost => write!(f, "Pre/Post"),
            Self::PreOnly => write!(f, "Pre"),
            Self::PostOnly => write!(f, "Post"),
        }
    }
}

unsafe impl NoUninit for SpectrogramView {}

// *** //

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DistortionType {
    #[default]
    /// No distortion.
    None,
    /// A smooth soft clipping function.
    ///
    /// ([`smooth_soft_clip`](crate::dsp::distortion::waveshaper::smooth_soft_clip))
    Soft,
    /// More aggressive clipping function — not technically hard digital clipping! TODO
    Hard,
    /// A wrapping clipping algorithm. TODO
    Wrap,
    /// Downsampling distortion. TODO
    Crush,
}

impl Display for DistortionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Soft => write!(f, "Soft"),
            Self::Hard => write!(f, "Hard"),
            Self::Wrap => write!(f, "Wrap"),
            Self::Crush => write!(f, "Crush"),
        }
    }
}

unsafe impl NoUninit for DistortionType {}

// *** //

#[derive(Clone, Copy, Debug, Default)]
pub enum SmoothLifeSize {
    S16,
    S32,
    S64,
    S128,
    #[default]
    S256,
    S512,
}

impl SmoothLifeSize {
    pub fn value(&self) -> usize {
        match self {
            Self::S16 => 16,
            Self::S32 => 32,
            Self::S64 => 64,
            Self::S128 => 128,
            Self::S256 => 256,
            Self::S512 => 512,
        }
    }
}

impl Display for SmoothLifeSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value())
    }
}

unsafe impl NoUninit for SmoothLifeSize {}

// *** //

#[derive(Clone, Copy, Debug, Default)]
pub enum SpectrogramSize {
    S1024,
    #[default]
    S2048,
    S4096,
    S8192,
}

impl SpectrogramSize {
    pub fn value(&self) -> usize {
        match self {
            Self::S1024 => 1024,
            Self::S2048 => 2048,
            Self::S4096 => 4096,
            Self::S8192 => 8192,
        }
    }
}

impl Display for SpectrogramSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value())
    }
}

unsafe impl NoUninit for SpectrogramSize {}

// *** //

/// The available block sizes for the spectral filter.
#[derive(Clone, Copy, Debug, Default)]
pub enum SpectralFilterSize {
    S64,
    S128,
    S256,
    S512,
    #[default]
    S1024,
    S2048,
    S4096,
    S8192,
    S16384,
}

impl SpectralFilterSize {
    pub fn value(&self) -> usize {
        match self {
            Self::S64 => 64,
            Self::S128 => 128,
            Self::S256 => 256,
            Self::S512 => 512,
            Self::S1024 => 1024,
            Self::S2048 => 2048,
            Self::S4096 => 4096,
            Self::S8192 => 8192,
            Self::S16384 => 16384,
        }
    }
}

impl Display for SpectralFilterSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value())
    }
}

unsafe impl NoUninit for SpectralFilterSize {}

// *** //


/// The current oscillator used for each voice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExciterOscillator {
    Sine,
    Tri,
    Saw,
    Square,
    #[default]
    Noise,
}

impl Display for ExciterOscillator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Sine => write!(f, "Sine"),
            Self::Tri => write!(f, "Tri"),
            Self::Saw => write!(f, "Saw"),
            Self::Square => write!(f, "Square"),
            Self::Noise => write!(f, "Noise"),
        }
    }
}

unsafe impl NoUninit for ExciterOscillator {}
