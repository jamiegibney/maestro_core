//! Polyphonic voice types and management.

use atomic::Atomic;
use nannou_audio::Buffer;
use std::sync::{mpsc, Arc, Mutex};

use super::audio_note::NoteHandler;
use crate::app::ExciterOscillator;
use crate::dsp::synthesis::*;
use crate::dsp::*;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VoiceEvent {
    ReleaseAll,
    KillAll,
}

/// A struct to represent each individual voice.
#[derive(Clone, Debug)]
pub struct Voice {
    /// The voice's unique ID.
    pub id: u64,
    /// The MIDI note of the voice.
    pub note: f64,

    /// The voice's ADSR envelope.
    pub envelope: AdsrEnvelope,

    /// Whether or not the voice is currently releasing, which contains
    /// the number of samples left until the voice should be cleared.
    pub releasing: bool,

    pub sample_rate: Arc<AtomicF64>,

    /// The type of generator to use.
    pub generator_type: Arc<Atomic<ExciterOscillator>>,
    pub curr_generator: ExciterOscillator,

    /// The audio generator stored within the voice.
    pub generator: Generator,
}

impl Voice {
    pub fn new(
        id: u64,
        note: f64,
        generator: Generator,
        generator_type_ref: Arc<Atomic<ExciterOscillator>>,
        sample_rate: Arc<AtomicF64>,
        envelope: Option<AdsrEnvelope>,
    ) -> Self {
        Self {
            id,
            note,
            envelope: envelope.unwrap_or_default(),
            releasing: false,
            sample_rate,
            curr_generator: generator_type_ref.lr(),
            generator_type: generator_type_ref,
            generator,
        }
    }

    pub fn update_generator(&mut self) {
        let new_type = self.generator_type.lr();

        if new_type == self.curr_generator {
            return;
        }

        self.curr_generator = new_type;

        let freq = note_to_freq(self.note);
        let sample_rate = self.sample_rate.lr();

        self.generator = match new_type {
            ExciterOscillator::Sine => {
                Generator::Sine(SineOsc::new(freq, sample_rate))
            }
            ExciterOscillator::Tri => {
                Generator::Tri(TriOsc::new(freq, sample_rate))
            }
            ExciterOscillator::Saw => {
                Generator::Saw(Phasor::new(freq, sample_rate))
            }
            ExciterOscillator::Square => {
                Generator::Square(SquareOsc::new(freq, sample_rate))
            }
            ExciterOscillator::Noise => Generator::Noise,
        }
    }
}

/// A struct to handle all voices, i.e. the spawning and termination of voices.
#[derive(Debug)]
pub struct VoiceHandler {
    /// A reference to the note handler to obtain note events.
    // pub note_handler_ref: Arc<Mutex<NoteHandler>>,
    /// The array of voices.
    pub voices: [Option<Voice>; NUM_VOICES as usize],
    voice_event_receiver: mpsc::Receiver<VoiceEvent>,
    /// Internal counter for assigning new IDs.
    id_counter: u64,
    generator: Option<Arc<Atomic<ExciterOscillator>>>,
    sample_rate: Arc<AtomicF64>,
}

impl VoiceHandler {
    /// Builds a new `VoiceHandler` with a reference to the `NoteHandler`.
    ///
    /// The `NoteHandler` reference is used to obtain new note events
    /// automatically.
    pub fn build(
        voice_event_receiver: mpsc::Receiver<VoiceEvent>,
        sample_rate_ref: Arc<AtomicF64>,
    ) -> Self {
        Self {
            // note_handler_ref,
            voices: std::array::from_fn(|_| None),
            voice_event_receiver,
            id_counter: 0,
            generator: None,
            sample_rate: sample_rate_ref,
        }
    }

    /// Attaches the current generator oscillator to the `VoiceHandler`.
    pub fn attach_generator_osc(
        &mut self,
        generator: Arc<Atomic<ExciterOscillator>>,
    ) {
        self.generator = Some(generator);
    }

    /// Attaches a reference to the sample rate to the `VoiceHandler`.
    pub fn attach_sample_rate_ref(&mut self, sample_rate_ref: Arc<AtomicF64>) {
        self.sample_rate = sample_rate_ref;
    }

