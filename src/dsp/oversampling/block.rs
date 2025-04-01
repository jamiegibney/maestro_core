//! Oversampling types.

use std::ops::{Deref, DerefMut};

use super::*;
use nannou_audio::Buffer;

/// This is a kind of reference to a `nannou_audio` `Buffer` type which holds mutable
/// pointers to the samples within it. It essentially makes it such that you can directly
/// access the separate channels within the buffer.
///
/// Using pointers is not strictly necessary - mutable referenced would suffice just fine,
/// but that involves dealing with lifetimes and using the pointers in this fashion is easy
/// to keep safe as they are intended to essentially used like references anyway. Still,
/// caution should be used regardless.
pub struct OversamplingBlock {
    // referenced_data: &'a [f64],
    // channel_references: Vec<Vec<&'a mut f64>>,
    channel_pointers: Vec<Vec<*mut f64>>,
    // num_channels: usize,
    // num_samples: usize,
}

impl OversamplingBlock {
    /// Creates a new `OversamplingBlock` from a `Buffer`.
    pub fn from_buffer(buffer: &mut Buffer<f64>) -> Self {
        let num_channels = buffer.channels();
        let num_samples = buffer.len_frames();

        Self {
            channel_pointers: (0..num_channels)
                .map(|ch| {
                    (0..num_samples)
                        .map(|smp| &mut buffer[smp * num_channels + ch] as *mut f64)
                        .collect()
                })
                .collect(),
        }
    }

    pub fn from_oversampling_buffer(buffer: &mut OversamplingBuffer) -> Self {
        let num_channels = buffer.num_channels();
        let _num_samples = buffer.num_samples();

        Self {
            channel_pointers: (0..num_channels)
                .map(|ch| vec![buffer.channel_mut(ch).as_mut_ptr()])
                .collect(),
        }
    }

    /// Creates a new `OversamplingBlock` from a slice of `f64`.
    pub fn from_interleaved_slice(slice: &mut [f64], num_channels: usize) -> Self {
        let num_samples = slice.len() / num_channels;

        Self {
            channel_pointers: (0..num_channels)
                .map(|ch| {
                    (0..num_samples)
                        .map(|smp| &mut slice[smp * num_channels + ch] as *mut f64)
                        .collect()
                })
                .collect(),
        }
    }

    /// Returns a slice of mutable `f64` pointers contained within the original buffer.
    ///
    /// There is likely a neater way of doing this, but I need easy, mutable access to
    /// values which are in an interleaved order, and this works.
    ///
    /// # Safety
    ///
    /// This function is completely safe to use, but dereferencing any of the pointers
    /// contained in the slice is, of course, an unsafe action.
    pub fn channel_data(&mut self, channel_idx: usize) -> &[*mut f64] {
        &self.channel_pointers[channel_idx]
    }

    /// Returns the number of channels referenced by the `OversamplingBlock`.
    pub fn num_channels(&self) -> usize {
        self.channel_pointers.len()
    }

    /// Returns the number of samples referenced by each channel of the `OversamplingBlock`.
    ///
    /// Returns `0` if no data is referenced.
    pub fn num_samples(&self) -> usize {
        if let Some(s) = self.channel_pointers.first() {
            s.len()
        } else {
            0
        }
    }
}

/// A struct for holding owned audio data, used for Oversampling.
#[derive(Default)]
pub struct OversamplingBuffer {
    data: Vec<Vec<f64>>,
}

impl OversamplingBuffer {
    /// Creates a new buffer holding `num_channels` channels of `num_samples` samples.
    pub fn new(num_channels: usize, num_samples: usize) -> Self {
        Self {
            data: vec![vec![0.0; num_samples]; num_channels],
        }
    }

    /// Copies the contents of `buffer` into the `OversamplingBuffer`. This essentially
    /// just copies the interleaved layout of `buffer` into the 2-dimensional layout of
    /// `self`, making it compatible with the oversamplers.
    ///
    /// This method does not allocate.
    ///
    /// # Panics
    ///
    /// Panics if the channel or sample count of `buffer` does not match the
    /// `OversamplingBuffer`'s.
    pub fn copy_from_buffer(&mut self, buffer: &Buffer<f64>) {
        let num_channels = buffer.channels();
        let num_samples = buffer.len_frames();
        debug_assert!(num_channels <= self.num_channels());
        debug_assert!(num_samples <= self.num_samples());

        for ch in 0..num_channels {
            for smp in 0..num_samples {
                self.data[ch][smp] = buffer[smp * num_channels + ch];
            }
        }
    }

    /// Copies the contents of the `OversamplingBuffer` into `buffer`. This essentially
    /// just copies the 2-dimensional layout of `self` into the interleaved layout of
    /// `buffer`.
    ///
    /// This method does not allocate.
    ///
    /// # Panics
    ///
    /// Panics if the channel or sample count of `buffer` does not match the
    /// `OversamplingBuffer`'s.
    pub fn copy_to_buffer(&self, buffer: &mut Buffer<f64>) {
        let num_channels = buffer.channels();
        let num_samples = buffer.len_frames();
        debug_assert!(num_channels <= self.num_channels());
        debug_assert!(num_samples <= self.num_samples());

        for ch in 0..num_channels {
            for smp in 0..num_samples {
                buffer[smp * num_channels + ch] = self.data[ch][smp];
            }
        }
    }

    /// Resizes the buffer.
    pub fn resize(&mut self, num_channels: usize, num_samples: usize) {
        for ch in &mut self.data {
            ch.resize(num_samples, 0.0);
        }

        self.data.resize(num_channels, vec![0.0; num_samples]);
    }

    /// Clears the buffer. Use [`resize()`](Self::resize) to reallocate space.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Returns an immutable slice to one of the buffer's channels.
    ///
    /// # Panics
    ///
    /// Panics if `channel_idx` is greater than [`num_channels()`](Self::num_channels)
    pub fn channel(&self, channel_idx: usize) -> &[f64] {
        &self.data[channel_idx]
    }

    /// Returns a mutable slice to one of the buffer's channels.
    ///
    /// # Panics
    ///
    /// Panics if `channel_idx` is greater than [`num_channels()`](Self::num_channels)
    pub fn channel_mut(&mut self, channel_idx: usize) -> &mut [f64] {
        &mut self.data[channel_idx]
    }

    /// Returns the number of channels held in the buffer.
    pub fn num_channels(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of samples held in each of the buffer's channels.
    ///
    /// Returns `0` if the buffer is empty (i.e. contains no data, not `== 0.0`).
    pub fn num_samples(&self) -> usize {
        self.data.first().map_or(0, |x| x.len())
    }
}

impl Deref for OversamplingBuffer {
    type Target = Vec<Vec<f64>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for OversamplingBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
