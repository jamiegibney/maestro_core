use super::*;

#[derive(Clone, Default, Debug)]
struct LRFCoefs {
    g: f64,
    h: f64,
    r2: f64,
}

/// A 4th order Linkwitz-Riley filter, which is commonly used for crossovers. See the
/// [`process_high_low()`](Self::process_high_low) method for obtaining a low- and
/// high-passed output simultaneously.
///
/// The sum of the low- and high-passed outputs (at the same cutoff) are equivalent
/// to an allpass filter at that cutoff with a flat magnitude response, hence why they
/// are favored for crossovers.
///
/// Only supports lowpass, highpass and allpass filter types.
///
/// Based on the Topology-Preserving Transform (TPT) filter structure, found in the JUCE
/// framework.
#[derive(Clone, Debug)]
pub struct LinkwitzRileyFilter {
    /// Filter coefficients.
    coefs: LRFCoefs,

    /// Delayed sample buffer. Must hold 4 samples for each channel.
    delayed: Vec<f64>,

    /// Filter cutoff frequency.
    cutoff: f64,

    /// Filter type.
    filter_type: FilterType,

    /// The internal sample rate.
    sample_rate: f64,
}

impl LinkwitzRileyFilter {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            coefs: LRFCoefs::default(),
            delayed: vec![0.0; 8],
            cutoff: 440.0,
            filter_type: FilterType::Lowpass,
            sample_rate,
        }
    }

    /// # Panics
    ///
    /// Panics if `filter_type` is not `Lowpass`, `Highpass`, or `Allpass`.
    pub fn set_type(&mut self, filter_type: FilterType) {
        if !matches!(
            filter_type,
            FilterType::Lowpass | FilterType::Highpass | FilterType::Allpass
        ) {
            panic!("{filter_type:?} is not yet supported for Linkwitz-Riley filters.");
        }

        self.filter_type = filter_type;
    }

    pub fn set_cutoff_freq(&mut self, freq_hz: f64) {
        self.cutoff = freq_hz;
        self.update();
    }

    pub fn reset(&mut self, value: f64) {
        self.delayed.iter_mut().for_each(|x| *x = value);
    }

    pub fn process_high_low(&mut self, in_l: f64, in_r: f64) -> ((f64, f64), (f64, f64)) {
        let LRFCoefs { g, h, r2 } = self.coefs;
        let input = [in_l, in_r];
        let mut high = [0.0, 0.0];
        let mut high_2 = [0.0, 0.0];
        let mut band = [0.0, 0.0];
        let mut band_2 = [0.0, 0.0];
        let mut low = [0.0, 0.0];
        let mut low_2 = [0.0, 0.0];

        for ch in 0..2 {
            high[ch] =
                (input[ch] - (r2 + g) * (&self.delayed[..2])[ch] - (&self.delayed[2..4])[ch]) * h;

            band[ch] = g * high[ch] + (&self.delayed[..2])[ch];
            self.delayed[ch] = g * high[ch] + band[ch];

            low[ch] = g * band[ch] + (&self.delayed[2..4])[ch];
            self.delayed[2 + ch] = g * band[ch] + low[ch];

            high_2[ch] =
                (low[ch] - (r2 + g) * (&self.delayed[4..6])[ch] - (&self.delayed[6..8])[ch]) * h;

            band_2[ch] = g * high_2[ch] + (&self.delayed[4..6])[ch];
            self.delayed[4 + ch] = g * high_2[ch] + band_2[ch];

            low_2[ch] = g * band_2[ch] + (&self.delayed[6..8])[ch];
            self.delayed[6 + ch] = g * band_2[ch] + low_2[ch];
        }

        let low_out = (low_2[0], low_2[1]);
        let high_out = (
            low[0] - r2 * band[0] + high[0] - low_2[0],
            low[1] - r2 * band[1] + high[1] - low_2[1],
        );

        (low_out, high_out)
    }

    fn update(&mut self) {
        let LRFCoefs { g, h, r2 } = &mut self.coefs;

        *g = (PI * self.cutoff / self.sample_rate).tan();
        *r2 = SQRT_2;
        *h = (1.0 + *r2 * *g + *g * *g).recip();
    }
}

impl Effect for LinkwitzRileyFilter {
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let LRFCoefs { g, h, r2 } = self.coefs;
        let input = [in_l, in_r];
        let mut high = [0.0, 0.0];
        let mut band = [0.0, 0.0];
        let mut low = [0.0, 0.0];

        for ch in 0..2 {
            high[ch] =
                (input[ch] - (r2 + g) * (&self.delayed[..2])[ch] - (&self.delayed[2..4])[ch]) * h;

            band[ch] = g * high[ch] + (&self.delayed[..2])[ch];
            self.delayed[ch] = g * high[ch] + band[ch];

            low[ch] = g * band[ch] + (&self.delayed[2..4])[ch];
            self.delayed[2 + ch] = g * band[ch] + low[ch];
        }

        if matches!(self.filter_type, FilterType::Allpass) {
            return (
                low[0] - r2 * band[0] + high[0],
                low[1] - r2 * band[1] + high[1],
            );
        }

        let mut high_2 = [0.0, 0.0];
        let band_2 = [0.0, 0.0];
        let mut low_2 = [0.0, 0.0];

        for ch in 0..2 {
            high_2[ch] = (if matches!(self.filter_type, FilterType::Lowpass) {
                low[ch]
            } else {
                high[ch]
            }) - (r2 + g) * (&self.delayed[4..6])[ch]
                - (&self.delayed[6..8])[ch] * h;

            band[ch] = g * high_2[ch] + (&self.delayed[4..6])[ch];
            self.delayed[4 + ch] = g * high_2[ch] + band_2[ch];

            low_2[ch] = g * band_2[ch] + (&self.delayed[6..8])[ch];
            self.delayed[6 + ch] = g * band_2[ch] + low_2[ch];
        }

        match self.filter_type {
            FilterType::Lowpass => (low_2[0], low_2[1]),
            FilterType::Highpass => (high_2[0], high_2[1]),
            // this should be impossible to match, but leave the input alone just in case
            _ => (in_l, in_r),
        }
    }

    fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn get_identifier(&self) -> &str {
        "linkwitz_riley_filter"
    }
}
