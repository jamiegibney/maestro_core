//! "Utility" wrapper around an `impl `[`Effect`] for basic amplitude control.

use super::Effect;
use crate::prelude::*;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug, Default)]
pub enum PanningLaw {
    /// AKA -6 dB
    Linear,
    /// AKA -3 dB, or sine law
    #[default]
    ConstantPower,
}

#[derive(Clone, Debug)]
pub struct AudioUtility<E: Effect> {
    inner: E,
    gain: f64,
    pan: f64,
    panning_law: PanningLaw,
    invert: (bool, bool),
    width: f64,
    swap_stereo: bool,
}

impl<E: Effect> AudioUtility<E> {
    pub fn new(effect: E) -> Self {
        Self {
            inner: effect,
            gain: 1.0,
            pan: 0.0,
            panning_law: PanningLaw::default(),
            invert: (false, false),
            width: 0.0,
            swap_stereo: false,
        }
    }

    pub fn set_gain_db(&mut self, gain_db: f64) {
        self.gain = db_to_level(gain_db);
    }

    pub fn set_gain(&mut self, gain: f64) {
        self.gain = gain;
    }

    /// `-1.0` is hard-left panning; `0.0` is centred; `1.0` is hard-right panning.
    pub fn set_pan(&mut self, pan: f64) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    pub fn set_panning_law(&mut self, panning_law: PanningLaw) {
        self.panning_law = panning_law;
    }

    /// `0.0` does not affect the signal; `-1.0` is 100% mid; `1.0` is 100% side.
    pub fn set_width(&mut self, width: f64) {
        self.width = width.clamp(-1.0, 1.0);
    }

    pub fn swap_stereo(&mut self, swap_stereo: bool) {
        self.swap_stereo = swap_stereo;
    }

    pub fn invert_left(&mut self, inverted: bool) {
        self.invert.0 = inverted;
    }

    pub fn invert_right(&mut self, inverted: bool) {
        self.invert.1 = inverted;
    }

    fn process_gain(&self, left: f64, right: f64) -> (f64, f64) {
        let gain_l = if self.invert.0 { -1.0 } else { 1.0 } * self.gain;
        let gain_r = if self.invert.1 { -1.0 } else { 1.0 } * self.gain;

        (left * gain_l, right * gain_r)
    }

    fn process_panning(&self, mut left: f64, mut right: f64) -> (f64, f64) {
        let pan = (self.pan + 1.0) / 2.0;

        if self.swap_stereo {
            std::mem::swap(&mut left, &mut right);
        }

        match self.panning_law {
            PanningLaw::Linear => {
                left *= 1.0 - pan;
                right *= pan;
            }
            // visual: https://www.desmos.com/calculator/khvab9wbqi
            PanningLaw::ConstantPower => {
                left *= (pan * FRAC_PI_2).cos();
                right *= (pan * FRAC_PI_2).sin();
            }
        }

        (left, right)
    }

    // TODO: test this
    fn process_width(&self, mut left: f64, mut right: f64) -> (f64, f64) {
        let mut mid = (left + right) * 0.5;
        let mut side_l = left - right;
        let mut side_r = right - left;

        if -1.0 <= self.width && self.width < 0.0 {
            side_l *= 1.0 + self.width;
            side_r *= 1.0 + self.width;
        } else if 0.0 <= self.width && self.width <= 1.0 {
            mid *= 1.0 - self.width;
        }

        left = mid + side_l;
        right = mid + side_r;

        (left, right)
    }
}

impl<E: Effect + Clone> Effect for AudioUtility<E> {
    fn process_mono(&mut self, input: f64, ch_idx: usize) -> f64 {
        let output = self.inner.process_mono(input, ch_idx);

        let (gain_l, gain_r) = self.process_gain(output, output);

        let (pan_l, pan_r) = self.process_panning(gain_l, gain_r);

        match ch_idx {
            0 => pan_l,
            1 => pan_r,
            _ => input,
        }
    }

    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let (pro_l, pro_r) = self.inner.process_stereo(in_l, in_r);

        let (gain_l, gain_r) = self.process_gain(pro_l, pro_r);

        // let (pan_l, pan_r) = self.process_panning(gain_l, gain_r);

        self.process_panning(gain_l, gain_r)
        // self.process_width(pan_l, pan_r)
    }

    fn get_sample_rate(&self) -> f64 {
        self.inner.get_sample_rate()
    }

    fn get_identifier(&self) -> &str {
        return "audio_utility";
    }
}

impl<E: Effect> Deref for AudioUtility<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<E: Effect> DerefMut for AudioUtility<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
