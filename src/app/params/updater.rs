use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
};

use std::f32::consts::TAU;

use super::*;
use attachment::*;
use eme_request::{EMEPlayback, EMEPosition};
use hands::{
    hand_types::{COMPair, SignificantHandValues},
    MAX_HAND_VELOCITY,
};
use midi_cc_attachments::build_midi_cc_attachments;
use midi_types::*;
use rand::seq::IndexedRandom;
use state::ParameterState;

use std::cell::RefCell;

pub(super) struct ParameterUpdater {
    senders: ParameterSenders,

    midi_bank: RefCell<MIDIParameterBank>,

    updated_cc_indices: RefCell<HashSet<MIDICCIndex>>,
    updated_note_indices: HashSet<MIDINoteIndex>,

    gesture_data: triple_buffer::Output<RawHandPairCOM>,

    hands: RawHandPairCOM,
    prev_com: COMPair,
    state: ParameterState,

    hand_velocities: (f32, f32),
    curr_eme_pos: Vec2,

    eme_is_playing: bool,
    eme_is_playing_prev: bool,

    eme_arrangement: String,
    eme_arrangement_prev: String,

    cc_attachments: RefCell<HashMap<MIDICCIndex, MIDICCAttachment>>,

    time_tracker: std::time::Instant,
    computed_delta_time: bool,
    time: f32,

    midi_bytes: usize,
}

impl ParameterUpdater {
    pub fn new(
        senders: ParameterSenders,
        gesture_data: triple_buffer::Output<RawHandPairCOM>,
    ) -> Self {
        let s = Self {
            senders,

            midi_bank: RefCell::new(MIDIParameterBank::new()),

            updated_cc_indices: RefCell::new(HashSet::with_capacity(
                NUM_MIDI_CHANNELS * NUM_MIDI_CCS,
            )),
            updated_note_indices: HashSet::with_capacity(
                NUM_MIDI_CHANNELS * NUM_MIDI_NOTES,
            ),

            gesture_data,

            hands: RawHandPairCOM::default(),
            prev_com: COMPair::default(),
            state: ParameterState::default(),

            hand_velocities: (0.0, 0.0),

            curr_eme_pos: vec2(0.0, 0.5),

            eme_is_playing: false,
            eme_is_playing_prev: false,

            eme_arrangement: String::new(),
            eme_arrangement_prev: String::new(),

            cc_attachments: RefCell::new(build_midi_cc_attachments()),

            time_tracker: std::time::Instant::now(),
            computed_delta_time: false,

            time: 0.0,
            midi_bytes: 0,
        };

        s.set_ccs_from_attachments();

        s
    }

    pub fn update_and_send(&mut self) {
        let dt = self.delta_time();
        self.time += dt;

        self.update_hands();

        if self.eme_arrangement.is_empty() {
            self.eme_arrangement = String::from(DEFAULT_EME_ARRANGEMENT_NAME);
        }

        if !self.eme_is_playing {
            self.eme_is_playing = true;
        }

        let mut attachments = self.cc_attachments.borrow_mut();

        for (idx, attachment) in attachments.iter_mut() {
            if !attachment.is_active_for(&self.state) {
                continue;
            }

            self.mark_cc_idx(*idx);

            let mut borrow = self.midi_bank.borrow_mut();
            let cc = borrow.get_cc_mut(idx);
            attachment.callback(&self.get_signficant_data(), &mut cc.value, dt);
        }

        drop(attachments);

        self.send_updated_midi_messages();
        self.send_eme_message();

        if self.time > 1.0 {
            println!("transmitted {} bytes of MIDI data", self.midi_bytes);
            self.midi_bytes = 0;
            self.time -= 1.0;
        }
    }

    pub fn reset_delta_time(&mut self) {
        self.computed_delta_time = false;
    }

    pub fn mark_active_midi_ccs_for_update(&self) {
        for channel in 0..NUM_MIDI_CHANNELS {
            for cc in 0..NUM_MIDI_CCS {
                let idx = MIDICCIndex { channel, cc };

                if self.cc_attachments.borrow().contains_key(&idx) {
                    self.updated_cc_indices.borrow_mut().insert(idx);
                    self.midi_bank.borrow_mut().force_update_cc_at(idx);
                }
            }
        }
    }

    pub fn mark_all_midi_notes_as_off(&mut self) {
        for channel in 0..NUM_MIDI_CHANNELS {
            for note in 0..NUM_MIDI_NOTES {
                let idx = MIDINoteIndex { channel, note };
                self.midi_bank.borrow_mut().get_note_mut(&idx).note_on = false;

                self.updated_note_indices.insert(idx);
            }
        }
    }

    // fn note_mut(
    //     &mut self,
    //     channel: usize,
    //     note: usize,
    // ) -> &mut MIDINoteParameter {
    //     let idx = MIDINoteIndex { channel, note };
    //     self.mark_note_idx(idx);
    //
    //     self.midi_bank.get_note_mut(&idx)
    // }

