//! JUCE-based oversampler.

use super::block::*;
use super::equiripple_fir::OversamplingEquiripple;
use super::*;

pub struct Oversampler {
    num_channels: usize,
    factor: usize,
    stages: Vec<OversamplingEquiripple>,
    is_ready: bool,
    integer_latency: bool,
    delay: Vec<RingBuffer>,
    fractional_delay: f64,
}

// TODO add more oversampling methods
impl Oversampler {
    /// Creates a new `Oversampler` which uses an equiripple filtering method.
    /// `factor` affects the number of stages in the oversampling, where each stage
    /// equates to a 2x oversampling rate. In other words, the fast domain rate is
    /// `2^factor`.
    ///
    /// **Important**: you must call the [`initialize_stages()`](Self::initialize_stages)
    /// method before processing with the `Oversampler`.
    ///
    /// `use_max_quality` decreases the transition widths and stopband amplitudes of
    /// the internal filters.
    ///
    /// # Panics
    ///
    /// Panics if `factor` is outside the range `1 - 4` (inclusive).
    pub fn new(
        num_channels: usize,
        factor: usize,
        max_quality: bool,
        use_integer_latency: bool,
    ) -> Self {
        assert!((1..5).contains(&factor));

        let mut s = Self {
            num_channels,
            factor,
            stages: vec![],
            is_ready: false,
            integer_latency: use_integer_latency,
            delay: vec![RingBuffer::new(8); num_channels],
            fractional_delay: 0.0,
        };

        for n in 0..factor {
            // transition widths
            let transition_width_up = if max_quality { 0.10 } else { 0.12 }
                * if n == 0 { 0.5 } else { 1.0 };
            let transition_width_down = if max_quality { 0.12 } else { 0.15 }
                * if n == 0 { 0.5 } else { 1.0 };

            // stopband amplitudes
            let gain_db_start_up = if max_quality { -90.0 } else { -70.0 };
            let gain_db_start_down = if max_quality { -75.0 } else { -60.0 };
            let gain_db_factor = if max_quality { 10.0 } else { 8.0 };

            s.add_stage(
                transition_width_up,
                gain_db_start_up * gain_db_factor * n as f64,
                transition_width_down,
                gain_db_start_down * gain_db_factor * n as f64,
            );
        }

        s
    }

    /// Adds an oversampling stage to the oversampler (FIR equiripple).
    pub fn add_stage(
        &mut self,
        normalised_transition_width_up: f64,
        stopband_amplitude_db_up: f64,
        normalised_transition_width_down: f64,
        stopband_amplitude_db_down: f64,
    ) {
        self.stages.push(OversamplingEquiripple::create(
            self.num_channels, normalised_transition_width_up,
            stopband_amplitude_db_up, normalised_transition_width_down,
            stopband_amplitude_db_down,
        ))
    }

    /// Clears all oversampling stages, i.e. resets to an oversampling factor of `1`.
    pub fn clear_stages(&mut self) {
        self.stages.clear();
        self.factor = 1;
    }

    /// Determines whether [`latency_samples()`](Self::latency_samples) should return
    /// an integer-valued latency amount.
    pub fn should_use_integer_latency(&mut self, integer_latency: bool) {
        self.integer_latency = integer_latency;
    }

    /// Computes the latency of the oversampler in samples.
    ///
    /// Use the [`should_use_integer_latency()`](Self::should_use_integer_latency)
    /// method to determine whether this value should be exact or an integer.
    pub fn latency_samples(&self) -> f64 {
        let latency = self.uncompensated_latency();

        if self.integer_latency {
            latency + self.fractional_delay
        }
        else {
            latency
        }
    }

    pub fn uncompensated_latency(&self) -> f64 {
        self.stages
            .iter()
            .enumerate()
            .map(|(i, st)| st.latency_samples() / 2.0_f64.powi(i as i32 + 1))
            .sum()
    }

    /// Returns the oversampling factor of the oversampler.
    ///
    /// Note: this is **not** the fast domain sampling rate factor. Use `2^factor` to
    /// obtain the fast domain sampling rate from the oversampler.
    pub fn factor(&self) -> usize {
        self.factor
    }

