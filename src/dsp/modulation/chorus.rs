//! Unused chorus effect.

use super::*;
use crate::dsp::synthesis::*;

#[derive(Clone, Debug)]
pub struct Chorus {
    delay_taps: Vec<StereoWrapper<Delay>>,
    lfo: Generator,
    mod_depth: f64,
    mod_rate: f64,
    invert_delay_polarity: bool,
}

impl Chorus {
    const DEFAULT_NUM_TAPS: usize = 3;

    pub fn new(sample_rate: f64) -> Self {
        Self {
            delay_taps: vec![
                StereoWrapper::from_single(Delay::new(250.0, sample_rate));
                Self::DEFAULT_NUM_TAPS
            ],
            lfo: Generator::Sine(SineOsc::new(3.0, sample_rate)),
            mod_depth: 1.0,
            mod_rate: 3.0,
            invert_delay_polarity: false,
        }
    }

    pub fn set_mod_rate(&mut self, rate: f64) {
        self.mod_rate = rate.clamp(0.1, 10.0);
    }

    pub fn set_mod_depth(&mut self, depth: f64) {
        self.mod_depth = depth.clamp(0.0, 1.0);
    }

    pub fn invert_delay_polarity(&mut self, should_invert: bool) {
        self.invert_delay_polarity = should_invert;
    }
}

impl Effect for Chorus {
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        unimplemented!("chorus is not yet implemented.")
    }

    fn get_sample_rate(&self) -> f64 {
        self.delay_taps[0].get_sample_rate()
    }

    fn get_identifier(&self) -> &str {
        "chorus"
    }
}