    fn update_hands(&mut self) {
        if self.gesture_data.updated() {
            self.hands = *self.gesture_data.read();
        }

        self.hand_velocities.0 = if let Some(curr) = self.hands.com.first
            && let Some(prev) = self.prev_com.first
        {
            let dist = f64::abs(curr.distance(prev));
            f64::clamp(dist / MAX_HAND_VELOCITY, 0.0, 1.0) as f32
        }
        else {
            0.0
        };

        self.hand_velocities.1 = if let Some(curr) = self.hands.com.second
            && let Some(prev) = self.prev_com.second
        {
            let dist = f64::abs(curr.distance(prev));
            f64::clamp(dist / MAX_HAND_VELOCITY, 0.0, 1.0) as f32
        }
        else {
            0.0
        };

        println!(
            "hand velocities: ({:.3}, {:.3})",
            self.hand_velocities.0, self.hand_velocities.1
        );

        self.prev_com = self.hands.com;
    }

    const fn get_signficant_data(&self) -> SignificantHandValues {
        SignificantHandValues {
            hands: &self.hands,
            velocities: &self.hand_velocities,
        }
    }

    fn send_updated_midi_messages(&mut self) {
        if let Ok(mut sender) = self.senders.midi_sender.lock() {
            if sender.is_full() {
                return;
            }

            let mut buf = Vec::with_capacity(
                self.updated_cc_indices.borrow().len()
                    + self.updated_note_indices.len(),
            );

            let mut bytes = 0;

            let mut bank = self.midi_bank.borrow_mut();
            let threshold_values = bank.get_ccs_outside_of_threshold();
            drop(bank);

            let mut clear = HashSet::new();

            for idx in self.updated_cc_indices.borrow().iter() {
                if !threshold_values.contains(idx) {
                    continue;
                }

                if bytes > MAX_MIDI_BUFFER_SIZE_BYTES {
                    break;
                }

                let mut bank = self.midi_bank.borrow_mut();
                bank.cache_cc(idx);

                let msg = bank.get_cc(idx).to_midi_message(idx.channel as u8);

                bytes += msg.size_bytes();

                buf.push(msg);

                clear.insert(*idx);
            }

            for idx in clear {
                self.updated_cc_indices.borrow_mut().remove(&idx);
            }

            let mut clear = HashSet::new();

            for idx in &self.updated_note_indices {
                if bytes > MAX_MIDI_BUFFER_SIZE_BYTES {
                    break;
                }

                let msg = self
                    .midi_bank
                    .borrow()
                    .get_note(idx)
                    .to_midi_message(idx.channel as u8);

                bytes += msg.size_bytes();

                buf.push(msg);

                clear.insert(*idx);
            }

            self.midi_bytes += bytes;

            for idx in clear {
                self.updated_note_indices.borrow_mut().remove(&idx);
            }

            if !buf.is_empty() {
                if let Err(e) = sender.try_send(buf) {
                    eprintln!("failed to send midi data buffer: {e}");
                }
            }
        }
    }

    fn send_eme_message(&mut self) {
        if let Ok(mut sender) = self.senders.eme_sender.lock() {
            if sender.is_full() {
                return;
            }

            let mut request = EMERequest::new();

            if self.eme_arrangement != self.eme_arrangement_prev {
                request.arrangement = Some(self.eme_arrangement.clone());
                self.eme_arrangement_prev = self.eme_arrangement.clone();
            }

            if (self.eme_is_playing != self.eme_is_playing_prev) {
                request.playback = Some(if self.eme_is_playing {
                    EMEPlayback::Start
                }
                else {
                    EMEPlayback::Stop
                });

                self.eme_is_playing_prev = self.eme_is_playing;
            }

            if let Some(com) = self.hands.com.first {
                self.curr_eme_pos.x = scale_f32(com.x as f32, -1.0, 1.0);
                self.curr_eme_pos.y = 1.0 - (com.y as f32);

                request.position = Some(EMEPosition::new(
                    self.curr_eme_pos.x, self.curr_eme_pos.y,
                ));
            }
            else {
                return;
            }

            if let Err(e) = sender.try_send(request) {
                eprintln!("failed to send EME request: {e}");
            }
        }
    }

    fn mark_cc_idx(&self, idx: MIDICCIndex) {
        self.updated_cc_indices.borrow_mut().insert(idx);
    }

    fn mark_note_idx(&mut self, idx: MIDINoteIndex) {
        self.updated_note_indices.insert(idx);
    }

    fn delta_time(&mut self) -> f32 {
        let dt = if self.computed_delta_time {
            self.time_tracker.elapsed().as_secs_f32()
        }
        else {
            self.computed_delta_time = true;
            0.0
        };

        self.time_tracker = std::time::Instant::now();

        dt
    }

    fn set_ccs_from_attachments(&self) {
        let mut borrow = self.midi_bank.borrow_mut();

        let attachments = self.cc_attachments.borrow();

        for (idx, attachment) in attachments.iter() {
            let cc = borrow.get_cc_mut(idx);
            cc.is_14_bit = attachment.is_14_bit();
            cc.update_threshold = attachment.update_threshold();
        }
    }
}
