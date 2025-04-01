//! JUCE-based filter design. Unused in this project.

use crate::prelude::*;
use realfft::num_complex::{Complex64, ComplexFloat};
use std::cell::RefCell;
use std::rc::Rc;

const J: Complex64 = Complex64::new(0.0, 1.0);

pub trait Coefficients {
    fn new() -> Rc<RefCell<Self>>;
    fn with_coefs(coefs: &[f64]) -> Rc<RefCell<Self>>;
    fn with_capacity(capacity: usize) -> Rc<RefCell<Self>>;
    fn filter_order(&self) -> usize;
    fn magnitude_at_freq(&self, freq: f64, sample_rate: f64) -> f64;
    fn magnitude_for_freqs(&self, freqs: &[f64], sample_rate: f64) -> Vec<f64>;
    fn magnitude_for_freqs_in_place(&self, freqs: &[f64], magnitudes: &mut [f64], sample_rate: f64);
    fn phase_at_freq(&self, freq: f64, sample_rate: f64) -> f64;
    fn phase_for_freqs(&self, freqs: &[f64], sample_rate: f64) -> Vec<f64>;
    fn phase_for_freqs_in_place(&self, freqs: &[f64], phases: &mut [f64], sample_rate: f64);
    fn normalise(&mut self) {}
    fn get_coefs(&self) -> &[f64];
    fn get_coefs_mut(&mut self) -> &mut [f64];
}

pub struct FIRCoefficients {
    coefs: Vec<f64>,
}

impl Coefficients for FIRCoefficients {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { coefs: Vec::new() }))
    }

    fn with_coefs(coefs: &[f64]) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            coefs: Vec::from(coefs),
        }))
    }

    fn with_capacity(capacity: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            coefs: Vec::with_capacity(capacity),
        }))
    }

    fn filter_order(&self) -> usize {
        self.coefs.len() - 1
    }

    fn magnitude_at_freq(&self, freq: f64, sample_rate: f64) -> f64 {
        assert!(freq.is_sign_positive() && freq <= sample_rate / 2.0);

        let order = self.filter_order();

        let mut numerator = Complex64::new(0.0, 0.0);
        let mut factor = Complex64::new(1.0, 0.0);
        let jw = (-TAU * freq * J / sample_rate).exp();

        for co in self.coefs.iter().take(order) {
            numerator += co * factor;
            factor *= jw;
        }

        numerator.abs()
    }

    fn magnitude_for_freqs(&self, freqs: &[f64], sample_rate: f64) -> Vec<f64> {
        let mut v = Vec::with_capacity(freqs.len());

        self.magnitude_for_freqs_in_place(freqs, &mut v, sample_rate);

        v
    }

    fn magnitude_for_freqs_in_place(
        &self,
        freqs: &[f64],
        magnitudes: &mut [f64],
        sample_rate: f64,
    ) {
        assert!(magnitudes.len() >= freqs.len());
        let order = self.filter_order();

        for (i, &freq) in freqs.iter().enumerate() {
            let mut numerator = Complex64::new(0.0, 0.0);
            let mut factor = Complex64::new(1.0, 0.0);
            let jw = (-TAU * freq * J / sample_rate).exp();

            for co in self.coefs.iter().take(order) {
                numerator += co * factor;
                factor *= jw;
            }

            magnitudes[i] = numerator.abs();
        }
    }

    fn phase_at_freq(&self, freq: f64, sample_rate: f64) -> f64 {
        assert!(freq.is_sign_positive() && freq <= sample_rate / 2.0);

        let order = self.filter_order();

        let mut numerator = Complex64::new(0.0, 0.0);
        let mut factor = Complex64::new(1.0, 0.0);

        let jw = (-TAU * freq * J / sample_rate).exp();
        for co in self.coefs.iter().take(order) {
            numerator += co * factor;
            factor *= jw;
        }

        numerator.arg()
    }

    fn phase_for_freqs(&self, freqs: &[f64], sample_rate: f64) -> Vec<f64> {
        let mut v = Vec::with_capacity(freqs.len());

        self.phase_for_freqs_in_place(freqs, &mut v, sample_rate);

        v
    }

    fn phase_for_freqs_in_place(&self, freqs: &[f64], phases: &mut [f64], sample_rate: f64) {
        assert!(phases.len() >= freqs.len());

        let order = self.filter_order();

        for (i, &freq) in freqs.iter().enumerate() {
            let mut numerator = Complex64::new(0.0, 0.0);
            let mut factor = Complex64::new(1.0, 0.0);
            let jw = (-TAU * freq * J / sample_rate).exp();

            for co in self.coefs.iter().take(order) {
                numerator += co * factor;
                factor *= jw;
            }

            phases[i] = numerator.arg();
        }
    }

    fn normalise(&mut self) {
        let mag: f64 = self.coefs.iter().map(|co| co * co).sum();

        self.coefs
            .iter_mut()
            .for_each(|co| *co *= (4.0 * mag.sqrt()).recip());
    }

    fn get_coefs(&self) -> &[f64] {
        &self.coefs
    }

    fn get_coefs_mut(&mut self) -> &mut [f64] {
        &mut self.coefs
    }
}

