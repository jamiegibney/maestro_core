//! Spectral filtering.

use super::{
    stft::stft_trait::{StftInput, StftInputMut},
    *,
};
use crate::util::window::*;
use nannou_audio::Buffer;
use realfft::{
    num_complex::Complex, ComplexToReal, RealFftPlanner, RealToComplex,
};
use std::sync::Arc;

pub mod mask;
use mask::*;

/// A spectral filtering processor, which accepts a `SpectralMask` as a frequency
/// mask and applies it to an audio signal in the frequency domain.
pub struct SpectralFilter {
    /// stft processor
    stft: StftHelper,

    /// a window function with gain compensation
    compensated_window_function: Vec<f64>,

    window_function: Vec<f64>,

    /// frequency domain buffers
    complex_buffers: Vec<Vec<Complex<f64>>>,

    /// forward fft plan
    fft: Arc<dyn RealToComplex<f64>>,

    /// inverse fft plan
    ifft: Arc<dyn ComplexToReal<f64>>,

    /// dry input data
    dry_buffer: Vec<f64>,

    mix: Smoother<f64>,

    /// filter mask
    mask: SpectralMask,
}

impl SpectralFilter {
    const OVERLAP_FACTOR: usize = 4;

    /// # Panics
    ///
    /// Panics if `num_channels` or `max_block_size` is `0`.
    pub fn new(num_channels: usize, max_block_size: usize) -> Self {
        Self {
            stft: StftHelper::new(num_channels, max_block_size, 0),

            compensated_window_function: hann(max_block_size)
                .into_iter()
                .map(|x| {
                    x * ((max_block_size * Self::OVERLAP_FACTOR) as f64).recip()
                })
                .collect(),

            window_function: hann(max_block_size),

            complex_buffers: vec![
                vec![
                    Complex::default();
                    max_block_size / 2 + 1
                ];
                num_channels
            ],

            fft: RealFftPlanner::new().plan_fft_forward(max_block_size),
            ifft: RealFftPlanner::new().plan_fft_inverse(max_block_size),

            dry_buffer: vec![0.0; max_block_size * num_channels],

            mix: Smoother::new(30.0, 1.0, unsafe { SAMPLE_RATE }),

            mask: SpectralMask::new(max_block_size)
                .with_size(max_block_size / 2),
        }
    }

    /// # Panics
    ///
    /// Panics if `block_size` is greater than the max block size of the processor.
    pub fn set_block_size(&mut self, block_size: usize) {
        assert!(block_size <= self.stft.max_block_size());

        let compensation_factor = self.compensation_factor(block_size);

        // window function
        self.window_function = hann(block_size);

        self.compensated_window_function = self
            .window_function
            .iter()
            .map(|x| x * compensation_factor)
            .collect();

        // stft
        self.stft.set_block_size(block_size);

        // complex buffer
        self.complex_buffers
            .iter_mut()
            .for_each(|buf| buf.resize(block_size / 2 + 1, Complex::default()));

        self.fft = RealFftPlanner::new().plan_fft_forward(block_size);
        self.ifft = RealFftPlanner::new().plan_fft_inverse(block_size);

        // mask
        unsafe {
            self.mask.set_len(block_size / 2);
        }
        // self.mask.resize(block_size / 2, 1.0);
    }

    /// Clones `mask` into the filter.
    ///
    /// Clones `min(self.block_size(), mask.len())` elements.
    ///
    /// (See [`set_block_size()`](Self::set_block_size))
    /// (See [`max_block_size()`](Self::max_block_size))
    pub fn set_mask(&mut self, mask: &SpectralMask) {
        for (dst, &src) in self.mask.iter_mut().zip(mask.iter()) {
            *dst = src;
        }
    }

    /// Sets the dry/wet mix of the filter. `0.0` is 100% dry, and `1.0` is
    /// 100% wet. The value is clamped to `[0.0, 1.0]`.
    pub fn set_mix(&mut self, mix: f64) {
        self.mix.set_target_value(mix.clamp(0.0, 1.0));
    }

