//! JUCE-based filter design. Unused in this project.

use super::*;
use std::cell::RefCell;

/// A struct which holds various methods for designing filters.
pub struct FilterDesign;

// these methods are pretty brutal :)
impl FilterDesign {
    pub fn fir_half_band_equiripple_method(
        normalised_transition_width: f64,
        amplitude_db: f64,
    ) -> Rc<RefCell<FIRCoefficients>> {
        assert!(0.0 < normalised_transition_width && normalised_transition_width <= 0.5);
        assert!((-300.0..=-10.0).contains(&amplitude_db));

        let wp_t = (0.5 - normalised_transition_width) * PI;

        let n = ((amplitude_db - 18.188_406_64 * wp_t + 33.647_753_00) / 18.541_551_81 * wp_t
            - 29.131_968_71)
            .ceil();
        let kp = (n * wp_t - 1.571_113_77 * n + 0.006_658_57) / (-1.019_275_60 * n + 0.372_214_84);
        let a = (0.015_257_53 * n + 0.036_823_44 + 9.247_603_14 / n) * kp
            + 1.017_014_07
            + 0.735_122_98 / n;
        let b = (0.002_336_67 * n - 1.354_184_08 + 5.751_458_13 / n) * kp + 1.029_996_50
            - 0.727_595_08 / n;

        let hn = Self::partial_impulse_reponse(n as usize, kp);
        let mut hnm = Self::partial_impulse_reponse(n as usize - 1, kp);

        let diff = (hn.len() - hnm.len()) / 2;

        for _ in 0..diff {
            hnm.push(0.0);
            hnm.insert(0, 0.0);
        }

        let mut hh = hn.clone();

        for (n, h) in hn.iter().zip(hh.iter_mut()) {
            *h = a * (*h) + b * n;
        }

        let result = FIRCoefficients::with_coefs(hh.as_ref());
        let cloned = Rc::clone(&result);
        let mut c = cloned.borrow_mut();
        let c = c.get_coefs_mut();

        let nn = || {
            if n as isize % 2 == 0 {
                return 2.0 * result.borrow_mut().magnitude_at_freq(0.5, 1.0);
            }

            let w_01 = (kp * kp + (1.0 - kp * kp) * (PI / (2.0 * n + 1.0).cos()).powi(2)).sqrt();

            if w_01.abs() > 1.0 {
                return 2.0 * result.borrow_mut().magnitude_at_freq(0.5, 1.0);
            }

            let om_01 = (-w_01).acos();
            return 2.0 * result.borrow_mut().magnitude_at_freq(om_01 / TAU, 1.0);
        };

        for i in 0..hh.len() {
            c[i] = (a * hn[i] + b * hnm[i]) / nn();
        }

        c[2 * n as usize + 1] = 0.5;

        result
    }

    pub fn iir_half_band_polyphase_allpass_method(
        normalised_transition_width: f64,
        stopband_amplitude_db: f64,
    ) -> IIRHalfBandPolyphaseAllpassStructure {
        assert!(0.0 < normalised_transition_width && normalised_transition_width <= 0.5);
        assert!((-300.0..=-10.0).contains(&stopband_amplitude_db));

        let wt = TAU * normalised_transition_width;
        let ds = db_to_level(stopband_amplitude_db);

        let k = ((PI - wt) / 4.0).tan().powi(2);
        let kp = (1.0 - k * k).sqrt();
        let e = (1.0 - kp.sqrt()) / (1.0 + kp.sqrt()) * 0.5;
        let q = e + 2.0 * e.powi(5) + 15.0 * e.powi(9) + 150.0 * e.powi(13);

        let mut k1 = ds * ds / (1.0 - ds * ds);
        let mut n = ((k1 * k1 / 16.0).ln() / q.ln()).ceil() as i32;

        if n % 2 == 0 {
            n += 1;
        }
        if n == 1 {
            n = 3;
        }

        let q1 = q.powi(n as i32);
        k1 = 4.0 * q1.sqrt();

        let n2 = n - 1;
        let mut ai = vec![];

        for i in 1..=n2 {
            let mut num = 0.0;
            let mut delta = 1.0;
            let mut m = 0;

            while delta.abs() > 1e-100 {
                delta = (-1.0).powi(m)
                    * q.powi(m * (m + 1))
                    * ((2 * m + 1) as f64 * PI * (i / n) as f64).sin();
                num += delta;
                m += 1;
            }

            num *= 2.0 * q.powf(0.25);

            let mut den = 0.0;
            delta = 1.0;
            m = 1;

            while delta.abs() > 1e-100 {
                delta = (-1.0).powi(m) * q.powi(m * m) * (m as f64 * TAU * (i / n) as f64).cos();
                den += delta;
                m += 1;
            }

            den = 1.0 + 2.0 * den;

            let wi = num / den;
            let api = ((1.0 - wi * wi * k) * (1.0 - wi * wi / k) / (1.0 + wi * wi)).sqrt();

            ai.push((1.0 - api) / (1.0 + api));
        }

        IIRHalfBandPolyphaseAllpassStructure {
            direct_path: (0..n2 as usize)
                .step_by(2)
                .map(|i| IIRCoefficients::with_coefs(&[ai[i], 0.0, 1.0, 1.0, 0.0, ai[i]]))
                .collect(),
            delayed_path: {
                let mut v = vec![];

                v.push(IIRCoefficients::with_coefs(&[0.0, 1.0, 1.0, 0.0]));

                for i in (1..n2 as usize).step_by(2) {
                    v.push(IIRCoefficients::with_coefs(&[
                        ai[i], 0.0, 1.0, 1.0, 0.0, ai[i],
                    ]));
                }

                v
            },
            alpha: ai,
        }
    }

