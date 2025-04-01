//! Trait for audio-processing types.

/// Generic trait for audio processing effects.
pub trait Effect: dyn_clone::DynClone + Send + std::fmt::Debug {
    /// Optional method to process two stereo samples of audio.
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        (in_l, in_r)
    }

    /// Optional method to process a single sample of audio.
    fn process_mono(&mut self, input: f64, _channel_idx: usize) -> f64 {
        input
    }

    /// Required method to obtain the sample rate of the processor.
    fn get_sample_rate(&self) -> f64;

    /// Required method to obtain the name of the effect processor.
    fn get_identifier(&self) -> &str;
}

// This is used to allow `dyn Effect` trait objects to implement clone.
dyn_clone::clone_trait_object!(Effect);
