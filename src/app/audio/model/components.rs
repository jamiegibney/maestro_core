//! Audio state.

use crossbeam_channel::{Receiver as CCReceiver, Sender as CCSender};
use std::time::Instant;

use super::*;

/// All signal processors.
#[derive(Default)]
pub struct AudioProcessors {
    //
}

/// Audio generation types.
#[derive(Default)]
pub struct AudioGeneration {
    //
}

/// Audio-related data.
pub struct AudioData {
    pub voice_gain: Smoother<f64>,
    // pub master_gain: Arc<SmootherAtomic<f64>>,
    pub sample_rate: Arc<AtomicF64>,
    pub upsampled_rate: Arc<AtomicF64>,

    pub latency_samples: u32,

    pub oversampling_factor: Arc<AtomicUsize>,

    pub is_processing: bool,
    pub idle_timer_samples: u64,

    pub average_load: Vec<f64>,
    pub average_pos: usize,

    pub delay_time_ms: f64,

    pub sample_timer: u32,

    pub callback_time_elapsed: Arc<Mutex<Instant>>,
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            voice_gain: Smoother::default(),
            // master_gain: Arc::new(SmootherAtomic::default()),
            sample_rate: Arc::default(),
            upsampled_rate: Arc::default(),

            latency_samples: Default::default(),

            oversampling_factor: Arc::default(),

            is_processing: Default::default(),
            idle_timer_samples: Default::default(),

            average_load: Vec::default(),
            average_pos: Default::default(),

            delay_time_ms: 250.0,

            sample_timer: 0,

            callback_time_elapsed: Arc::new(Mutex::new(Instant::now())),
        }
    }
}

/// Audio-related buffers.
#[derive(Default)]
pub struct AudioBuffers {
    pub master_gain_buffer: Vec<f64>,

    pub oversampling_buffer: OversamplingBuffer,
}

/// The fields of this struct are used to communicate directly
/// with the audio thread.
#[derive(Default)]
pub struct AudioMessageReceivers {
    pub note_event: Option<CCReceiver<NoteEvent>>,
}

/// Audio message channel senders.
pub struct AudioMessageSenders {
    pub note_event: CCSender<NoteEvent>,
}
