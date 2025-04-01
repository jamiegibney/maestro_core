//! Oversampling driven by Lanczos resampling.

use crate::prelude::lanczos_kernel;
use std::sync::Arc;

use super::lanczos_stage::Lanczos3Stage;

/// An oversampler which uses Lanczos resampling.
//
// TODO: add the ability to change the number of stages.
#[derive(Clone)]
pub struct Lanczos3Oversampler {
    stages: Vec<Lanczos3Stage>,
    latencies: Vec<u32>,
}

impl Lanczos3Oversampler {
    /// # Panics
    ///
    /// Panics if `quality_factor == 0`, or if `max_factor == 0`.
    pub fn new(max_block_size: usize, max_factor: usize, quality_factor: u8) -> Self {
        assert_ne!(max_factor, 0);
        let mut stages = Vec::with_capacity(max_factor);
        let upsampling_kernel = Arc::from(lanczos_kernel(quality_factor, 1.0, true));
        // the downsampling kernel is identical, but scaled by half to result in
        // unity gain after oversampling.
        let downsampling_kernel = Arc::from(lanczos_kernel(quality_factor, 0.5, true));

        for stage in 0..max_factor {
            stages.push(Lanczos3Stage::new(
                max_block_size,
                stage as u32,
                quality_factor,
                Arc::clone(&upsampling_kernel),
                Arc::clone(&downsampling_kernel),
            ));
        }

        Self {
            latencies: stages
                .iter()
                .map(|stage| stage.effective_latency())
                .scan(0, |total_latency, latency| {
                    *total_latency += latency;
                    Some(*total_latency)
                })
                .collect(),

            stages,
        }
    }

    pub fn reset(&mut self) {
        self.stages.iter_mut().for_each(|stage| stage.reset());
    }

    #[rustfmt::skip]
    pub fn latency(&self, factor: usize) -> u32 {
        if factor == 0 { 0 } else { self.latencies[factor - 1] }
    }

    pub fn max_stages(&self) -> usize {
        self.stages.len()
    }

    pub fn process(&mut self, block: &mut [f64], factor: usize, f: impl FnOnce(&mut [f64])) {
        if factor == 0 {
            f(block);
            return;
        }
        debug_assert!(factor <= self.stages.len());
        debug_assert!(
            block.len() <= self.stages[0].scratch_buffer.len() / 2,
            "The block exceeded the max size"
        );

        let upsampled = self.upsample_from(block, factor);
        f(upsampled);
        self.downsample_to(block, factor);
    }

    pub fn upsample_from(&mut self, block: &[f64], factor: usize) -> &mut [f64] {
        debug_assert_ne!(factor, 0);
        debug_assert!(factor <= self.stages.len());

        self.stages[0].upsample_from(block);

        let mut previous_block_len = block.len() * 2;

        for to_stage_idx in 1..factor {
            let ([.., from], [to, ..]) = self.stages.split_at_mut(to_stage_idx) else {
                unreachable!()
            };

            to.upsample_from(&from.scratch_buffer[..previous_block_len]);
            previous_block_len *= 2;
        }

        &mut self.stages[factor - 1].scratch_buffer[..previous_block_len]
    }

    pub fn upsample_only<'a>(&'a mut self, block: &'a mut [f64], factor: usize) -> &'a mut [f64] {
        debug_assert!(factor <= self.stages.len());

        if factor == 0 {
            return block;
        }

        assert!(
            block.len() <= self.stages[0].scratch_buffer.len() / 2,
            "The block exceeded the max size"
        );

        self.upsample_from(block, factor)
    }

    pub fn downsample_to(&mut self, block: &mut [f64], factor: usize) {
        debug_assert_ne!(factor, 0);
        debug_assert!(factor <= self.stages.len());

        let mut next_block_len = block.len() * 2usize.pow(factor as u32 - 1);

        for to_stage_idx in (1..factor).rev() {
            let ([.., to], [from, ..]) = self.stages.split_at_mut(to_stage_idx) else {
                unreachable!()
            };

            from.downsample_to(&mut to.scratch_buffer[..next_block_len]);

            next_block_len /= 2;
        }

        assert_eq!(next_block_len, block.len());
        self.stages[0].downsample_to(block);
    }
}
