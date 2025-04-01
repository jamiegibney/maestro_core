//! Musical scale representations.

use bytemuck::NoUninit;
use std::fmt::Display;

/// Common scale representations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scale {
    Major,
    Minor,
    #[default]
    MajPentatonic,
    MinPentatonic,
    Chromatic,
}

impl Display for Scale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Major => write!(f, "Major"),
            Self::Minor => write!(f, "Minor"),
            Self::MajPentatonic => write!(f, "Maj Pent."),
            Self::MinPentatonic => write!(f, "Min Pent."),
            Self::Chromatic => write!(f, "Chromatic"),
        }
    }
}

unsafe impl NoUninit for Scale {}

impl Scale {
    pub fn get(&self) -> &[f64] {
        match self {
            Self::Major => &ScaleValues::MAJOR,
            Self::Minor => &ScaleValues::MINOR,
            Self::MajPentatonic => &ScaleValues::MAJ_PENTATONIC,
            Self::MinPentatonic => &ScaleValues::MIN_PENTATONIC,
            Self::Chromatic => &ScaleValues::CHROMATIC,
        }
    }

    /// Quantizes `note` to its current scale, i.e. snaps it to the nearest
    /// possible note within the scale. `root` is only used to find the offset
    /// for this particular scale.
    pub fn quantize_to_scale(&self, note: f64, root: f64) -> f64 {
        // FIXME: need to avoid the output being -1 (if note == 0)
        // start with the root note
        let mut lower = root;

        // shift the root note until it is the bottom of the octave
        // containing `note`
        while !(lower..=(lower + 12.0)).contains(&note) {
            lower += if note > lower { 12.0 } else { -12.0 };
        }

        // then get the scale
        let scale = self.get();

        // and find the smallest difference between note and the scale intervals
        let mut min = f64::MAX;
        let mut idx = 0;

        for (i, &interval) in scale.iter().enumerate() {
            let cur = lower + interval; // interval in scale
            let val = (note - cur).abs(); // difference

            if val < min {
                min = val;
                idx = i;
            }
        }

        lower + scale[idx]
    }
}

struct ScaleValues;

impl ScaleValues {
    pub const MAJOR: [f64; 7] = [0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0];
    pub const MINOR: [f64; 7] = [0.0, 2.0, 3.0, 5.0, 7.0, 8.0, 10.0];
    pub const MAJ_PENTATONIC: [f64; 5] = [0.0, 2.0, 4.0, 7.0, 9.0];
    pub const MIN_PENTATONIC: [f64; 5] = [0.0, 3.0, 5.0, 7.0, 10.0];
    pub const CHROMATIC: [f64; 12] =
        [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0];
}
