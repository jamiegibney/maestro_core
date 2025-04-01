//! The app's audio state model.

use super::*;
use crate::prelude::xfer::smooth_soft_clip;
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;

pub mod audio_constructor;
pub mod components;
pub mod builder;
pub use components::*;

/// When the DSP stops , it will continue to process for this length of time to
/// allow the audio spectrums to fully relax. After this time has passed, the
/// DSP is skipped to reduce total load when idle.
const DSP_IDLE_HOLD_TIME_SECS: f64 = 0.8;

/// The program's audio state.
pub struct AudioModel {
    /// Fields related to audio generation (envelopes, oscillators, ...).
    pub generation: AudioGeneration,
    /// Signal processors — both musical FX and DSP-related.
    pub processors: AudioProcessors,

    /// Audio-related data (gain, oversampling state, ...).
    pub data: AudioData,
    pub buffers: AudioBuffers,

    /// The audio thread's voice handler.
    pub voice_handler: VoiceHandler,
    /// Audio-related contextual data.
    pub context: AudioContext,

    /// Message receiving channels.
    pub message_channels: RefCell<AudioMessageReceivers>,

    // /// All audio-related parameters linked to the UI.
    // pub params: ParameterHandler,

    /// The audio thread pool, intended for processing the spectrograms
    /// asynchronously.
    thread_pool: ThreadPool,
}

impl AudioModel {
    /// Sets the DSP idle timer.
    pub fn set_idle_timer(&mut self, is_processing: bool) {
        self.data.idle_timer_samples = if is_processing {
            (self.data.sample_rate.load(Relaxed) * DSP_IDLE_HOLD_TIME_SECS)
                as u64
        }
        else if self.data.idle_timer_samples > 0 {
            self.data.idle_timer_samples - 1
        }
        else {
            0
        };
    }

    /// Determines whether the audio thread is idle or not.
    pub fn is_idle(&self) -> bool {
        !self.data.is_processing && self.data.idle_timer_samples == 0
    }

    /// # Panics
    ///
    /// Panics if the callback timer cannot be locked.
    pub fn current_sample_idx(&self) -> u32 {
        let guard = self.data.callback_time_elapsed.lock().unwrap();

        let samples_exact =
            guard.elapsed().as_secs_f64() * self.data.sample_rate.load(Relaxed);

        drop(guard);

        samples_exact.round() as u32 % BUFFER_SIZE as u32
    }

    /// Returns the internal sample rate of the audio model.
    pub fn get_sample_rate(&self) -> f64 {
        self.data.sample_rate.lr()
    }

    /// Returns the internal upsampled rate of the audio model.
    pub fn get_upsampled_rate(&self) -> f64 {
        self.data.upsampled_rate.lr()
    }

    /// Returns the next available note event, if it exists.
    pub fn next_note_event(&self) -> Option<NoteEvent> {
        self.message_channels
            .borrow()
            .note_event
            .as_ref()
            .and_then(|ch| ch.try_recv().ok())
    }

    pub fn increment_sample_count(&mut self, buffer_size: u32) {
        let time = 6.0;
        let tmr = (time * self.data.sample_rate.lr()) as u32;

        self.data.sample_timer += buffer_size;
        if self.data.sample_timer > tmr {
            self.data.sample_timer -= tmr;
        }
    }

    /// Updates the internal state of the post-processors.
    #[allow(clippy::too_many_lines)]
    pub fn update_post_processors(&mut self) {
        //
    }

    /// Processes all filters.
    pub fn process_filters(&mut self, mut sample: f64, ch_idx: usize) -> f64 {
        //

        sample
    }
}
