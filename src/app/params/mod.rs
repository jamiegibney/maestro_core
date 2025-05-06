//! GUI parameters.

mod attachment;
mod midi_cc_attachments;
mod midi_types;
mod mode;
mod state;
pub mod types;
mod updater;

use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
};

use attachment::MIDICCAttachment;
use eme_request::EMERequest;
use hands::hand_types::RawHandPairCOM;
use message::MIDIMessage;
use midi_cc_attachments::build_midi_cc_attachments;
use midi_types::MIDICCIndex;
use timer::TimerThread;
use mode::Mode;
pub use types::*;
use updater::ParameterUpdater;

use super::{args::Arguments, *};
use midi::*;
use osc::*;

pub struct ParameterHandler {
    update_thread: TimerThread,
    updater: Arc<Mutex<ParameterUpdater>>,

    cc_attachments: HashMap<MIDICCIndex, MIDICCAttachment>,
}

pub struct ParameterSenders {
    midi_sender: Arc<Mutex<CCSender<Vec<MIDIMessage>>>>,
    eme_sender: Arc<Mutex<CCSender<EMERequest>>>,
}

pub struct ParameterReceivers {
    pub midi_receiver: CCReceiver<Vec<MIDIMessage>>,
    pub eme_receiver: CCReceiver<EMERequest>,
}

#[allow(clippy::redundant_closure_for_method_calls)]
impl ParameterHandler {
    pub fn new(
        gesture_data: triple_buffer::Output<RawHandPairCOM>,
        args: &Arguments,
    ) -> (Self, ParameterReceivers) {
        let (midi_tx, midi_rx) = bounded_channel(MIDI_MESSAGE_QUEUE_SIZE);
        let (eme_tx, eme_rx) = bounded_channel(EME_OSC_MESSAGE_QUEUE_SIZE);

        let updater = Arc::new(Mutex::new(ParameterUpdater::new(
            ParameterSenders {
                midi_sender: Arc::new(Mutex::new(midi_tx)),
                eme_sender: Arc::new(Mutex::new(eme_tx)),
            },
            gesture_data,
            args,
        )));

        let param_updater = Arc::clone(&updater);
        let update_thread = TimerThread::new(move || {
            if let Ok(mut guard) = param_updater.lock() {
                guard.update_and_send();
            }
        });

        let s = Self {
            update_thread,
            updater,
            cc_attachments: build_midi_cc_attachments(),
        };

        let rx_channels =
            ParameterReceivers { midi_receiver: midi_rx, eme_receiver: eme_rx };

        (s, rx_channels)
    }

    pub fn set_eme_arrangement(&mut self, arrangement_name: &str) {
        if let Ok(mut guard) = self.updater.lock() {
            guard.set_eme_arrangement(arrangement_name);
        }
    }

    pub fn reset_updater(&self) {
        if let Ok(mut guard) = self.updater.lock() {
            guard.reset_delta_time();
            guard.mark_active_midi_ccs_for_update();
        }
    }

    pub fn start_update(&mut self) {
        self.update_thread.start_hz(PARAM_UPDATE_RATE);

        if let Ok(mut guard) = self.updater.lock() {
            // NOTE(jamie): we need to set the arrangement *before* we start
            // playback, otherwise the EME will return an error.
            guard.set_eme_arrangement(DEFAULT_EME_ARRANGEMENT_NAME);
            guard.set_eme_playback(true);
            guard.start_mode_change();
        }
    }

    pub fn stop_update(&mut self) {
        if let Ok(mut guard) = self.updater.lock() {
            guard.set_eme_playback(false);
        }

        self.update_thread.stop();
    }

    pub fn get_name_for_cc(&self, channel: u8, cc: u8) -> Option<&str> {
        self.cc_attachments
            .get(&MIDICCIndex::new(channel, cc))
            .map(|att| att.name())
    }

    pub fn is_14_bit(&self, channel: u8, cc: u8) -> bool {
        self.cc_attachments
            .get(&MIDICCIndex::new(channel, cc))
            .is_some_and(|att| att.is_14_bit())
    }
}