    pub fn process_block(
        &mut self,
        buffer: &mut Buffer<f64>,
        block_start: usize,
        block_end: usize,
        gain: [f64; MAX_BLOCK_SIZE],
    ) {
        let block_len = block_end - block_start;
        let mut voice_amp_envelope = [0.0; MAX_BLOCK_SIZE];

        // process any received voice events
        if let Ok(msg) = self.voice_event_receiver.try_recv() {
            match msg {
                VoiceEvent::ReleaseAll => {
                    self.start_release_for_active_voices();
                }
                VoiceEvent::KillAll => self.kill_active_voices(),
            }
        }

        for voice in self.voices.iter_mut().filter_map(|v| v.as_mut()) {
            voice
                .envelope
                .next_block(&mut voice_amp_envelope, block_len);

            voice.update_generator();

            for (value_idx, sample_idx) in (block_start..block_end).enumerate()
            {
                let amp = gain[value_idx] * voice_amp_envelope[value_idx];

                let (sample_l, sample_r) = voice.generator.process();

                // * 2 because the channels are interleaved
                buffer[sample_idx * 2] += sample_l * amp;
                buffer[sample_idx * 2 + 1] += sample_r * amp;
            }
        }
    }

    /// Starts a new voice.
    #[allow(clippy::missing_panics_doc)] // this function should not panic
    pub fn start_voice(
        &mut self,
        note: f64,
        sample_rate: f64,
        envelope: Option<AdsrEnvelope>,
    ) -> &mut Voice {
        let next_voice_id = self.next_voice_id();
        let gen = self
            .generator
            .as_ref()
            .expect("expected reference to generator type");

        let mut new_voice = Voice {
            id: next_voice_id,
            note,
            envelope: envelope.unwrap_or_default(),
            releasing: false,
            sample_rate: Arc::clone(&self.sample_rate),
            generator_type: Arc::clone(gen),
            curr_generator: ExciterOscillator::Noise,
            generator: { Generator::Noise },
        };

        new_voice.update_generator();

        new_voice.envelope.set_trigger(true);

        // is there a free voice?
        if let Some(free_idx) =
            self.voices.iter().position(|voice| voice.is_none())
        {
            self.voices[free_idx] = Some(new_voice);
            return self.voices[free_idx].as_mut().unwrap();
        }

        // as we know voices are in use, we can use unwrap_unchecked()
        // to avoid some unnecessary checks.
        let oldest_voice = unsafe {
            self.voices
                .iter_mut()
                .min_by_key(|voice| voice.as_ref().unwrap_unchecked().id)
                .unwrap_unchecked()
        };

        *oldest_voice = Some(new_voice);
        return oldest_voice.as_mut().unwrap();
    }

    /// Starts a voice's release stage.
    pub fn start_release_for_voice(
        &mut self,
        voice_id: Option<u64>,
        note: f64,
    ) {
        for voice in &mut self.voices {
            match voice {
                Some(Voice {
                    id: candidate_id,
                    note: candidate_note,
                    releasing,
                    envelope,
                    ..
                }) if voice_id == Some(*candidate_id)
                    || note == *candidate_note =>
                {
                    *releasing = true;
                    envelope.set_trigger(false);
                }
                _ => (),
            }
        }
    }

    /// Starts the release stage for all active voices.
    pub fn start_release_for_active_voices(&mut self) {
        self.voices.iter_mut().for_each(|v| {
            if let Some(voice) = v {
                voice.releasing = true;
                voice.envelope.set_trigger(false);
            }
        });
    }

    /// Immediately terminates all active voices.
    pub fn kill_active_voices(&mut self) {
        self.voices.iter_mut().for_each(|v| {
            if v.is_some() {
                *v = None;
            }
        });
    }

    /// Terminates all voices which are releasing and which have an
    /// idle envelope.
    pub fn terminate_finished_voices(&mut self) {
        for voice in &mut self.voices {
            match voice {
                Some(v) if v.releasing && v.envelope.is_idle() => {
                    *voice = None;
                }
                _ => (),
            }
        }
    }

    /// Returns whether there is at least one voice active or not.
    pub fn is_voice_active(&self) -> bool {
        self.voices.iter().any(|v| v.is_some())
    }

    fn next_voice_id(&mut self) -> u64 {
        self.id_counter = self.id_counter.wrapping_add(1);
        self.id_counter
    }
}
