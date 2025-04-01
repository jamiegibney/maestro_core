//! JUCE-based oversampling filters.

use super::block::{OversamplingBlock, OversamplingBuffer};
use super::*;
use nannou_audio::Buffer;
use std::cell::RefCell;
use std::rc::Rc;

const EQUIRIPPLE_FACTOR: usize = 2;

pub(super) struct OversamplingEquiripple {
    // buffer: OversamplingBuffer,
    buffer: Vec<f64>,
    // buffer_block: OversamplingBlock,
    num_channels: usize,

    coefs_up: Rc<RefCell<FIRCoefficients>>,
    coefs_down: Rc<RefCell<FIRCoefficients>>,

    state_up: OversamplingBuffer,
    state_down: OversamplingBuffer,
    state_down_2: OversamplingBuffer,

    position: Vec<usize>,
}

// TODO: a lot of these method should be extracted to a trait so that extra
// filtering methods can be added and used in `Oversampler`
impl OversamplingEquiripple {
    pub fn create(
        num_channels: usize,
        normalised_transition_width_up: f64,
        stopband_amplitude_db_up: f64,
        normalised_transition_width_down: f64,
        stopband_amplitude_db_down: f64,
    ) -> Self {
        let coefs_up = FilterDesign::fir_half_band_equiripple_method(
            normalised_transition_width_up, stopband_amplitude_db_up,
        );
        let coefs_down = FilterDesign::fir_half_band_equiripple_method(
            normalised_transition_width_down, stopband_amplitude_db_down,
        );

        let n_up = coefs_up.borrow().filter_order() + 1;
        let n_down = coefs_down.borrow().filter_order() + 1;

        Self {
            // buffer: OversamplingBuffer::new(num_channels, 0),
            buffer: Vec::new(),
            num_channels,
            coefs_up,
            coefs_down,
            state_up: OversamplingBuffer::new(num_channels, n_up),
            state_down: OversamplingBuffer::new(num_channels, n_down),
            state_down_2: OversamplingBuffer::new(num_channels, n_down / 4),
            position: vec![0; num_channels],
        }
    }

    pub fn initialize_buffer(
        &mut self,
        max_samples_before_oversampling: usize,
    ) {
        let num_samples = max_samples_before_oversampling
            * EQUIRIPPLE_FACTOR
            * self.num_channels;
        self.buffer.resize(self.num_channels * num_samples, 0.0);
    }

    pub fn latency_samples(&self) -> f64 {
        (self.coefs_up.borrow().filter_order()
            + self.coefs_down.borrow().filter_order()) as f64
            * 0.5
    }

    pub fn reset(&mut self) {
        self.state_up.clear();
        self.state_down.clear();
        self.state_down_2.clear();

        self.position.fill(0);
    }

    pub fn factor(&self) -> usize {
        EQUIRIPPLE_FACTOR
    }

    // TODO the handling of channels/samples is very messy here - interleaved,
    // storing channels, 2-dim vectors? an "audio block" or "buffer" object
    // would make a lot more sense for handling all of that.
    pub fn process_samples_up(&mut self, input_block: &[f64]) {
        let num_channels = self.num_channels;
        let num_samples = input_block.len() / self.num_channels;

        let fir = self.coefs_up.borrow().get_coefs();
        let n = self.coefs_up.borrow().filter_order() + 1;
        let n_2 = n / 2;

        // processing
        for ch in 0..num_channels {
            let s_up = self.state_up.channel_mut(ch);

            for i in 0..num_samples {
                // input
                s_up[n - 1] = 2.0 * input_block[i * num_channels + ch];

                // convolution
                let out: f64 = (0..n_2)
                    .step_by(2)
                    .map(|k| {
                        (s_up[k] + self.buffer_sample(ch, n - k - 1)) * fir[k]
                    })
                    .sum();

                // outputs
                *self.buffer_sample_mut(ch, i << 1) = out;
                *self.buffer_sample_mut(ch, (i << 1) + 1) =
                    s_up[n_2 + 1] * fir[n_2];

                // shift
                for k in (0..(n - 2)).step_by(2) {
                    s_up[k] = s_up[k + 2];
                }
            }
        }
    }

    pub fn process_samples_down(
        &mut self,
        // output_block: &mut OversamplingBlock,
        output_block: &mut [f64],
    ) {
        let num_channels = self.num_channels();
        let num_samples = output_block.len() / self.num_channels;

        let fir = self.coefs_down.borrow().get_coefs();
        let n = self.coefs_down.borrow().filter_order() + 1;
        let n_2 = n / 2;
        let n_4 = n / 4;

        for ch in 0..num_channels {
            let s_down = self.state_down.channel_mut(ch);
            let s_down_2 = self.state_down_2.channel_mut(ch);
            let pos = &mut self.position[ch];

            for i in 0..num_samples {
                // input
                s_down[n - 1] = self.buffer_sample(ch, i << 1);

                // convolution
                let mut out: f64 = (0..n_2)
                    .step_by(2)
                    .map(|k| (s_down[k] + s_down[n - k - 1]) * fir[k])
                    .sum();

                // output
                out += s_down_2[*pos] * fir[n_2];
                s_down_2[*pos] = self.buffer_sample(ch, (i << 1) + 1);

                output_block[Self::interleaved_idx(self.num_channels, ch, i)] =
                    out;

                // shift
                for k in 0..(n - 2) {
                    s_down[k] = s_down[k + 2];
                }

                // wrap buffer
                *pos = if *pos == 0 { n_4 } else { *pos - 1 };
            }
        }
    }

    /// Returns an *immutable* slice to the **interleaved** samples in the internal buffer.
    pub fn get_processed_samples(
        &self,
        // channel: usize,
        num_samples: usize,
    ) -> &[f64] {
        &self.buffer[..num_samples]
    }

    /// Returns a *mutable* slice to the **interleaved** samples in the internal buffer.
    pub fn get_processed_samples_mut(
        &mut self,
        num_samples: usize,
    ) -> &mut [f64] {
        &mut self.buffer[..num_samples]
    }

    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    fn interleaved_idx(
        num_channels: usize,
        channel_idx: usize,
        sample_idx: usize,
    ) -> usize {
        sample_idx * num_channels + channel_idx
    }

    fn buffer_sample(&self, channel_idx: usize, sample_idx: usize) -> f64 {
        self.buffer[sample_idx * self.num_channels + channel_idx]
    }

    fn buffer_sample_mut(
        &mut self,
        channel_idx: usize,
        sample_idx: usize,
    ) -> &mut f64 {
        &mut self.buffer[sample_idx * self.num_channels + channel_idx]
    }
}
