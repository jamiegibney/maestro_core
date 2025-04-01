//! Module for a short-time Fourier transform "helper".
//!
//! Implementation based on the one found in [`nih-plug`](https://github.com/robbert-vdh/nih-plug).

use super::stft_trait::*;
use crate::prelude::*;

/// A convenience struct for a short-time Fourier transform algorithm.
pub struct StftHelper {
    /// These buffers store samples from the input *and* the output produced by summing
    /// overlapping windows. Whenever a new overlapping window is reached, the computed
    /// output is written to the main buffer (passed to the STFT externally), and then
    /// a new block is processed.
    main_input_ring_buffers: Vec<Vec<f64>>,
    main_output_ring_buffers: Vec<Vec<f64>>,

    /// Results from the buffers are copied to this buffer before being written to the
    /// main audio buffer - this is required to manage window overlap.
    scratch_buffer: Vec<f64>,

    padding_buffers: Vec<Vec<f64>>,

    /// The current position in the ring buffers. When this wraps back to 0, a new block
    /// is processed.
    current_pos: usize,
    /// If padding is used, this holds the amount of extra capacity to add to the buffers.
    padding: usize,
}

impl StftHelper {
    /// Returns a new `StftHelper`.
    ///
    /// # Panics
    ///
    /// Panics if `num_channels` or `max_block_size` is `0`.
    #[must_use]
    pub fn new(
        num_channels: usize,
        max_block_size: usize,
        max_padding: usize,
    ) -> Self {
        assert_ne!(num_channels, 0);
        assert_ne!(max_block_size, 0);

        Self {
            main_input_ring_buffers: vec![
                vec![0.0; max_block_size];
                num_channels
            ],
            main_output_ring_buffers: vec![
                vec![0.0; max_block_size];
                num_channels
            ],
            scratch_buffer: vec![0.0; max_block_size + max_padding],
            padding_buffers: vec![vec![0.0; max_padding]; num_channels],
            current_pos: 0,
            padding: max_padding,
        }
    }

    /// Sets the block size. This clears the internal buffers, meaning the next block will
    /// be silent.
    ///
    /// # Panics
    ///
    /// Panics if `block_size > max_block_size`.
    pub fn set_block_size(&mut self, block_size: usize) {
        assert!(block_size <= self.main_input_ring_buffers[0].capacity());

        self.initialize_buffers(block_size);
    }

    /// Clears the internal buffers.
    pub fn clear(&mut self) {
        self.main_input_ring_buffers
            .iter_mut()
            .for_each(|v| v.fill(0.0));
        self.main_output_ring_buffers
            .iter_mut()
            .for_each(|v| v.fill(0.0));
        self.scratch_buffer.fill(0.0);
        self.padding_buffers.iter_mut().for_each(|v| v.fill(0.0));
        self.current_pos = 0;
    }

    /// Set the amount of padding. This clears the internal buffers, meaning the next block
    /// will be silent.
    ///
    /// # Panics
    ///
    /// Panics if `padding > max_padding`.
    pub fn set_padding(&mut self, padding: usize) {
        assert!(padding <= self.padding_buffers[0].capacity());

        self.padding = padding;
        self.initialize_buffers(self.main_input_ring_buffers[0].len());
    }

    /// Number of channels the `StftHelper` was set up with.
    pub fn num_channels(&self) -> usize {
        self.main_input_ring_buffers.len()
    }

    /// Maximum block size the `StftHelper` was set up with.
    pub fn max_block_size(&self) -> usize {
        self.main_input_ring_buffers[0].capacity()
    }

    /// Maximum amount of padding the `StftHelper` was set up with.
    pub fn max_padding(&self) -> usize {
        self.padding_buffers[0].len()
    }

    /// Amount of latency produced by the STFT process in samples.
    pub fn latency_samples(&self) -> u32 {
        self.main_input_ring_buffers[0].capacity() as u32
    }

