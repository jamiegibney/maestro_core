//! Dry/wet mix wrapper for `impl `[`Effect`] types.

use super::Effect;
use crate::prelude::*;
use std::ops::{Deref, DerefMut};

/// A simple dry-wet wrapper around an `impl `[`Effect`].
#[derive(Clone, Debug, Default)]
pub struct DryWet<E: Effect> {
    dry: Smoother<f64>,
    wet: Smoother<f64>,
    effect: E,
}

impl<E: Effect> DryWet<E> {
    pub fn new(effect: E) -> Self {
        let sr = effect.get_sample_rate();

        Self {
            dry: Smoother::new(5.0, 0.0, sr),
            wet: Smoother::new(5.0, 1.0, sr),
            effect,
        }
    }

    pub fn set_dry(&mut self, dry_level: f64) {
        self.dry.set_target_value(dry_level);
    }

    pub fn set_dry_db(&mut self, dry_db: f64) {
        self.dry.set_target_value(db_to_level(dry_db));
    }

    pub fn set_wet(&mut self, wet_level: f64) {
        self.wet.set_target_value(wet_level);
    }

    pub fn set_wet_db(&mut self, wet_db: f64) {
        self.wet.set_target_value(db_to_level(wet_db));
    }

    /// `mix == 0.0` is 100% dry, and `mix == 1.0` is 100% wet.
    ///
    /// `mix` is clamped between `0.0` and `1.0`.
    pub fn set_mix_equal_gain(&mut self, mut mix: f64) {
        mix = mix.clamp(0.0, 1.0);

        self.set_dry(mix);
        self.set_wet(1.0 - mix);
    }

    /// `mix == 0.0` is 100% dry, and `mix == 1.0` is 100% wet.
    ///
    /// `mix` is clamped between `0.0` and `1.0`.
    pub fn set_mix_equal_power(&mut self, mut mix: f64) {
        mix = mix.clamp(0.0, 1.0);

        self.set_dry((FRAC_PI_2 * mix).cos());
        self.set_wet((FRAC_PI_2 * mix).sin());
    }

    /// Unwraps the contained effect.
    pub fn unwrap(self) -> E {
        self.effect
    }

    fn dry_wet_next(&mut self) -> (f64, f64) {
        (self.dry.next(), self.wet.next())
    }
}

impl<E: Effect> Deref for DryWet<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.effect
    }
}

impl<E: Effect> DerefMut for DryWet<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.effect
    }
}

impl<E: Effect + Clone> Effect for DryWet<E> {
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let (dry, wet) = self.dry_wet_next();
        let (sig_l, sig_r) = self.effect.process_stereo(in_l, in_r);

        (dry * in_l + wet * sig_l, dry * in_r + wet * sig_r)
    }

    fn process_mono(&mut self, input: f64, ch_idx: usize) -> f64 {
        let (dry, wet) = self.dry_wet_next();
        let sig = self.effect.process_mono(input, ch_idx);

        dry * input + wet * sig
    }

    fn get_sample_rate(&self) -> f64 {
        self.effect.get_sample_rate()
    }

    fn get_identifier(&self) -> &str {
        "dry_wet"
    }
}
