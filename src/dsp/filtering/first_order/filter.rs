use super::*;

use std::f64::consts::TAU;

#[derive(Debug, Clone)]
struct Coefs {
    a0: f64,
    a1: f64,
    b1: f64,
}

impl Coefs {
    pub fn identity() -> Self {
        Self {
            a0: 1.0,
            a1: 0.0,
            b1: 0.0,
        }
    }
}

impl Default for Coefs {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Clone, Default, Debug)]
pub struct FirstOrderFilter {
    coefs: Coefs,
    z1: f64,

    freq: f64,
    filter_type: FilterType,
    sample_rate: f64,
}

impl crate::dsp::Effect for FirstOrderFilter {
    fn process_mono(&mut self, input: f64, _: usize) -> f64 {
        self.process(input)
    }

    fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn get_identifier(&self) -> &str {
        "first_order_filter"
    }
}

impl Filter for FirstOrderFilter {
    fn process(&mut self, sample: f64) -> f64 {
        let Coefs { a0, a1, b1 } = self.coefs;

        match self.filter_type {
            FilterType::Lowpass => self.set_lowpass_coefs(),
            FilterType::Highpass => self.set_highpass_coefs(),
            _ => {
                self.identity();
                dbg!(
                    &self.filter_type,
                    "only low/highpass filters are implemented for first order filters"
                );
            }
        }

        let output = a0.mul_add(sample, self.z1);
        self.z1 = a1.mul_add(sample, -b1 * output);

        output
    }
}

impl FirstOrderFilter {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sample_rate,
            ..Self::default()
        }
    }

    pub fn reset_sample_rate(&mut self, new_sample_rate: f64) {
        self.sample_rate = new_sample_rate;
    }

    pub fn identity(&mut self) {
        self.coefs = Coefs::identity();
    }

    pub fn set_freq(&mut self, freq: f64) {
        self.freq = freq;
        self.debug_assertions();
    }

    pub fn set_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
    }

    pub fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn set_lowpass_coefs(&mut self) {
        self.set_common_coefs();
        let Coefs { a0, a1, b1 } = &mut self.coefs;

        *a0 = (1.0 + *b1) / 2.0;
        *a1 = *a0;
    }

    fn set_highpass_coefs(&mut self) {
        self.set_common_coefs();
        let Coefs { a0, a1, b1 } = &mut self.coefs;

        *a0 = (1.0 - *b1) * 0.5;
        *a1 = -(*a0);
    }

    fn set_common_coefs(&mut self) {
        let freq = self.freq;
        let sr = self.sample_rate;
        let phi = (TAU * freq) / sr;

        self.coefs.b1 = (-phi.cos()) / (1.0 + phi.sin());
    }

    fn debug_assertions(&self) {
        let freq = self.freq;
        let sr = self.sample_rate;

        debug_assert!(freq.is_sign_positive() && freq <= sr / 2.0);
    }
}
