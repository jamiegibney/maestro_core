//! The whole app's state.

use super::audio::audio_constructor;
use super::audio::*;
use super::view::view;
use super::*;
use crate::app::params::*;
use crate::dsp::{
    BiquadFilter, BiquadParams, Filter, FilterType, ResoBankData,
    ResonatorBankParams, SpectralMask, BUTTERWORTH_Q,
};
use crate::prelude::interp::linear_unclamped;
use atomic::Atomic;
use crossbeam_channel::{unbounded, Receiver, Sender};
use hands::hand_types::RawHandPairCOM;
use hands::HandManager;
use midi::message::MIDIMessage;
use midi::sender::{MIDISender, MIDISenderTimedThread};
use nannou::draw::mesh::Colors;
use nannou::prelude::WindowId as Id;
use nannou_audio::Stream;
use osc::EMERequestOSCSender;
use std::f64::consts::SQRT_2;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{mpsc, Arc, Mutex, RwLock},
    time::Instant,
};
use timer::TimerThread;
use triple_buffer::triple_buffer;

mod constructors;
use constructors::*;

type CallbackTimerRef = Arc<Mutex<Instant>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MIDISendMode {
    MIDIControlChange,
    MIDINote,
}

impl std::fmt::Display for MIDISendMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MIDIControlChange => write!(f, "MIDI CC"),
            Self::MIDINote => write!(f, "MIDI note"),
        }
    }
}

/// The app's model, i.e. its state.
#[allow(clippy::struct_excessive_bools)]
pub struct Model {
    window: Id,

    /// The CPAL audio stream.
    pub audio_stream: nannou_audio::Stream<AudioModel>,
    /// Channels to send messages directly to the audio thread.
    pub audio_senders: Arc<AudioMessageSenders>,

    /// Input to the spectral mask channel, sent to the audio thread.
    pub spectral_mask: triple_buffer::Input<SpectralMask>,

    /// Channel to send voice events (such as killing all voices).
    pub voice_event_sender: mpsc::Sender<VoiceEvent>,

    /// A thread-safe reference to the timer which tracks when the audio
    /// callback was last called.
    pub audio_callback_timer: CallbackTimerRef,

    /// A reference to the sample rate value.
    pub sample_rate_ref: Arc<AtomicF64>,

    /// Current octave for note input (via typing keyboard).
    pub octave: Octave,
    /// A thread-safe reference to the note handler.
    pub note_handler: NoteHandlerRef,
    /// A `HashMap` of the currently-pressed keys.
    pub pressed_keys: HashMap<Key, bool>,

    pub hand_manager: HandManager,

    pub midi_sender: MIDISender,
    pub midi_send_mode: MIDISendMode,
    pub midi_send_value: u8,
    pub midi_send_channel: u8,
    pub show_state_data: bool,

    rx_tx_ports: (u16, u16),

    midi_timed_thread: MIDISenderTimedThread,

    eme_osc_sender: EMERequestOSCSender,
    params: ParameterHandler,

    gesture_input: triple_buffer::Input<RawHandPairCOM>,

    is_sending: bool,
}

impl Model {
    /// Builds the app's `Model`.
    ///
    /// # Panics
    ///
    /// Panics if a new window cannot be initialized.
    #[allow(clippy::too_many_lines)]
    pub fn build(app: &App) -> Self {
        let AudioSystem {
            stream: audio_stream,
            sample_rate_ref,
            senders: audio_senders,
            callback_timer_ref: audio_callback_timer,
            note_handler,
            voice_event_sender,
            spectral_mask,
            reso_bank_data,
        } = build_audio_system();

        let (_w, _h) = (WINDOW_SIZE.x as f32, WINDOW_SIZE.y as f32);

        let window =
            build_window(app, WINDOW_SIZE.x as u32, WINDOW_SIZE.y as u32);

        let audio_senders = Arc::new(audio_senders);
        let audio_senders_cl = Arc::clone(&audio_senders);

        // *** *** *** //

        let args = args::Arguments::from_env();
        if let Err(e) = &args {
            panic!("failed to obtain arguments: {e}");
        }
        let args = args.unwrap();

        let (gesture_input, gesture_output) =
            triple_buffer(&RawHandPairCOM::default());

        let (param_handler, param_receivers) =
            ParameterHandler::new(gesture_output);

        let (eme_sender, osc_receiver) = osc::create_osc_sender_and_receiver(
            &args, param_receivers.eme_receiver,
        )
        .expect("failed to create eme sender & osc receiver");

        // *** *** *** //

        let midi_timed_thread = MIDISenderTimedThread::new(
            "maestro_timed_midi", "maestro", param_receivers.midi_receiver,
        )
        .expect("failed to create timed MIDI sender thread");

        // *** *** *** //

        Self {
            window,

            audio_stream,
            audio_senders,

            octave: Octave::default(), // C3 - B3

            note_handler: Arc::clone(&note_handler),

            pressed_keys: build_pressed_keys_map(),

            audio_callback_timer,

            voice_event_sender,

            hand_manager: HandManager::new(osc_receiver),

            spectral_mask,

            midi_sender: MIDISender::new_with_port_containing(
                "maestro_test_midi", "maestro",
            )
            .expect("failed to create MIDI output"),

            sample_rate_ref,

            rx_tx_ports: (args.osc_rx_port, args.osc_tx_port),

            midi_send_mode: MIDISendMode::MIDIControlChange,
            midi_send_value: 0,
            midi_send_channel: 0,
            show_state_data: true,

            midi_timed_thread,

            eme_osc_sender: eme_sender,
            params: param_handler,
            gesture_input,

            is_sending: false,
        }
    }

