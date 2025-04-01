//! First-order lowpass and highpass filters.

pub mod filter;
use super::*;

pub use filter::FirstOrderFilter;

#[cfg(test)]
mod tests {
    use super::*;
    use nannou::rand;

    #[test]
    #[should_panic]
    fn bad_freq_argument_1() {
        let sample_rate = 44100.0;
        let mut filter = FirstOrderFilter::new(sample_rate);
        filter.set_freq(sample_rate);
    }

    #[test]
    #[should_panic]
    fn bad_freq_argument_2() {
        let sample_rate = 44100.0;
        let mut filter = FirstOrderFilter::new(sample_rate);
        filter.set_freq(sample_rate / 2.0 + 0.00001);
    }

    #[test]
    fn unsupported_type_had_no_effect() {
        let sample_rate = 44100.0;
        let original: Vec<f64> = (0..1000).map(|_| rand::random_range(-1.0, 1.0)).collect();

        let mut filter = FirstOrderFilter::new(sample_rate);
        filter.set_freq(440.0);
        // unsupported filter type
        filter.set_type(FilterType::Bandpass);

        let filtered: Vec<f64> = original.iter().map(|x| filter.process(*x)).collect();

        assert_eq!(original, filtered);
    }

    #[test]
    fn high_low_pass_are_different() {
        let sample_rate = 44100.0;
        let samples: Vec<f64> = (0..1000).map(|_| rand::random_range(-1.0, 1.0)).collect();

        let mut filter = FirstOrderFilter::new(sample_rate);
        filter.set_freq(440.0);
        filter.set_type(FilterType::Lowpass);

        let low_passed: Vec<f64> = samples.iter().map(|x| filter.process(*x)).collect();

        filter = FirstOrderFilter::new(sample_rate);
        filter.set_freq(440.0);
        filter.set_type(FilterType::Highpass);

        let high_passed: Vec<f64> = samples.iter().map(|x| filter.process(*x)).collect();

        assert!(low_passed != high_passed);
    }
}
