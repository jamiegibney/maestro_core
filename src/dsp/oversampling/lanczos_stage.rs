//! Lanczos resampling stage.

// TODO there is massive potential to parallelise these operations, namely the
// loop which calls the convolution algorithm. Using an FFT to convolve is likely
// not worthwhile for such a small data set (the kernel is only convolved with
// very small buffers).

use std::sync::Arc;

#[derive(Clone)]
pub(super) struct Lanczos3Stage {
    oversampling_amount: usize,

    // Arc is used as the same data is shared among stages, so may as well
    // have only one copy to hopefully improve cache locality.
    // Same applies to downsampling_kernel
    upsampling_kernel: Arc<[f64]>,
    upsampling_buffer: Vec<f64>,
    upsampling_write_pos: usize,

    additional_upsampling_latency: usize,

    downsampling_kernel: Arc<[f64]>,
    downsampling_buffer: Vec<f64>,
    downsampling_write_pos: usize,

    pub(super) scratch_buffer: Vec<f64>,
}

impl Lanczos3Stage {
    /// Creates a new oversampling stage.
    ///
    /// `quality_factor` directly affects the "a" variable in the Lanczos kernel.
    /// A value of `3` offers a good compromise of performance and quality.
    ///
    /// Note that increases in quality will exponentially decrease performance, but
    /// offer diminishing returns in resampling quality.
    ///
    /// # Panics
    ///
    /// Panics if `quality_factor == 0`.
    pub fn new(
        max_block_size: usize,
        stage_number: u32,
        quality_factor: u8,
        upsampling_kernel: Arc<[f64]>,
        downsampling_kernel: Arc<[f64]>,
    ) -> Self {
        assert_ne!(quality_factor, 0);
        let oversampling_amount = 2usize.pow(stage_number + 1);

        let uncompensated_stage_latency = upsampling_kernel.len() * 2;
        let additional_upsampling_latency = (-(uncompensated_stage_latency as isize))
            .rem_euclid(oversampling_amount as isize)
            as usize;

        Self {
            oversampling_amount,

            upsampling_buffer: vec![0.0; upsampling_kernel.len() + additional_upsampling_latency],
            upsampling_kernel,
            upsampling_write_pos: 0,

            additional_upsampling_latency,

            downsampling_buffer: vec![0.0; downsampling_kernel.len()],
            downsampling_kernel,
            downsampling_write_pos: 0,

            scratch_buffer: vec![0.0; max_block_size * oversampling_amount],
        }
    }

    pub fn reset(&mut self) {
        self.upsampling_buffer.fill(0.0);
        self.upsampling_write_pos = 0;

        self.downsampling_buffer.fill(0.0);
        self.downsampling_write_pos = 0;
    }

    pub fn effective_latency(&self) -> u32 {
        let uncompensated_stage_latency = self.stage_latency() * 2;
        let total_stage_latency = uncompensated_stage_latency + self.additional_upsampling_latency;

        (total_stage_latency as f64 / self.oversampling_amount as f64) as u32
    }

    /// Upsamples a single block of audio. That is, **one** channel.
    pub fn upsample_from(&mut self, block: &[f64]) {
        let output_length = block.len() * 2;
        assert!(output_length <= self.scratch_buffer.len());

        for (i, &smp) in block.iter().enumerate() {
            self.scratch_buffer[i * 2] = smp;
            self.scratch_buffer[i * 2 + 1] = 0.0;
        }

        let mut direct_read_pos =
            (self.upsampling_write_pos + self.stage_latency()) % self.upsampling_buffer.len();

        for out_idx in 0..output_length {
            self.upsampling_buffer[self.upsampling_write_pos] = self.scratch_buffer[out_idx];

            self.increment_up_write_positions(&mut direct_read_pos);

            self.scratch_buffer[out_idx] = if out_idx % 2 == (self.stage_latency() % 2) {
                debug_assert!(
                    self.upsampling_buffer[(direct_read_pos + self.upsampling_buffer.len() - 1)
                        % self.upsampling_buffer.len()]
                        <= f64::EPSILON
                );
                debug_assert!(
                    self.upsampling_buffer[(direct_read_pos + 1) % self.upsampling_buffer.len()]
                        <= f64::EPSILON
                );

                self.upsampling_buffer[direct_read_pos]
            } else {
                convolve(
                    &self.upsampling_buffer,
                    &self.upsampling_kernel,
                    self.upsampling_write_pos,
                )
            }
        }
    }

