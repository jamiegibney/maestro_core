//! Audio model constructor.

use super::*;
use crossbeam_channel::{bounded, unbounded};
use std::cell::RefCell;

pub struct AudioModelBuilder {
    /// The audio model.
    model: AudioModel,
    /// A byte which tracks which fields of the `AudioModel` have been set.
    prepared_state: u8,
}

pub struct AudioPackage {
    pub model: AudioModel,
    pub callback_timer_ref: Arc<Mutex<std::time::Instant>>,
    pub sample_rate_ref: Arc<AtomicF64>,
    pub message_channels: AudioMessageSenders,
}

impl AudioModelBuilder {
    /// The bits required for the `AudioModel` to be "prepared".
    const PREPARED_CHECKSUM: u8 = 0b0000_1111;

    /// Initialises a new, default audio model.
    ///
    /// You must call the following methods before using the model:
    ///
    /// - [`processors()`](Self::processors)
    /// - [`generation()`](Self::generation)
    /// - [`data()`](Self::data)
    /// - [`buffers()`](Self::buffers)
    /// - [`params()`](Self::params)
    ///
    /// # Panics
    ///
    /// Panics if the `voice_event_receiver` field of `context` is `None`,
    /// or if the internal thread pool fails to spawn threads.
    pub fn new(mut context: AudioContext) -> Self {
        Self {
            model: AudioModel {
                generation: AudioGeneration::default(),
                processors: AudioProcessors::default(),
                data: AudioData::default(),
                buffers: AudioBuffers::default(),
                voice_handler: VoiceHandler::build(
                    context.voice_event_receiver.take().unwrap(),
                    Arc::new(AtomicF64::new(context.sample_rate)),
                ),
                context,
                message_channels: RefCell::new(AudioMessageReceivers::default()),
                thread_pool: ThreadPool::build(4).unwrap(),
            },
            prepared_state: 0b0000_0000,
        }
    }

    /// Moves `processors` into the `AudioModel`.
    pub fn processors(mut self, processors: AudioProcessors) -> Self {
        self.model.processors = processors;
        self.prepared_state |= 0b0000_0001;
        self
    }

    /// Moves `generation` into the `AudioModel`.
    pub fn generation(mut self, generation: AudioGeneration) -> Self {
        self.model.generation = generation;
        self.prepared_state |= 0b0000_0010;
        self
    }

    /// Moves `data` into the `AudioModel`.
    pub fn data(mut self, data: AudioData) -> Self {
        self.model.data = data;

        self.prepared_state |= 0b0000_0100;
        self
    }

    /// Moves `buffers` into the `AudioModel`.
    pub fn buffers(mut self, buffers: AudioBuffers) -> Self {
        self.model.buffers = buffers;
        self.prepared_state |= 0b0000_1000;
        self
    }

    /// Connects the appropriate values with the UI.
    pub fn params(mut self, ui_params: &ParameterHandler) -> Self {
        self.prepared_state |= 0b0001_0000;
        self.attach_params(ui_params);
        self
    }

    /// Builds the audio model.
    ///
    /// # Panics
    ///
    /// Panics if you haven't called **all** of the following methods:
    ///
    /// - [`processors()`](Self::processors)
    /// - [`generation()`](Self::generation)
    /// - [`data()`](Self::data)
    /// - [`buffers()`](Self::buffers)
    /// - [`params()`](Self::params)
    pub fn build(mut self) -> AudioPackage {
        assert!(
            self.prepared_state == Self::PREPARED_CHECKSUM,
            "AudioModelBuilder::build(): failed to verify preparation checksum, please call all the required methods"
        );

        AudioPackage {
            callback_timer_ref: Arc::clone(
                &self.model.data.callback_time_elapsed,
            ),
            sample_rate_ref: Arc::clone(&self.model.data.sample_rate),
            message_channels: self.message_channels(),
            model: self.model,
        }
    }

    fn message_channels(&mut self) -> AudioMessageSenders {
        let mut msg_ch = self.model.message_channels.borrow_mut();
        let (note_event, receiver) = bounded(MAX_NOTE_EVENTS_PER_BUFFER);
        msg_ch.note_event = Some(receiver);

        AudioMessageSenders {
            note_event,
        }
    }

    pub fn attach_params(&mut self, ui_params: &ParameterHandler) {
        //
    }
}
