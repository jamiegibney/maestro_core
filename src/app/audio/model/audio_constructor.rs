//! Audio state constructors.

use super::builder::*;
use super::*;
use crate::dsp::*;
use atomic_float::AtomicF64;
use std::sync::atomic::AtomicUsize;
use triple_buffer::Output;

pub const DEFAULT_SPECTRAL_BLOCK_SIZE: usize = 1 << 10; // 1024
pub const DEFAULT_GAIN: f64 = 1.5;
pub const MAX_NUM_RESONATORS: usize = 32;

pub fn build_audio_model(
    mut context: AudioContext,
) -> AudioPackage {
    let sr = context.sample_rate;
    AudioModelBuilder::new(context)
        .processors(audio_processors(sr, sr))
        .generation(audio_generation(sr))
        .data(audio_data(sr, sr))
        .buffers(audio_buffers())
        .build()
}

#[allow(clippy::too_many_lines)]
fn audio_processors(
    sample_rate: f64,
    upsampled_rate: f64,
) -> AudioProcessors {
    AudioProcessors {}
}

fn audio_generation(sample_rate: f64) -> AudioGeneration {
    AudioGeneration {}
}

fn audio_data(
    sample_rate: f64,
    upsampled_rate: f64,
) -> AudioData {
    AudioData {
        voice_gain: Smoother::new(1.0, 0.01, sample_rate),
        // master_gain: Arc::new(SmootherAtomic::new(
        //     1.0, DEFAULT_GAIN, upsampled_rate,
        // )),
        sample_rate: Arc::new(AtomicF64::new(sample_rate)),
        upsampled_rate: Arc::new(AtomicF64::new(upsampled_rate)),
        latency_samples: 0,
        oversampling_factor: Arc::new(AtomicUsize::new(
            DEFAULT_OVERSAMPLING_FACTOR,
        )),
        is_processing: false,
        idle_timer_samples: 0,
        average_load: vec![0.0; DSP_LOAD_AVERAGING_SAMPLES],
        average_pos: 0,
        sample_timer: 0,
        callback_time_elapsed: Arc::new(Mutex::new(std::time::Instant::now())),

        delay_time_ms: 250.0,
    }
}

fn audio_buffers() -> AudioBuffers {
    AudioBuffers {
        master_gain_buffer: vec![
            DEFAULT_GAIN;
            BUFFER_SIZE
                * (1 << DEFAULT_OVERSAMPLING_FACTOR)
        ],
        oversampling_buffer: OversamplingBuffer::new(
            NUM_CHANNELS, MAX_BUFFER_SIZE,
        ),
    }
}