    /// Initializes the `Oversampler`'s stages for processing. Must be called before processing.
    ///
    /// `samples_in_buffer` must be at least the length of the audio buffers you will pass to
    /// the `Oversampler`. You can freely pass the maximum possible buffer size here if you wish,
    /// if the extra memory usage is acceptable, and the usage of the `Oversampler` will not
    /// be affected in any way.
    pub fn initialize_stages(&mut self, mut samples_in_buffer: usize) {
        assert!(!self.stages.is_empty());

        for stage in &mut self.stages {
            stage.initialize_buffer(samples_in_buffer);
            samples_in_buffer *= stage.factor();
        }

        for ch in &mut self.delay {
            ch.reset();
            ch.resize(samples_in_buffer);
        }

        self.is_ready = true;
        self.reset();
    }

    /// Resets the `Oversampler` as though it was just initialized. This does **not**
    /// clear its stages.
    pub fn reset(&mut self) {
        assert!(!self.stages.is_empty());

        if self.is_ready {
            self.stages.iter_mut().for_each(|st| st.reset());
        }

        self.delay.iter_mut().for_each(|ch| ch.clear());
    }

    /// Upsamples `input_block` and places the result in `upsampled_block`.
    ///
    /// **Important**: this method assumes that both `input_block` and `upsampled_block`
    /// are interleaved. With regards to the upsampled_block, this means that the channels
    /// are still interleaved as usual, but there are `oversampling factor` time as many
    /// samples in the block.
    ///
    /// This method will have no effect if [`initialize_stages`](Self::initialize_stages)
    /// was not called after the `Oversampler` was created.
    ///
    /// # Panics
    ///
    /// Panics if `upsampled_block` is smaller than `input_block.len() *
    /// oversampling_factor`, or if the `Oversampler` contains no stages.
    pub fn process_samples_up(
        &mut self,
        input_block: &[f64],
        upsampled_block: &mut [f64],
    ) {
        if !self.is_ready {
            return;
        }

        assert!(upsampled_block.len() >= input_block.len() * self.factor);
        assert!(!self.stages.is_empty());

        let input_samples = input_block.len() / self.num_channels;

        self.stages[0].process_samples_up(input_block);

        for ch in 0..self.num_channels {
            let mut block =
                self.stages[0].get_processed_samples(input_samples * 2);

            for stage in self.stages.iter_mut().skip(1) {
                stage.process_samples_up(block);
                block = stage.get_processed_samples(block.len() * 2);
            }

            for i in 0..block.len() {
                upsampled_block[i * self.num_channels + ch] = block[i];
            }
        }
    }

    pub fn process_samples_down(&mut self, output_block: &mut [f64]) {
        if !self.is_ready {
            return;
        }

        assert!(!self.stages.is_empty());

        let stages = self.stages.len();
        let mut current_num_samples = output_block.len() / self.num_channels;

        for n in 0..(stages - 1) {
            current_num_samples *= 2;
        }

        for ch in 0..self.num_channels {
            for n in (1..(stages - 1)).rev() {
                let block = self.stages[n - 1]
                    .get_processed_samples_mut(current_num_samples);
                self.stages[n].process_samples_down(block);

                current_num_samples /= 2;
            }
        }

        self.stages[0].process_samples_down(output_block);

        if self.integer_latency && self.fractional_delay.is_sign_positive() {
            //
        }
    }

    pub fn create_oversampling_block_from_interleaved_slice(
        block: &[f64],
        num_channels: usize,
    ) -> OversamplingBlock {
        OversamplingBlock::from_interleaved_slice(block, num_channels)
    }

    /// Internal method to update the `Oversampler`'s delay lines.
    fn update_delay(&mut self) {
        let latency = self.uncompensated_latency();

        self.fractional_delay = 1.0 - (latency - latency.floor());

        if self.fractional_delay == 1.0 {
            self.fractional_delay = 0.0;
        }
        else if self.fractional_delay < 0.618 {
            self.fractional_delay += 1.0;
        }

        for ch in self.delay.iter_mut() {
            ch.set_delay_time(self.fractional_delay);
        }
    }
}
