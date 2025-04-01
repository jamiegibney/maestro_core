//! Ring buffer for creating delays.

use crate::prelude::*;
use crate::util::interp;

// TODO this is a very high smoothing time for a default value...
const DEFAULT_SMOOTHING_TIME: f64 = 500.0;

/// A resizable ring buffer which supports interpolation and parameter
/// smoothing (for delay time).
#[derive(Debug, Clone, Default)]
pub struct RingBuffer {
    /// The internal data buffer.
    data: Vec<f64>,
    /// The write position of the buffer.
    write_pos: usize,

    /// The smoothed delay time parameter.
    delay_secs: SmootherAtomic<f64>,

    /// The kind of interpolation to use.
    interpolation_type: InterpType,

    smoothing_type: SmoothingType,
    smoothing_time_secs: f64,

    sample_rate: f64,
}

impl RingBuffer {
    /// Returns a new, initialised `RingBuffer` which holds `size` samples.
    ///
    /// Defaults to linear smoothing (see `set_smoothing()`) and interpolation
    /// (see `set_interpolation()`).
    #[must_use]
    pub fn new(size: usize, sample_rate: f64) -> Self {
        Self {
            data: vec![0.0; size],
            write_pos: 0,

            delay_secs: SmootherAtomic::new(
                DEFAULT_SMOOTHING_TIME, 0.0, sample_rate,
            ),

            interpolation_type: InterpType::default(), // linear

            smoothing_type: SmoothingType::default(),
            smoothing_time_secs: DEFAULT_SMOOTHING_TIME,

            sample_rate,
        }
    }

    /// Sets the smoothing method and time for the `RingBuffer`. This affects
    /// how the buffer responds to changes in delay time.
    ///
    /// Constructing method.
    pub fn with_smoothing(
        mut self,
        smoothing_type: SmoothingType,
        smoothing_time_secs: f64,
    ) -> Self {
        self.set_smoothing(smoothing_type, smoothing_time_secs);
        self
    }

    /// Sets the interpolation method for the `RingBuffer`. This affects how
    /// the buffer handles delay times which lie between samples.
    ///
    /// Constructing method.
    pub fn with_interpolation(
        mut self,
        interpolation_type: InterpType,
    ) -> Self {
        self.set_interpolation(interpolation_type);
        self
    }

    /// Pushes `element` to the `RingBuffer`.
    pub fn push(&mut self, element: f64) {
        self.data[self.write_pos] = element;
        self.increment_write_pos();
    }

    /// Reads the delayed element from the `RingBuffer`.
    pub fn read(&mut self) -> f64 {
        use InterpType as IT;
        let (read_pos, interp) = self.get_read_pos_and_interp();
        // r1 is the same as read_pos
        let (r0, r1, r2, r3) =
            if matches!(self.interpolation_type, IT::NoInterp) {
                (0, read_pos, 0, 0)
            }
            else {
                self.get_cubic_read_positions(read_pos)
            };

        match self.interpolation_type {
            IT::NoInterp => self.data[r1],
            IT::Linear => lerp(self.data[r1], self.data[r2], interp),
            IT::Cosine => interp::cosine(self.data[r1], self.data[r2], interp),
            IT::DefaultCubic => interp::cubic(
                self.data[r0], self.data[r1], self.data[r2], self.data[r3],
                interp,
            ),
            IT::CatmullCubic => interp::cubic_catmull(
                self.data[r0], self.data[r1], self.data[r2], self.data[r3],
                interp,
            ),
            IT::HermiteCubic(tension, bias) => interp::cubic_hermite(
                self.data[r0], self.data[r1], self.data[r2], self.data[r3],
                interp, tension, bias,
            ),
        }
    }

    /// Sets the delay time of the `RingBuffer` in seconds.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `delay_secs` is greater than the maximum delay
    /// time of the `RingBuffer` in seconds to avoid buffer overruns.
    ///
    /// In release mode, this may cause buffer overruns.
    pub fn set_delay_time(&mut self, delay_secs: f64) {
        debug_assert!(delay_secs <= self.max_delay_secs());

        if !epsilon_eq(delay_secs, self.delay_secs.current_value()) {
            self.delay_secs.set_target_value(delay_secs);
        }
    }

    /// Sets the smoothing method and time for the `RingBuffer`. This affects
    /// how the buffer responds to changes in delay time.
    pub fn set_smoothing(
        &mut self,
        smoothing_type: SmoothingType,
        smoothing_time_secs: f64,
    ) {
        self.delay_secs.set_smoothing_type(smoothing_type);

        self.smoothing_type = smoothing_type;
        self.smoothing_time_secs = smoothing_time_secs;
    }

    /// Sets the interpolation method for the `RingBuffer`. This affects how
    /// the buffer handles delay times which lie between samples.
    pub fn set_interpolation(&mut self, interpolation_type: InterpType) {
        self.interpolation_type = interpolation_type;
    }

    /// Resets the `RingBuffer` to its default settings. Does not allocate.
    pub fn reset(&mut self) {
        self.data.clear();

        self.write_pos = 0;

        self.delay_secs.reset();
        self.delay_secs.set_smoothing_period(DEFAULT_SMOOTHING_TIME);

        self.interpolation_type = InterpType::default();

        self.smoothing_type = SmoothingType::default();
        self.smoothing_time_secs = DEFAULT_SMOOTHING_TIME;
    }

    /// Sets the internal sample rate.
    ///
    /// # Panics
    ///
    /// Panics if `sample_rate` is negative.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        assert!(sample_rate.is_sign_positive());
        self.sample_rate = sample_rate;
    }

    /// Returns the internal sample rate of the `RingBuffer`.
    pub fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }

    /// Clears the contents of the buffer, i.e. sets its contents to `0.0`.
    pub fn clear(&mut self) {
        self.data.iter_mut().for_each(|x| *x = 0.0);
    }

    /// # Safety
    ///
    /// This may reallocate memory, so you should not call this on the audio
    /// thread or in real-time usage.
    pub fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size, 0.0);
    }

    /// Returns the number of elements held by the `RingBuffer`.
    #[must_use]
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns the maximum delay time possible for the current sample rate.
    pub fn max_delay_secs(&self) -> f64 {
        let size = self.size() as f64;
        size / self.sample_rate
    }

    fn get_read_pos_and_interp(&mut self) -> (usize, f64) {
        let delay_samples = self.sample_rate * self.delay_secs.next();
        // the exact delay sample, i.e. read position
        let samples_exact = delay_samples.floor();
        // the interpolation between this sample and the next
        let interp = delay_samples - samples_exact;

        let samples_exact = samples_exact as usize;
        let read_pos = if samples_exact > self.write_pos {
            let overrun = self.size() + self.write_pos;
            overrun - samples_exact
        }
        else {
            self.write_pos - samples_exact
        };

        (read_pos, interp)
    }

    /// Returns the read positions +1, at, -1, and -2 relative to the read
    /// position, used for interpolation.
    // TODO: try to account for delay time less than 2 samples?
    fn get_cubic_read_positions(
        &self,
        read_pos: usize,
    ) -> (usize, usize, usize, usize) {
        let size = self.size();
        let r0 = (read_pos + 1) % size;
        let r2 = if read_pos == 0 { size - 1 } else { read_pos - 1 };
        let r3 = if read_pos <= 1 { size - 2 + read_pos } else { read_pos - 2 };

        (r0, read_pos, r2, r3)
    }

    fn increment_write_pos(&mut self) {
        let size = self.size();

        self.write_pos += 1;
        if size <= self.write_pos {
            self.write_pos = 0;
        }
    }
}