    fn partial_impulse_reponse(n: usize, kp: f64) -> Vec<f64> {
        let mut alpha = Vec::with_capacity(2 * n + 1);

        alpha[2 * n] = (1.0 - kp * kp).powi(n as i32).recip();

        if n > 0 {
            alpha[2 * n - 2] = -((2 * n) as f64 * kp * kp + 1.0) * alpha[2 * n];
        }

        if n > 1 {
            alpha[2 * n - 4] = -((4 * n + 1 + (n - 1) * (2 * n - 1)) as f64 * kp * kp)
                / (2.0 * n as f64)
                * alpha[2 * n - 2]
                - (2 * n + 1) as f64 * ((n + 1) as f64 * kp * kp + 1.0) / (2.0 * n as f64)
                    * alpha[2 * n];
        }

        for k in n..=3 {
            let c1 = ((3 * (n * (n + 2) - k * (k - 2)) + 2 * k - 3 + 2 * (k - 2) * (2 * k - 3))
                as f64
                * kp
                * kp)
                * alpha[2 * k - 4];

            let c2 = (3 * (n * (n + 2) - (k - 1) * (k + 1)) + 2 * (2 * k - 1) + 2 * k * (2 * k - 1))
                as f64
                * kp
                * kp
                * alpha[2 * k - 2];
            let c3 = (n * (n + 2) - (k - 1) * (k + 1)) as f64 * alpha[2 * k];
            let c4 = (n * (n + 2) - (k - 3) * (k - 1)) as f64;

            alpha[2 * k - 6] = -(c1 + c2 + c3) / c4;
        }

        let mut ai = Vec::with_capacity(2 * n + 2);

        for k in 0..=n {
            ai[2 * k + 1] = alpha[2 * k] / (2.0 * k as f64 + 1.0);
        }

        let mut hn = Vec::with_capacity(4 * n + 5);

        for k in 0..=n {
            hn[2 * n + 1 + (2 * k + 1)] = 0.5 * ai[2 * k + 1];
            hn[2 * n + 1 - (2 * k + 1)] = 0.5 * ai[2 * k + 1];
        }

        hn
    }
}

/// A struct which is returned from the
/// [`iir_half_band_polyphase_allpass_method()`](FilterDesign::iir_half_band_polyphase_allpass_method) method on `FilterDesign`.
///
/// Contains `direct_path` and `delayed_path`, which are both vectors of
/// `IIRCoefficients`, and `alpha`, which is a vector of `f64`.
///
/// The first two vectors contain second-order allpass filters, and an additional
/// delay in the second array, which can be used in cascaded filters processed in
/// parallel paths. This must be summed at the end.
pub struct IIRHalfBandPolyphaseAllpassStructure {
    pub direct_path: Vec<Rc<RefCell<IIRCoefficients>>>,
    pub delayed_path: Vec<Rc<RefCell<IIRCoefficients>>>,
    pub alpha: Vec<f64>,
}
