use super::*;

#[derive(Clone, Default, Debug)]
struct SVFCoefs {
    g: f64,
    h: f64,
    r2: f64,
}

#[derive(Clone, Debug)]
pub struct StateVariableFilter {
    coefs: SVFCoefs,

    z1: Vec<f64>,
    z2: Vec<f64>,

    cutoff_freq: f64,
    q: f64,

    filter_type: FilterType,

    sample_rate: f64,
}

impl StateVariableFilter {
    pub fn new(num_channels: usize, sample_rate: f64) -> Self {
        Self {
            coefs: SVFCoefs::default(),
            z1: vec![0.0; num_channels],
            z2: vec![0.0; num_channels],
            cutoff_freq: 440.0,
            q: BUTTERWORTH_Q,
            filter_type: FilterType::Lowpass,
            sample_rate,
        }
    }

    pub fn set_type(&mut self, filter_type: FilterType) {
        if !matches!(
            filter_type,
            FilterType::Lowpass | FilterType::Highpass | FilterType::Bandpass
        ) {
            panic!("{filter_type:?} is not yet a supported type for state variable filters");
        }

        self.filter_type = filter_type;
    }

    pub fn set_cutoff_freq(&mut self, freq: f64) {
        assert!(freq.is_sign_positive() && freq <= self.sample_rate / 2.0);
        self.cutoff_freq = freq;
        self.update();
    }

    pub fn set_q(&mut self, q: f64) {
        assert!(q.is_sign_positive());
        self.cutoff_freq = q;
        self.update();
    }

    pub fn set_num_channels(&mut self, num_channels: usize) {
        self.z1.resize(num_channels, 0.0);
        self.z2.resize(num_channels, 0.0);
    }

    pub fn reset(&mut self, value: f64) {
        self.z1.iter_mut().for_each(|x| *x = value);
        self.z2.iter_mut().for_each(|x| *x = value);
    }

    fn update(&mut self) {
        let SVFCoefs { g, h, r2 } = &mut self.coefs;
        let sr = unsafe { SAMPLE_RATE };

        *g = (PI * self.cutoff_freq / sr).tan();
        *r2 = self.q.recip();
        *h = (1.0 + *r2 * *g + *g * *g).recip();
    }
}

impl Effect for StateVariableFilter {
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let SVFCoefs { g, h, r2 } = self.coefs;
        let input = [in_l, in_r];

        let mut high = [0.0, 0.0];
        let mut band = [0.0, 0.0];
        let mut low = [0.0, 0.0];

        for ch in 0..2 {
            let ls_1 = self.z1[ch];
            let ls_2 = self.z2[ch];

            high[ch] = h * (input[ch] - ls_1 * (g + r2) - ls_2);

            band[ch] = high[ch] * g + ls_1;
            self.z1[ch] = high[ch] * g + band[ch];

            low[ch] = band[ch] * g + ls_2;
            self.z2[ch] = band[ch] * g + low[ch];
        }

        match self.filter_type {
            FilterType::Lowpass => (low[0], low[1]),
            FilterType::Highpass => (high[0], high[1]),
            FilterType::Bandpass => (band[0], band[1]),
            _ => (in_l, in_r),
        }
    }

    fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn get_identifier(&self) -> &str {
        "state_variable_filter"
    }
}
