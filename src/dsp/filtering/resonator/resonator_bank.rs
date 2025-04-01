//! Resonator bank with musical features.

use super::*;
use crate::dsp::*;
use crate::prelude::*;
use two_pole_resonator::TwoPoleResonator;

type Resonator = AudioUtility<StereoWrapper<TwoPoleResonator>>;

#[derive(Clone, Debug, Default)]
pub struct ResonatorBankParams {
    pub root_note: f64,
    pub scale: Scale,

    /// How much panning is applied to each resonator.
    pub panning_scale: f64,
    /// Whether each resonator's left and right filter should have the same pitch.
    // stereo_link: bool,
    /// Whether each resonator's pitch should be quantised to the musical scale.
    pub quantize_to_scale: bool,
    /// The overall range of (original) resonator pitches.
    pub freq_spread: f64,
    /// The overall shift in (original) resonator pitches.
    pub freq_shift: f64,
    /// The amount each pitch can skew towards its original value.
    pub inharm: f64,
}

#[derive(Clone, Debug, Default)]
pub struct ResoBankData {
    pub pitches: Vec<f64>,
    pub panning: Vec<f64>,
}

impl ResoBankData {
    pub fn new(size: usize) -> Self {
        Self { pitches: vec![0.0; size], panning: vec![0.0; size] }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ResonatorBank {
    resonators: Vec<Resonator>,
    original_pitches: Vec<f64>,
    active_pitches: Vec<Smoother<f64>>,
    panning: Vec<Smoother<f64>>,
    params: ResonatorBankParams,
    num_active: usize,
}

impl ResonatorBank {
    pub const NOTE_MIN: f64 = 30.0;
    pub const NOTE_MIDDLE: f64 = 79.0;
    pub const NOTE_MAX: f64 = 128.0;
    const INHARM_SCALE: f64 = 0.015;

    pub fn new(sample_rate: f64, max_num_resonators: usize) -> Self {
        assert!(max_num_resonators > 0);

        let pitch_smoother = Smoother::new(200.0, 69.0, sample_rate)
            .with_smoothing_type(SmoothingType::Cosine);
        let pan_smoother = Smoother::new(100.0, 0.0, sample_rate);

        let mut s = Self {
            resonators: vec![
                AudioUtility::new(StereoWrapper::from_single(
                    TwoPoleResonator::new(sample_rate)
                ));
                max_num_resonators
            ],
            original_pitches: vec![0.0; max_num_resonators],
            active_pitches: vec![pitch_smoother; max_num_resonators],
            panning: vec![pan_smoother; max_num_resonators],
            num_active: max_num_resonators,
            params: ResonatorBankParams {
                panning_scale: 1.0,
                freq_shift: 0.0,
                freq_spread: 0.0,
                root_note: 69.0,
                quantize_to_scale: false,
                scale: Scale::default(),
                inharm: 0.0,
            },
        };

        s.resonators.iter_mut().for_each(|res| {
            res.l.set_resonance(0.9999);
            res.r.set_resonance(0.9999);
            res.set_gain_db(-75.0);
        });

        s
    }

    pub fn set_params(&mut self, params: ResonatorBankParams) {
        self.params = params;
        self.set_active_pitches();
    }

    /// Sets the number of active resonators in the bank.
    ///
    /// # Panics
    ///
    /// Panics if `num_resonators > self.max_num_resonators()`.
    pub fn set_num_resonators(&mut self, num_resonators: usize) {
        if num_resonators == self.resonators.len() {
            return;
        }

        assert!(
            num_resonators <= self.resonators.capacity() && num_resonators != 0
        );

        self.num_active = num_resonators;
    }

    /// Returns the maximum number of resonators available in the bank.
    pub fn max_num_resonators(&self) -> usize {
        self.resonators.capacity()
    }

    /// Randomizes the raw pitches of the resonator bank.
    pub fn randomize(&mut self) {
        self.original_pitches.iter_mut().for_each(|p| {
            *p = random_range(Self::NOTE_MIN, Self::NOTE_MAX);
        });

        self.set_active_pitches();
        self.randomize_panning();
    }

    pub fn quantize_to_scale(&mut self, quantize_to_scale: bool) {
        if quantize_to_scale == self.params.quantize_to_scale {
            return;
        }

        self.params.quantize_to_scale = quantize_to_scale;
        self.set_active_pitches();
    }

    /// Scales the panning applied to each resonator.
    ///
    /// Clamped to `[0.0 - 1.0]`.
    pub fn set_panning_scale(&mut self, scale: f64) {
        if epsilon_eq(self.params.panning_scale, scale) {
            return;
        }

        self.params.panning_scale = scale.clamp(0.0, 1.0);
        self.update_panning();
    }

    pub fn set_freq_spread(&mut self, spread: f64) {
        if epsilon_eq(self.params.freq_spread, spread) {
            return;
        }

        self.params.freq_spread = spread.clamp(0.0, 1.0);
        self.set_active_pitches();
    }

    pub fn set_freq_shift(&mut self, shift: f64) {
        if epsilon_eq(self.params.freq_shift, shift) {
            return;
        }

        self.params.freq_shift = shift;
        self.set_active_pitches();
    }

    /// Sets the root note of the bank's scale.
    ///
    /// Only active if `quantise_to_scale` is true.
    pub fn set_root_note(&mut self, root_note_midi: f64) {
        if epsilon_eq(self.params.root_note, root_note_midi) {
            return;
        }

        self.params.root_note = root_note_midi;

        if self.params.quantize_to_scale {
            self.set_active_pitches();
        }
    }

    /// Sets the internal musical scale of the bank.
    ///
    /// Only active if `quantise_to_scale` is true.
    pub fn set_scale(&mut self, scale: Scale) {
        self.params.scale = scale;

        if self.params.quantize_to_scale {
            self.set_active_pitches();
        }
    }

    /// Sets how much each resonator pitch skews towards its original pitch.
    ///
    /// Only active if `quantise_to_scale` is true.
    pub fn set_inharm(&mut self, inharm: f64) {
        if epsilon_eq(self.params.inharm, inharm) {
            return;
        }

        self.params.inharm = inharm.clamp(0.0, 1.0) * Self::INHARM_SCALE;

        if self.params.quantize_to_scale {
            self.set_active_pitches();
        }
    }

    /// Returns a mutable reference to the raw resonator pitches.
    pub fn original_pitches_mut(&mut self) -> &mut [f64] {
        &mut self.original_pitches
    }

    /// Returns a mutable reference to the raw pan values.
    ///
    /// Each value is controlled by a smoother, so use the
    /// [`set_target_value()`](Smoother::set_target_value) method
    /// to change the value.
    pub fn pan_values_mut(&mut self) -> &mut [Smoother<f64>] {
        &mut self.panning
    }

    pub fn set_state_from_data(&mut self, data: &ResoBankData) {
        let mut should_update_panning = false;
        let mut should_update_pitches = false;

        let len = data.pitches.len().min(self.resonators.len());

        for i in 0..len {
            if !epsilon_eq(self.original_pitches[i], data.pitches[i]) {
                should_update_pitches = true;
                self.original_pitches[i] = data.pitches[i];
            }
            if !epsilon_eq(self.panning[i].target_value(), data.panning[i]) {
                should_update_panning = true;
                self.panning[i].set_target_value(data.panning[i]);
            }
        }

        if should_update_pitches {
            self.set_active_pitches();
        }
        if should_update_panning {
            self.update_panning();
        }
    }

    /// Returns a reference to the internal resonators.
    pub fn inner(&self) -> &[Resonator] {
        &self.resonators
    }

    /// Returns a mutable reference to the internal resonators.
    pub fn inner_mut(&mut self) -> &mut [Resonator] {
        &mut self.resonators
    }

    /// Randomises the panning values for each resonator.
    fn randomize_panning(&mut self) {
        self.panning.iter_mut().for_each(|pan| {
            pan.set_target_value(random_range(-1.0, 1.0));
        });

        self.update_panning();
    }

    /// Updates the panning value of each resonator.
    fn update_panning(&mut self) {
        self.resonators
            .iter_mut()
            .zip(self.panning.iter_mut())
            .for_each(|(res, pan)| {
                // apply randomised pan values, scaled by panning param

                // whilst not ideal, calling next each sample is OK as the smoothers
                // have low overhead when they are already at their target value.
                res.set_pan(pan.next() * self.params.panning_scale);
            });
    }

    /// Updates each resonator's pitch.
    fn update_resonator_pitches(&mut self) {
        // avoid recalculating filter coefs if the pitches haven't changed
        if !self.active_pitches[0].is_active() {
            return;
        }
        let nyquist = self.get_sample_rate() * 0.5;

        self.resonators
            .iter_mut()
            .take(self.num_active)
            .zip(self.active_pitches.iter_mut())
            .for_each(|(res, p)| {
                let note = p.next();
                let freq = note_to_freq(note);

                res.l.set_cutoff(freq.min(nyquist));
                res.r.set_cutoff(freq.min(nyquist));
            });

        for (res, p) in self
            .resonators
            .iter_mut()
            .skip(self.num_active)
            .zip(self.active_pitches.iter_mut())
        {
            if !p.is_active() {
                continue;
            }

            let note = p.next();
            let freq = note_to_freq(note);

            res.l.set_cutoff(freq.min(nyquist));
            res.r.set_cutoff(freq.min(nyquist));
        }
    }

    fn set_active_pitches(&mut self) {
        for (active, &original) in self
            .active_pitches
            .iter_mut()
            .take(self.num_active)
            .zip(self.original_pitches.iter())
        {
            // apply frequency spread and then add shift
            let spread_shift = if original < Self::NOTE_MIDDLE {
                map(
                    original,
                    Self::NOTE_MIN,
                    Self::NOTE_MIDDLE,
                    lerp(
                        Self::NOTE_MIDDLE,
                        Self::NOTE_MIN,
                        self.params.freq_spread,
                    ),
                    Self::NOTE_MIDDLE,
                )
            }
            else {
                map(
                    original,
                    Self::NOTE_MIDDLE,
                    Self::NOTE_MAX,
                    Self::NOTE_MIDDLE,
                    lerp(
                        Self::NOTE_MIDDLE,
                        Self::NOTE_MAX,
                        self.params.freq_spread,
                    ),
                )
            } + self.params.freq_shift;

            if self.params.quantize_to_scale {
                // quantize to scale
                let quantized = self
                    .params
                    .scale
                    .quantize_to_scale(spread_shift, self.params.root_note);

                // apply inharmonic skew
                active.set_target_value(lerp(
                    quantized, original, self.params.inharm,
                ));
            }
            else {
                active.set_target_value(spread_shift);
            }
        }
    }
}

impl Effect for ResonatorBank {
    fn process_mono(&mut self, input: f64, ch_idx: usize) -> f64 {
        self.update_resonator_pitches();

        let mut output = 0.0;
        for res in self.resonators.iter_mut().take(self.num_active) {
            output += res.process_mono(input, ch_idx);
        }

        for res in self.resonators.iter_mut().skip(self.num_active) {
            output += res.process_mono(0.0, ch_idx);
        }

        output
    }

    fn process_stereo(&mut self, mut left: f64, mut right: f64) -> (f64, f64) {
        self.update_resonator_pitches();

        let (mut out_l, mut out_r) = (0.0, 0.0);
        for res in self.resonators.iter_mut().take(self.num_active) {
            let (l, r) = res.process_stereo(left, right);
            out_l += l;
            out_r += r;
        }

        for res in self.resonators.iter_mut().skip(self.num_active) {
            let (l, r) = res.process_stereo(0.0, 0.0);
            out_l += l;
            out_r += r;
        }

        (out_l, out_r)
    }

    fn get_sample_rate(&self) -> f64 {
        self.resonators[0].get_sample_rate()
    }

    fn get_identifier(&self) -> &str {
        "resonator_bank"
    }
}
