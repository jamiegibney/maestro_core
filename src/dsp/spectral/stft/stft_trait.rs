//! Module for STFT traits used by [`StftHelper`].

use crate::prelude::*;

use nannou_audio::Buffer;

/// A buffer which may be read by the `StftHelper`.
pub trait StftInput {
    /// Number of samples in the input buffer.
    fn num_samples(&self) -> usize;

    /// Number of channels in the input buffer.
    fn num_channels(&self) -> usize;

    /// Obtains a copy of a specific sample without any bounds checking.
    unsafe fn get_sample_unchecked(&self, channel_idx: usize, sample_idx: usize) -> f64;
}

/// A buffer which may be written to by the `StftHelper`.
pub trait StftInputMut: StftInput {
    /// Obtains a mutable reference to a specific sample without any
    /// bounds checking.
    unsafe fn get_sample_unchecked_mut(
        &mut self,
        channel_idx: usize,
        sample_idx: usize,
    ) -> &mut f64;
}

impl StftInput for Buffer<f64> {
    #[inline]
    fn num_samples(&self) -> usize {
        self.len_frames()
    }

    #[inline]
    fn num_channels(&self) -> usize {
        self.channels()
    }

    #[inline]
    unsafe fn get_sample_unchecked(&self, channel_idx: usize, sample_idx: usize) -> f64 {
        // the samples of this buffer are interleaved, hence channel * 2
        unsafe { *self.get_unchecked(sample_idx * 2 + channel_idx) }
    }
}

impl StftInputMut for Buffer<f64> {
    #[inline]
    unsafe fn get_sample_unchecked_mut(
        &mut self,
        channel_idx: usize,
        sample_idx: usize,
    ) -> &mut f64 {
        // the samples of this buffer are interleaved, hence channel * 2
        unsafe { self.get_unchecked_mut(sample_idx * 2 + channel_idx) }
    }
}

impl StftInput for [&[f64]] {
    #[inline]
    fn num_samples(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self[0].len()
        }
    }

    #[inline]
    fn num_channels(&self) -> usize {
        self.len()
    }

    #[inline]
    unsafe fn get_sample_unchecked(&self, channel_idx: usize, sample_idx: usize) -> f64 {
        unsafe { *self.get_unchecked(channel_idx).get_unchecked(sample_idx) }
    }
}

impl StftInput for [&mut [f64]] {
    #[inline]
    fn num_samples(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self[0].len()
        }
    }

    #[inline]
    fn num_channels(&self) -> usize {
        self.len()
    }

    #[inline]
    unsafe fn get_sample_unchecked(&self, channel_idx: usize, sample_idx: usize) -> f64 {
        unsafe { *self.get_unchecked(channel_idx).get_unchecked(sample_idx) }
    }
}

impl StftInputMut for [&mut [f64]] {
    #[inline]
    unsafe fn get_sample_unchecked_mut(
        &mut self,
        channel_idx: usize,
        sample_idx: usize,
    ) -> &mut f64 {
        unsafe {
            self.get_unchecked_mut(channel_idx)
                .get_unchecked_mut(sample_idx)
        }
    }
}

impl StftInput for &[f64] {
    #[inline]
    fn num_samples(&self) -> usize {
        self.len() / NUM_CHANNELS
    }

    #[inline]
    fn num_channels(&self) -> usize {
        NUM_CHANNELS
    }

    #[inline]
    unsafe fn get_sample_unchecked(&self, channel_idx: usize, sample_idx: usize) -> f64 {
        // the samples of this buffer are interleaved, hence channel * 2
        unsafe { *self.get_unchecked(sample_idx * 2 + channel_idx) }
    }
}
