//! A wrapper around two mono `impl `[`Effect`] types with stereo utilities.

use super::Effect;

/// A simple wrapper around two mono `Effect` objects.
#[derive(Clone, Debug)]
pub struct StereoWrapper<E> {
    pub l: E,
    pub r: E,
}

impl<E: Effect + Clone + Default> StereoWrapper<E> {
    pub fn from_single(effect: E) -> Self {
        Self { l: effect.clone(), r: effect }
    }

    pub fn from_pair(effect_l: E, effect_r: E) -> Self {
        Self { l: effect_l, r: effect_r }
    }

    pub fn unwrap(self) -> (E, E) {
        (self.l, self.r)
    }
}

impl<E: Effect + Clone + Default> Effect for StereoWrapper<E> {
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let out_l = self.l.process_mono(in_l, 0);
        let out_r = self.r.process_mono(in_r, 1);

        (out_l, out_r)
    }

    fn process_mono(&mut self, input: f64, ch_idx: usize) -> f64 {
        match ch_idx {
            0 => self.l.process_mono(input, ch_idx),
            1 => self.r.process_mono(input, ch_idx),
            _ => input,
        }
    }

    fn get_sample_rate(&self) -> f64 {
        self.l.get_sample_rate()
    }

    fn get_identifier(&self) -> &str {
        "stereo_wrapper"
    }
}

impl<E: Effect + Clone + Default> Default for StereoWrapper<E> {
    fn default() -> Self {
        Self { l: E::default(), r: E::default() }
    }
}