    /// Processes the audio from `main_buffer` in short, overlapping blocks, then sums the
    /// blocks back to the main buffer one block later. This means that this function
    /// introduces **one block of latency**. Use [`latency_samples()`][Self::latency_samples()]
    /// to obtain the amount of latency in samples.
    ///
    /// If a padding value was provided to the `StftHelper`, that many zeroes will be appended
    /// to each block. Padding values are added to the next block before the `callback` is called.
    ///
    /// Any window functions must be applied within the callback, as there are many to choose from.
    ///
    /// The `callback` function expects its first argument to be the channel index (as `usize`),
    /// and its second to be the real-valued buffer (as `&mut [f64]`). The real-valued buffer
    /// is a slice of `block_size` read-valued samples, which can be passed directly to most FFT
    /// algorithms.
    ///
    /// This function reuses the same buffer for all calls to `callback`, which does mean that you
    /// can only access one channel of windowed data at a time.
    ///
    /// # Panics
    ///
    /// Panics if the buffer has a different number of channels to the `StftHelper`,
    /// or if `overlap_factor == 0`.
    pub fn process_overlap_add<M, F>(
        &mut self,
        main_buffer: &mut M,
        overlap_factor: usize,
        mut callback: F,
    ) where
        M: StftInputMut,
        F: FnMut(usize, &mut [f64]),
    {
        assert_eq!(main_buffer.num_channels(), self.num_channels());
        assert!(overlap_factor > 0);

        let main_buffer_len = main_buffer.num_samples();
        let num_channels = main_buffer.num_channels();
        let block_size = self.main_input_ring_buffers[0].len();
        let window_interval = (block_size / overlap_factor) as i32;

        let mut num_processed_samples = 0;

        while num_processed_samples < main_buffer_len {
            let num_remaining_samples = main_buffer_len - num_processed_samples;
            let samples_until_next_window =
                ((window_interval - self.current_pos as i32 - 1)
                    .rem_euclid(window_interval)
                    + 1) as usize;
            let samples_to_process =
                samples_until_next_window.min(num_remaining_samples);

            for sample_offset in 0..samples_to_process {
                for ch in 0..num_channels {
                    let sample = unsafe {
                        main_buffer.get_sample_unchecked_mut(
                            ch,
                            num_processed_samples + sample_offset,
                        )
                    };
                    let input_ring_buffer_sample = unsafe {
                        self.main_input_ring_buffers
                            .get_unchecked_mut(ch)
                            .get_unchecked_mut(self.current_pos + sample_offset)
                    };
                    let output_ring_buffer_sample = unsafe {
                        self.main_output_ring_buffers
                            .get_unchecked_mut(ch)
                            .get_unchecked_mut(self.current_pos + sample_offset)
                    };

                    *input_ring_buffer_sample = *sample;
                    *sample = *output_ring_buffer_sample;
                    // very important to avoid feedback...
                    *output_ring_buffer_sample = 0.0;
                }
            }

            num_processed_samples += samples_to_process;
            self.current_pos =
                (self.current_pos + samples_to_process) % block_size;

            if samples_to_process == samples_until_next_window {
                for (ch, ((input_buffer, output_buffer), padding_buffer)) in
                    self.main_input_ring_buffers
                        .iter()
                        .zip(self.main_output_ring_buffers.iter_mut())
                        .zip(self.padding_buffers.iter_mut())
                        .enumerate()
                {
                    copy_ring_to_scratch(
                        &mut self.scratch_buffer, self.current_pos,
                        input_buffer,
                    );

                    if self.padding > 0 {
                        self.scratch_buffer[block_size..].fill(0.0);
                    }

                    callback(ch, &mut self.scratch_buffer);

                    if self.padding > 0 {
                        let padding_to_copy = self.padding.min(block_size);

                        for (scratch, padding) in self.scratch_buffer
                            [..padding_to_copy]
                            .iter_mut()
                            .zip(&mut padding_buffer[..padding_to_copy])
                        {
                            *scratch += *padding;
                        }

                        padding_buffer.copy_within(padding_to_copy.., 0);

                        padding_buffer[self.padding - padding_to_copy..]
                            .fill(0.0);
                    }

                    add_scratch_to_ring(
                        &self.scratch_buffer, self.current_pos, output_buffer,
                    );

                    if self.padding > 0 {
                        for (padding, scratch) in padding_buffer
                            .iter_mut()
                            .zip(&mut self.scratch_buffer[block_size..])
                        {
                            *padding += *scratch;
                        }
                    }
                }
            }
        }
    }