    /// Processes a block of audio. This does not necessarily call the FFT algorithms.
    #[allow(clippy::missing_panics_doc)] // this function will not panic.
    pub fn process_block<B>(&mut self, buffer: &mut B)
    where
        B: StftInputMut,
    {
        self.store_dry(buffer);

        self.stft.process_overlap_add(
            buffer,
            Self::OVERLAP_FACTOR,
            |ch_idx, audio_block| {
                // window the input
                multiply_buffers(audio_block, &self.window_function);

                // to freq domain
                self.fft
                    .process(audio_block, &mut self.complex_buffers[ch_idx])
                    .unwrap();

                // process magnitudes
                self.complex_buffers[ch_idx]
                    .iter_mut()
                    .zip(self.mask.iter())
                    .for_each(|(bin, &mask)| {
                        *bin *= mask;
                    });

                self.complex_buffers[ch_idx][0] *= 0.0;

                // back to time domain
                self.ifft
                    .process(&mut self.complex_buffers[ch_idx], audio_block)
                    .unwrap();

                // window the output
                multiply_buffers(
                    audio_block, &self.compensated_window_function,
                );
            },
        );

        self.apply_mix(buffer);
    }

    /// The maximum block size of the filter.
    pub fn max_block_size(&self) -> usize {
        self.stft.max_block_size()
    }

    /// The current block size of the filter.
    pub fn block_size(&self) -> usize {
        self.mask.len()
    }

    /// Clears the filter's internal buffers.
    pub fn clear(&mut self) {
        self.complex_buffers
            .iter_mut()
            .for_each(|b| b.fill(Complex::new(0.0, 0.0)));
        self.stft.clear();
        self.mask.fill(0.0);
    }

    /// The compensation factor for a hanning window, resulting in unity gain for
    /// overlap factors of 4 and above.
    pub fn compensation_factor(&self, block_size: usize) -> f64 {
        ((Self::OVERLAP_FACTOR as f64 / 4.0) * 1.5).recip() / block_size as f64
    }

    /// Stores the input data into a temporary scratch buffer, used for
    /// dry/wet mixing.
    fn store_dry<B: StftInput>(&mut self, buffer: &B) {
        let num_ch = buffer.num_channels();
        let num_sm = buffer.num_samples();

        for ch in 0..num_ch {
            for smp in 0..num_sm {
                // safety: this will not violate the bounds of the
                // num_channels() and num_samples() methods, thus
                // should not expect to go out of bounds
                unsafe {
                    self.dry_buffer[ch * num_sm + smp] =
                        buffer.get_sample_unchecked(ch, smp);
                }
            }
        }
    }

    fn apply_mix<B: StftInputMut>(&mut self, buffer: &mut B) {
        let num_ch = buffer.num_channels();
        let num_sm = buffer.num_samples();

        for smp in 0..num_sm {
            let (dry, wet) = self.get_dry_wet();

            for ch in 0..num_ch {
                let dry_sample = self.dry_buffer[ch * num_sm + smp];

                unsafe {
                    let wet_sample = buffer.get_sample_unchecked(ch, smp);

                    *buffer.get_sample_unchecked_mut(ch, smp) =
                        dry.mul_add(dry_sample, wet * wet_sample);
                }
            }
        }
    }

    /// Returns the `(dry, wet)` values of the filter.
    fn get_dry_wet(&mut self) -> (f64, f64) {
        let mix = self.mix.next();

        let dry = ((FRAC_PI_2 * mix).cos());
        // * self.compensation_factor(self.mask.len()).recip();
        let wet = ((FRAC_PI_2 * mix).sin());

        (dry, wet)
    }
}

impl Default for SpectralFilter {
    fn default() -> Self {
        const DEFAULT_BLOCK_SIZE: usize = 1 << 14;

        Self {
            stft: StftHelper::new(NUM_CHANNELS, DEFAULT_BLOCK_SIZE, 0),
            fft: RealFftPlanner::new().plan_fft_forward(DEFAULT_BLOCK_SIZE),
            ifft: RealFftPlanner::new().plan_fft_inverse(DEFAULT_BLOCK_SIZE),

            mask: SpectralMask::new(DEFAULT_BLOCK_SIZE)
                .with_size(DEFAULT_BLOCK_SIZE / 2),

            compensated_window_function: Vec::default(),
            window_function: Vec::default(),

            dry_buffer: Vec::default(),

            mix: Smoother::new(30.0, 1.0, unsafe { SAMPLE_RATE }),

            complex_buffers: Vec::default(),
        }
    }
}