    pub fn downsample_to(&mut self, block: &mut [f64]) {
        let input_length = block.len() * 2;
        assert!(input_length <= self.scratch_buffer.len());

        for input_idx in 0..input_length {
            self.downsampling_buffer[self.downsampling_write_pos] = self.scratch_buffer[input_idx];

            self.increment_down_write_pos();

            if input_idx % 2 == 0 {
                let output_idx = input_idx / 2;

                block[output_idx] = convolve(
                    &self.downsampling_buffer,
                    &self.downsampling_kernel,
                    self.downsampling_write_pos,
                );
            }
        }
    }

    fn stage_latency(&self) -> usize {
        self.upsampling_kernel.len() / 2
    }

    fn increment_up_write_positions(&mut self, direct_pos: &mut usize) {
        self.upsampling_write_pos += 1;
        if self.upsampling_write_pos == self.upsampling_buffer.len() {
            self.upsampling_write_pos = 0;
        }

        *direct_pos += 1;
        if *direct_pos == self.upsampling_buffer.len() {
            *direct_pos = 0;
        }
    }

    fn increment_down_write_pos(&mut self) {
        self.downsampling_write_pos += 1;
        if self.downsampling_write_pos == self.downsampling_buffer.len() {
            self.downsampling_write_pos = 0;
        }
    }
}

/// This function is optimised to skip the interleaved zeroes in the Lanczos kernel, 
/// and expects the first and last elements to be non-zero. This makes the operation
/// significantly faster.
#[rustfmt::skip]
fn convolve(input_buffer: &[f64], kernel: &[f64], buffer_pos: usize) -> f64 {
    let len = input_buffer.len();
    debug_assert!(len >= kernel.len());

    // technically this is cross-correlation, not convolution, because the kernel 
    // is processed forwards, but because the Lanczos kernel is symmetrical the
    // reversal is a redundant operation.
    kernel
        .iter().step_by(2).enumerate()
        .map(|(off, &smp)| smp * input_buffer[(buffer_pos + off) % len])
        .sum()
}

/* /// Returns a vector containing points of a Lanczos kernel. `a_factor` is the "a"
/// variable in the kernel calculation. Only holds enough points to represent each lobe.
/// Returns `4 * a_factor + 1` elements (when `trim_zeroes == false`).
///
/// `scale` will automatically scale each element in the kernel, and `trim_zeroes` will
/// remove the first and last elements (which are always `0.0`) if true.
///
/// [Source](https://en.wikipedia.org/wiki/Lanczos_resampling)
///
/// # Panics
///
/// Panics if `a_factor == 0`.
fn lanczos_kernel(a_factor: u8, scale: f64, trim_zeroes: bool) -> Vec<f64> {
    assert_ne!(a_factor, 0);

    let a = a_factor as f64;
    let num_stages = a_factor * 4 + 1;

    (if trim_zeroes { 1..num_stages - 1 } else { 0..num_stages })
        .map(|i| {
            if i % 2 == 0 {
                0.0
            }
            else {
                let x = 2.0f64.mul_add(-a, i as f64) / 2.0;

                if x == 0.0 {
                    1.0
                }
                else if -a <= x && x < a {
                    sinc(PI * x) * sinc((PI * x) / a) * scale
                }
                else {
                    0.0
                }
            }
        })
        .collect()
} */
