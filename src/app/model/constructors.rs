//! App constructors.

use super::*;
use crate::app::audio::audio_constructor::MAX_NUM_RESONATORS;
use crate::dsp::ResoBankData;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc;

/// Builds the app window.
pub fn build_window(app: &App, width: u32, height: u32) -> Id {
    app.new_window()
        .size(width, height)
        .resizable(true)
        .msaa_samples(1)
        .key_pressed(keys::key_pressed)
        .key_released(keys::key_released)
        .view(view)
        .title("Maestro")
        .build()
        .expect("failed to build app window!")
}

pub struct AudioSystem {
    pub(super) stream: Stream<AudioModel>,
    pub(super) sample_rate_ref: Arc<AtomicF64>,
    pub(super) senders: AudioMessageSenders,
    pub(super) callback_timer_ref: CallbackTimerRef,
    pub(super) note_handler: NoteHandlerRef,
    pub(super) voice_event_sender: mpsc::Sender<VoiceEvent>,
    pub(super) spectral_mask: triple_buffer::Input<SpectralMask>,
    pub(super) reso_bank_data: triple_buffer::Input<ResoBankData>,
}

/// Builds the audio stream, audio message channel senders, and input note
/// handler.
pub fn build_audio_system() -> AudioSystem {
    set_sample_rate();

    // setup audio structs
    let note_handler = Arc::new(Mutex::new(NoteHandler::new()));
    let (spectral_mask, spectral_mask_output) =
        triple_buffer::TripleBuffer::new(&SpectralMask::new(
            MAX_SPECTRAL_BLOCK_SIZE,
        ))
        .split();

    let (reso_bank_data, reso_bank_data_output) =
        triple_buffer::TripleBuffer::new(&ResoBankData::new(
            MAX_NUM_RESONATORS,
        ))
        .split();

    let (voice_event_sender, voice_event_receiver) = mpsc::channel();
    let (note_channel_sender, note_channel_receiver) = mpsc::channel();

    // build the audio context
    let audio_context = AudioContext {
        note_channel_receiver,
        sample_rate: unsafe { SAMPLE_RATE },
        spectral_mask_output: Some(spectral_mask_output),
        reso_bank_data_output: Some(reso_bank_data_output),
        voice_event_sender: voice_event_sender.clone(),
        voice_event_receiver: Some(voice_event_receiver),
    };

    // setup audio stream
    let audio_host = nannou_audio::Host::new();

    let builder::AudioPackage {
        model: audio_model,
        callback_timer_ref,
        sample_rate_ref,
        message_channels: senders,
    } = audio_constructor::build_audio_model(audio_context);

    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio::process)
        .channels(NUM_CHANNELS)
        .sample_rate(sample_rate_ref.load(Relaxed) as u32)
        .frames_per_buffer(BUFFER_SIZE)
        .build()
        .unwrap();

    stream.play().unwrap();

    // construct audio system
    AudioSystem {
        stream,
        sample_rate_ref,
        senders,
        callback_timer_ref,
        note_handler,
        voice_event_sender,
        spectral_mask,
        reso_bank_data,
    }
}

/// Builds the `HashMap` used to track which keys are currently pressed or not.
pub fn build_pressed_keys_map() -> HashMap<Key, bool> {
    let mut map = HashMap::new();

    for k in KEYBOARD_MIDI_NOTES {
        map.insert(k, false);
    }

    map
}

fn set_sample_rate() {
    unsafe {
        SAMPLE_RATE = nannou_audio::Host::new().default_output_device().map_or(
            SAMPLE_RATE,
            |device| {
                device
                    .supported_output_configs()
                    .map_or(SAMPLE_RATE, |cfg| {
                        cfg.map(|x| x.min_sample_rate().0)
                            .min()
                            .unwrap_or(SAMPLE_RATE as u32)
                            as f64
                    })
            },
        );
    }
}