    /// Returns the (approximate) sample index for the current moment in time.
    ///
    /// This is **not** a particularly precise method of tracking time events,
    /// but should be more than adequate for things like note events.
    ///
    /// If a lock on the callback timer is not obtained, then `0` is returned.
    /// This doesn't create too much of an issue as note events are still
    /// handled quite quickly in the audio thread.
    pub fn current_sample_idx(&self) -> u32 {
        self.audio_callback_timer.lock().map_or(0, |guard| {
            let samples_exact =
                guard.elapsed().as_secs_f64() * unsafe { SAMPLE_RATE };
            samples_exact.round() as u32 % BUFFER_SIZE as u32
        })
    }

    pub fn send_midi(&mut self) {
        let msg = match self.midi_send_mode {
            MIDISendMode::MIDIControlChange => {
                if self.params.is_14_bit(self.midi_send_channel, self.midi_send_value) { 
                    MIDIMessage::control_change_14_bit(
                        self.midi_send_value, 0, self.midi_send_channel,
                    )
                } 
                else { 
                    MIDIMessage::control_change(
                        self.midi_send_value, 0, self.midi_send_channel,
                    )
                }
            }
            MIDISendMode::MIDINote => MIDIMessage::note_on(
                self.midi_send_value, 0, self.midi_send_channel,
            ),
        };

        if let Err(e) = self.midi_sender.send_direct(&msg) {
            eprintln!("failed to send MIDI message: \"{e}\"");
        }
        // else {
        //     let b = msg.as_bytes();
        //     let bytes = format!("{:b} {:b} {:b}", b[0], b[1], b[2]);
        //     println!("Sending {msg} (bytes: {bytes})");
        // }
    }

    pub fn format_state(&self) -> String {
        format!(
            "OSC/MIDI are {}\nBound to OSC ports #{} (receive) and #{} (send)\nBound to MIDI port \"{}\"",
            if self.is_sending {
                "active (press 'S' to stop)" 
            } 
            else {
                "inactive (press 'T' to start)" 
            },
            self.rx_tx_ports.0,
            self.rx_tx_ports.1,
            self.midi_sender.bound_port_name(),
        )
    }

    pub fn format_midi_ping(&self) -> String {
        let send_mode = self.midi_send_mode.to_string();

        let mut cc_label = if self.midi_send_mode == MIDISendMode::MIDIControlChange {
            self.params
                .get_name_for_cc(
                    self.midi_send_channel,
                    self.midi_send_value
                )
                .map_or_else(
                    || String::from("no attachment"),
                    |s| {
                        format!("\"{s}{}\"", 
                            if self.params.is_14_bit(self.midi_send_channel,
                                self.midi_send_value) { " (14-bit)" } 
                            else { "" }
                        )
                    },
                )
        } 
        else {
            String::new()
        };

        format!(
            "Ready to ping {send_mode} #{} to channel {} ({cc_label})",
            self.midi_send_value,
            self.midi_send_channel + 1,
        )
    }

    pub fn send_and_update(&mut self, send_update: bool) {
        if send_update {
            self.hand_manager.start_update();
            self.eme_osc_sender.start_send();
            self.midi_timed_thread.start_send();

            self.params.reset_updater();
            self.params.start_update();
        }
        else {
            self.hand_manager.stop_update();
            self.eme_osc_sender.stop_send();
            self.midi_timed_thread.stop_send();

            self.params.stop_update();
        }

        self.is_sending = send_update;
    }
}

impl Updatable for Model {
    fn update(&mut self, update: &Update) {
        self.hand_manager.update(update);
        self.gesture_input.write(*self.hand_manager.damped_hands());
    }
}

impl Drawable for Model {
    fn draw(&self, draw: &Draw, frame: &Frame) {
        if !self.show_state_data {
            return;
        }

        let ping_msg = self.format_midi_ping();
        let state_msg = self.format_state();

        let r = frame.rect();
        let bottom = r.bottom();

        draw.text(&ping_msg)
            .line_spacing(4.5)
            .xy(vec2(0.0, bottom + 80.0))
            .wh(vec2(800.0, 80.0))
            .justify(text::Justify::Center)
            .font_size(16);

        draw.text(&state_msg)
            .color(Rgba::new(0.5, 0.5, 0.5, 1.0))
            .line_spacing(4.5)
            .xy(vec2(0.0, bottom + 40.0))
            .wh(vec2(400.0, 80.0))
            .justify(text::Justify::Center)
            .font_size(12);
    }
}