    /// Only processes the forward FFT, so the buffer is only ever *read*, not mutated.
    ///
    /// The `callback` function expects its first argument to be the channel
    /// index (as `usize`), and its second to be the real-valued buffer (as
    /// `&mut [f64]`).
    ///
    /// # Panics
    ///
    /// Panics if the buffer has a different number of channels to the `StftHelper`,
    /// or if `overlap_factor == 0`.
    pub fn process_forward_only<B, F>(
        &mut self,
        main_buffer: &B,
        overlap_factor: usize,
        mut callback: F,
    ) where
        B: StftInput,
        F: FnMut(usize, &mut [f64]),
    {
        assert_eq!(main_buffer.num_channels(), self.num_channels());
        assert!(overlap_factor > 0);

        let main_buffer_len = main_buffer.num_samples();
        let num_channels = main_buffer.num_channels();
        let block_size = self.main_input_ring_buffers[0].len();
        let window_interval = (block_size / overlap_factor) as i32;

        let mut num_processed_samples = 0;

        while num_processed_samples < main_buffer_len {
            let num_remaining_samples = main_buffer_len - num_processed_samples;
            let samples_until_next_window =
                ((window_interval - self.current_pos as i32 - 1)
                    .rem_euclid(window_interval)
                    + 1) as usize;
            let samples_to_process =
                samples_until_next_window.min(num_remaining_samples);

            for sample_offset in 0..samples_to_process {
                for ch in 0..num_channels {
                    let sample = unsafe {
                        main_buffer.get_sample_unchecked(
                            ch,
                            num_processed_samples + sample_offset,
                        )
                    };
                    let input_ring_buffer_sample = unsafe {
                        self.main_input_ring_buffers
                            .get_unchecked_mut(ch)
                            .get_unchecked_mut(self.current_pos + sample_offset)
                    };
                    *input_ring_buffer_sample = sample;
                }
            }

            num_processed_samples += samples_to_process;

            self.current_pos =
                (self.current_pos + samples_to_process) % block_size;

            if samples_to_process == samples_until_next_window {
                for (ch, input_buffer) in
                    self.main_input_ring_buffers.iter().enumerate()
                {
                    copy_ring_to_scratch(
                        &mut self.scratch_buffer, self.current_pos,
                        input_buffer,
                    );

                    if self.padding > 0 {
                        self.scratch_buffer[block_size..].fill(0.0);
                    }

                    callback(ch, &mut self.scratch_buffer);
                }
            }
        }
    }

    /// Resizes the internal buffers to `block_size` and clears each element to `0.0`.
    fn initialize_buffers(&mut self, block_size: usize) {
        for main_ring_buf in &mut self.main_input_ring_buffers {
            main_ring_buf.resize(block_size, 0.0);
            main_ring_buf.fill(0.0);
        }
        for main_ring_buf in &mut self.main_output_ring_buffers {
            main_ring_buf.resize(block_size, 0.0);
            main_ring_buf.fill(0.0);
        }

        self.scratch_buffer.resize(block_size + self.padding, 0.0);
        self.scratch_buffer.fill(0.0);

        for padding_buf in &mut self.padding_buffers {
            padding_buf.resize(block_size, 0.0);
            padding_buf.fill(0.0);
        }

        self.current_pos = 0;
    }
}

/// Copies content from a ring buffer to a scratch buffer, starting at the current position.
#[inline]
fn copy_ring_to_scratch(
    scratch_buffer: &mut [f64],
    current_pos: usize,
    ring_buffer: &[f64],
) {
    let block_size = ring_buffer.len();
    let num_before_wrap = block_size - current_pos;

    scratch_buffer[0..num_before_wrap]
        .copy_from_slice(&ring_buffer[current_pos..block_size]);
    scratch_buffer[num_before_wrap..block_size]
        .copy_from_slice(&ring_buffer[0..current_pos]);
}

/// Adds elements from a scratch buffer to a ring buffer.
#[inline]
fn add_scratch_to_ring(
    scratch_buffer: &[f64],
    current_pos: usize,
    ring_buffer: &mut [f64],
) {
    let block_size = ring_buffer.len();
    let num_before_wrap = block_size - current_pos;

    for (scratch, ring) in scratch_buffer[0..num_before_wrap]
        .iter()
        .zip(&mut ring_buffer[current_pos..block_size])
    {
        *ring += *scratch;
    }
    for (scratch, ring) in scratch_buffer[num_before_wrap..block_size]
        .iter()
        .zip(&mut ring_buffer[0..current_pos])
    {
        *ring += *scratch;
    }
}