pub struct IIRCoefficients {
    coefs: Vec<f64>,
}

impl Coefficients for IIRCoefficients {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { coefs: vec![] }))
    }

    fn with_coefs(coefs: &[f64]) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            coefs: Vec::from(coefs),
        }))
    }

    fn with_capacity(capacity: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            coefs: Vec::with_capacity(capacity),
        }))
    }

    fn filter_order(&self) -> usize {
        (self.coefs.len() - 1) / 2
    }

    fn magnitude_at_freq(&self, freq: f64, sample_rate: f64) -> f64 {
        assert!(freq.is_sign_positive() && freq <= sample_rate * 0.5);
        let order = self.filter_order();

        let mut numerator = Complex64::new(0.0, 0.0);
        let mut denominator = Complex64::new(0.0, 0.0);
        let mut factor = Complex64::new(1.0, 0.0);

        let jw = (-TAU * freq * J / sample_rate).exp();

        for co in self.coefs.iter().take(order) {
            numerator += co * factor;
            factor *= jw;
        }

        denominator = Complex64::new(1.0, 0.0);
        factor = jw;

        for i in (order + 1)..=(2 * order) {
            denominator += self.coefs[i] * factor;
            factor *= jw;
        }

        (numerator / denominator).abs()
    }

    fn magnitude_for_freqs(&self, _freqs: &[f64], _sample_rate: f64) -> Vec<f64> {
        unimplemented!()
    }

    fn magnitude_for_freqs_in_place(
        &self,
        _freqs: &[f64],
        _magnitudes: &mut [f64],
        _sample_rate: f64,
    ) {
        unimplemented!()
    }

    fn phase_at_freq(&self, freq: f64, sample_rate: f64) -> f64 {
        assert!(freq.is_sign_positive() && freq <= sample_rate * 0.5);

        let order = self.filter_order();
        let mut numerator = Complex64::new(0.0, 0.0);
        let mut denominator = Complex64::new(0.0, 0.0);
        let mut factor = Complex64::new(1.0, 0.0);

        let jw = (-TAU * freq * J / sample_rate).exp();

        for co in self.coefs.iter().take(order) {
            numerator += co * factor;
            factor *= jw;
        }

        denominator = Complex64::new(1.0, 0.0);
        factor = jw;

        for i in (order + 1)..=(2 * order) {
            denominator += self.coefs[i] * factor;
            factor *= jw;
        }

        (numerator / denominator).arg()
    }

    fn phase_for_freqs(&self, _freqs: &[f64], _sample_rate: f64) -> Vec<f64> {
        unimplemented!()
    }

    fn phase_for_freqs_in_place(&self, _freqs: &[f64], _phases: &mut [f64], _sample_rate: f64) {
        unimplemented!()
    }

    fn get_coefs(&self) -> &[f64] {
        &self.coefs
    }

    fn get_coefs_mut(&mut self) -> &mut [f64] {
        &mut self.coefs
    }
}
