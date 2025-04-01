//! A simple stereo delay.

use super::*;

/// A stereo delay processor, capable of ping-pong effects via channel
/// cross-feeding.
#[derive(Clone, Debug, Default)]
pub struct StereoDelay {
    buffer_l: RingBuffer,
    buffer_r: RingBuffer,
    feedback_amount: f64,
    use_ping_pong: bool,
}

impl StereoDelay {
    /// Creates a new stereo delay with `max_delay_time_secs` maximum time allocated.
    pub fn new(max_delay_time_secs: f64, sample_rate: f64) -> Self {
        let buffer = RingBuffer::new(
            (max_delay_time_secs * sample_rate) as usize,
            sample_rate,
        )
        .with_smoothing(delay::DEFAULT_DELAY_SMOOTHING, 0.1)
        .with_interpolation(InterpType::DefaultCubic);
        Self {
            buffer_l: buffer.clone(),
            buffer_r: buffer,
            feedback_amount: 0.0,
            use_ping_pong: false,
        }
    }

    /// Sets the initial delay time of the stereo delay.
    pub fn with_delay_time(mut self, delay_secs: f64) -> Self {
        self.set_delay_time(delay_secs);
        self
    }

    /// Sets the initial delay time of the stereo delay in samples.
    pub fn with_delay_time_samples(mut self, delay_samples: f64) -> Self {
        self.set_delay_time_samples(delay_samples);
        self
    }

    /// Sets whether the stereo delay is initialized with ping-pong or not.
    pub fn with_ping_pong(self, use_ping_pong: bool) -> Self {
        Self { use_ping_pong, ..self }
    }

    /// Sets the delay time of the delay in seconds.
    pub fn set_delay_time(&mut self, delay_secs: f64) {
        self.buffer_l.set_delay_time(delay_secs);
        self.buffer_r.set_delay_time(delay_secs);
    }

    /// Sets the delay time of the delay in samples.
    pub fn set_delay_time_samples(&mut self, delay_samples: f64) {
        self.set_delay_time(delay_samples / self.get_sample_rate());
    }

    /// Sets the feedback level of the delay.
    pub fn set_feedback_amount(&mut self, feedback: f64) {
        self.feedback_amount = feedback.clamp(0.0, 1.0);
    }

    /// Returns the maximum delay time of the stereo delay.
    pub fn max_delay_time_secs(&self) -> f64 {
        self.buffer_l.max_delay_secs()
    }

    /// Sets whether the stereo delay uses ping-pong or not.
    pub fn ping_pong(&mut self, use_ping_pong: bool) {
        self.use_ping_pong = use_ping_pong;
    }

    /// Sets the smoothing time of the stereo delay.
    pub fn set_smoothing_time(&mut self, smoothing_time_secs: f64) {
        self.buffer_l
            .set_smoothing(delay::DEFAULT_DELAY_SMOOTHING, smoothing_time_secs);
        self.buffer_r
            .set_smoothing(delay::DEFAULT_DELAY_SMOOTHING, smoothing_time_secs);
    }

    /// Resets the sample rate of the stereo delay.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.buffer_l.set_sample_rate(sample_rate);
        self.buffer_r.set_sample_rate(sample_rate);
    }
}

impl Effect for StereoDelay {
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let out_l = self.buffer_l.read();
        let out_r = self.buffer_r.read();
        if self.use_ping_pong {
            self.buffer_l.push(in_l + out_r * self.feedback_amount);
            self.buffer_r.push(out_l * self.feedback_amount);
        }
        else {
            self.buffer_l
                .push(out_l.mul_add(self.feedback_amount, in_l));
            self.buffer_r
                .push(out_r.mul_add(self.feedback_amount, in_r));
        }

        (out_l, out_r)
    }

    fn get_sample_rate(&self) -> f64 {
        self.buffer_l.get_sample_rate()
    }

    fn get_identifier(&self) -> &str {
        "stereo_delay"
    }
}
