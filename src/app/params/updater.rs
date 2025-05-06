#![allow(clippy::similar_names)]
use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
    time::Instant,
};

use std::f32::consts::TAU;

use crate::app::{args::Arguments, hands::hand_types::Finger};

use super::*;
use attachment::*;
use eme_request::{EMEPlayback, EMEPosition};
use hands::{
    hand_types::{CCUpdateData, COMPair},
    MAX_HAND_VELOCITY, VELOCITY_MAPPING_TENSION, VELOCITY_THRESHOLD,
};
use midi_cc_attachments::build_midi_cc_attachments;
use midi_types::*;
use rand::seq::IndexedRandom;
use state::ParameterState;

use std::cell::RefCell;

const VELOCITY_EPSILON: f64 = 0.001;

const PINCH_ENTER_THRESHOLD: f64 = 0.85;
const PINCH_RELEASE_THRESHOLD: f64 = 0.60;
const MODE_PINCH_RANGE_FROM_TOP: f64 = 0.20;
const MODE_PINCH_X_DEADZONE: f64 = 0.35;

const MODE_CHANGE_MIDI_NOTE: u8 = 16;

// TODO: change this to be appropriate
const PINCH_TIME_GOAL_SECS: f64 = 1.0;

const MIN_MODE_UPDATE_TIME: f64 = 20.0;
const MAX_MODE_UPDATE_TIME: f64 = 45.0;
const SWITCH_GESTURE_MODE_UPDATE_TIME: f64 = 0.2;
const SWITCH_GESTURE_COOLDOWN: f64 = MODE_SWEEP_TIME * 1.0;

const MODE_SWEEP_TIME: f64 = 1.0;

fn velocity_map(input: f64, tension: f64, threshold: f64) -> f64 {
    let x = input.clamp(0.0, 1.0);
    let t = threshold.clamp(0.0, 1.0);
    let c = tension.clamp(1.0, 20.0);

    if input >= threshold {
        return input;
    }

    let result = t * (x / t).powf(c);

    if result < VELOCITY_EPSILON {
        0.0
    }
    else {
        result
    }
}

fn map_eme_pos(mut pos: Vec2, mode: Mode) -> Vec2 {
    let rect = mode.eme_bounds();

    pos.x = map_f32(pos.x, 0.0, 1.0, rect.x(), rect.x() + rect.w());
    pos.y = map_f32(pos.y, 1.0, 0.0, rect.y(), rect.y() + rect.h());

    pos
}

fn get_random_other(mut mode: Mode) -> Mode {
    let curr = mode;
    mode = Mode::random();

    while (mode == curr) {
        mode = Mode::random();
    }

    mode
}

fn get_random_mode_other_than(mut mode: Mode, previous_mode: Mode) -> Mode {
    let init = mode;
    mode = Mode::random();

    while (mode == init || mode == previous_mode) {
        mode = Mode::random();
    }

    mode
}

#[allow(clippy::struct_excessive_bools)]
pub(super) struct ParameterUpdater {
    senders: ParameterSenders,

    midi_bank: RefCell<MIDIParameterBank>,

    updated_cc_indices: RefCell<HashSet<MIDICCIndex>>,
    updated_note_indices: HashSet<MIDINoteIndex>,

    gesture_data: triple_buffer::Output<RawHandPairCOM>,

    hands: RawHandPairCOM,
    prev_com: COMPair,
    state: ParameterState,

    velocity_time_point: Instant,
    hand_velocities: (f32, f32),
    curr_eme_pos: Vec2,

    previous_mode: Mode,
    mode: Mode,

    eme_is_playing: bool,
    eme_arrangement: String,

    cc_attachments: RefCell<HashMap<MIDICCIndex, MIDICCAttachment>>,

    time_tracker: Instant,
    computed_delta_time: bool,
    time: f32,

    midi_bytes: usize,

    pinch_start_time: Instant,
    is_pinched: bool,
    pinch_at_edge: bool,

    mode_change_midi_message: Option<MIDIMessage>,
    mode_change_posted: bool,

    mode_change_time: Instant,
    mode_change_time_goal: f64,

    mode_sweep_time: Instant,
    mode_sweep_active: bool,

    switch_gesture_cooldown: Instant,
    switch_gesture_posted: bool,
    switch_gesture_prev: bool,
    switch_gesture_time: Instant,
    switch_gesture_time_goal: f64,

    debug_mode: bool,
    print_updates: bool,
    auto_change_mode: bool,
}

impl ParameterUpdater {
    pub fn new(
        senders: ParameterSenders,
        gesture_data: triple_buffer::Output<RawHandPairCOM>,
        args: &Arguments,
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

            velocity_time_point: Instant::now(),
            hand_velocities: (0.0, 0.0),

            curr_eme_pos: vec2(0.0, 0.5),

            previous_mode: Mode::default(),
            mode: Mode::default(),

            eme_is_playing: false,

            eme_arrangement: String::new(),

            cc_attachments: RefCell::new(build_midi_cc_attachments()),

            time_tracker: Instant::now(),
            computed_delta_time: false,

            time: 0.0,
            midi_bytes: 0,

            pinch_start_time: Instant::now(),
            is_pinched: false,
            pinch_at_edge: false,

            mode_change_midi_message: None,
            mode_change_posted: false,

            mode_change_time: Instant::now(),
            mode_change_time_goal: MAX_MODE_UPDATE_TIME,

            mode_sweep_time: Instant::now(),
            mode_sweep_active: false,

            switch_gesture_cooldown: Instant::now(),
            switch_gesture_posted: false,
            switch_gesture_prev: false,
            switch_gesture_time: Instant::now(),
            switch_gesture_time_goal: SWITCH_GESTURE_MODE_UPDATE_TIME,

            debug_mode: args.debug,
            auto_change_mode: args.auto_change_mode,
            print_updates: args.print,
        };

        s.set_ccs_from_attachments();

        s
    }

    pub fn update_and_send(&mut self) {
        let dt = self.delta_time();
        self.time += dt;

        self.try_queue_mode_change_note_off();

        if self.auto_change_mode
            && self.mode_change_time.elapsed().as_secs_f64()
                >= self.mode_change_time_goal
        {
            self.start_mode_change();
        }

        if self.mode_sweep_active {
            const HALF_SWEEP_TIME: f64 = MODE_SWEEP_TIME * 0.5;
            let elapsed = self.mode_sweep_time.elapsed().as_secs_f64();

            if elapsed >= HALF_SWEEP_TIME
                && self.mode_change_midi_message.is_none()
                && !self.mode_change_posted
            {
                self.switch_mode();
                self.mode_change_posted = true;

                println!("mode change was posted");
            }

            if elapsed >= MODE_SWEEP_TIME {
                self.mode_sweep_active = false;
                self.mode_change_posted = false;
            }
        }

        self.update_hands(dt);
        self.update_gestures();

        let mut attachments = self.cc_attachments.borrow_mut();

        for (idx, attachment) in attachments.iter_mut() {
            if !attachment.is_active_for(&self.state) {
                continue;
            }

            self.mark_cc_as_updated(*idx);

            let mut borrow = self.midi_bank.borrow_mut();
            let cc = borrow.get_cc_mut(idx);
            attachment.callback(&self.get_cc_update_data(), &mut cc.value, dt);
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

    pub fn set_eme_playback(&mut self, is_playing: bool) {
        if self.print_updates || self.debug_mode {
            println!("eme playback: {is_playing}");
        }

        self.eme_is_playing = is_playing;

        if let Ok(mut sender) = self.senders.eme_sender.lock() {
            if sender.is_full() {
                return;
            }

            let mut request = EMERequest::new();

            request.playback = Some(if self.eme_is_playing {
                EMEPlayback::Start
            }
            else {
                EMEPlayback::Stop
            });

            if self.print_updates || self.debug_mode {
                println!("adding eme playback: {:?}", request.playback);
            }

            if let Err(e) = sender.try_send(request) {
                eprintln!("failed to send EME request: {e}");
            }
        }
    }

    pub fn set_eme_arrangement(&mut self, arrangement_name: &str) {
        self.eme_arrangement = String::from(arrangement_name);

        if let Ok(mut sender) = self.senders.eme_sender.lock() {
            if sender.is_full() {
                return;
            }

            let mut request = EMERequest::new();

            request.arrangement = Some(self.eme_arrangement.clone());

            if self.print_updates || self.debug_mode {
                println!("adding eme arrangement \"{}\"", self.eme_arrangement);
            }

            if let Err(e) = sender.try_send(request) {
                eprintln!("failed to send EME request: {e}");
            }
        }
    }

    pub fn start_mode_change(&mut self) {
        self.mode_sweep_time = Instant::now();
        self.mode_sweep_active = true;

        self.set_midi_note(
            MODE_CHANGE_MIDI_NOTE, MIDI_CHANNEL_1, MAX_NOTE_VELOCITY, true,
        );

        if self.print_updates || self.debug_mode {
            println!("started mode sweep");
        }
    }

    fn switch_mode(&mut self) {
        let mode = self.mode;
        self.mode = get_random_mode_other_than(self.mode, self.previous_mode);
        self.previous_mode = mode;

        let note = self.mode.get_midi_note_value();
        let message =
            MIDIMessage::note_on(note, MAX_NOTE_VELOCITY, MIDI_CHANNEL_1);

        self.mode_change_midi_message = Some(message);

        self.set_midi_note(note, MIDI_CHANNEL_1, MAX_NOTE_VELOCITY, true);

        if self.print_updates || self.debug_mode {
            println!("mode set to {:?}", self.mode);
        }

        if self.auto_change_mode {
            self.mode_change_time_goal = if self.debug_mode {
                MIN_MODE_UPDATE_TIME
            }
            else {
                random_range(MIN_MODE_UPDATE_TIME, MAX_MODE_UPDATE_TIME)
            };

            self.mode_change_time = Instant::now();
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

    fn update_hands(&mut self, dt: f32) {
        if !self.gesture_data.updated() {
            return;
        }

        self.hands = *self.gesture_data.read();

        // self.detect_pinch();

        let vel_dt = self.velocity_time_point.elapsed().as_secs_f64();

        self.hand_velocities.0 = if let Some(mut curr) = self.hands.com.first
            && let Some(mut prev) = self.prev_com.first
        {
            let dist =
                f64::abs(dvec2(curr.x, curr.y).distance(dvec2(prev.x, prev.y)));
            let speed = dist / vel_dt;
            let normalized = (speed / MAX_HAND_VELOCITY).clamp(0.0, 1.0);

            velocity_map(
                normalized, VELOCITY_MAPPING_TENSION, VELOCITY_THRESHOLD,
            ) as f32
        }
        else {
            0.0
        };

        // println!("velocity at {:?}", self.hand_velocities.0);

        // NOTE(jamie): second hand is unused for now
        // self.hand_velocities.1 = if let Some(mut curr) =
        // self.hands.com.second     && let Some(mut prev) =
        // self.prev_com.second {
        //     let dist =
        //         f64::abs(dvec2(curr.x, curr.y).distance(dvec2(prev.x,
        // prev.y)));     let speed = dist / vel_dt;
        //     let normalized = (speed / MAX_HAND_VELOCITY).clamp(0.0, 1.0);
        //
        //     velocity_map(
        //         normalized, VELOCITY_MAPPING_TENSION, VELOCITY_THRESHOLD,
        //     ) as f32
        // }
        // else {
        //     0.0
        // };

        self.velocity_time_point = Instant::now();

        self.prev_com = self.hands.com;
    }

    fn get_cc_update_data(&self) -> CCUpdateData {
        CCUpdateData {
            hands: &self.hands,
            velocities: &self.hand_velocities,
            mode_sweep: self.mode_sweep_active.then_some(
                self.mode_sweep_time.elapsed().as_secs_f64() / MODE_SWEEP_TIME,
            ),
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

            // *** *** *** *** *** //

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

            for idx in clear {
                self.updated_note_indices.remove(&idx);
            }

            // *** *** *** *** *** //

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

            // *** *** *** *** *** //

            if !buf.is_empty() {
                if let Err(e) = sender.try_send(buf) {
                    eprintln!("failed to send midi data buffer: {e}");
                }
            }

            self.midi_bytes += bytes;

            // if self.debug_mode {
            //     println!("sent {bytes} bytes of MIDI data for broadcast");
            // }
        }
    }

    fn send_eme_message(&mut self) {
        if let Ok(mut sender) = self.senders.eme_sender.lock() {
            if sender.is_full() {
                return;
            }

            let mut request = EMERequest::new();

            if let Some(com) = self.hands.com.first {
                let v2 = vec2(com.x as f32, com.y as f32);
                self.curr_eme_pos = map_eme_pos(v2, self.mode);

                request.position = Some(EMEPosition::new(
                    self.curr_eme_pos.x, self.curr_eme_pos.y,
                ));
            }
            else {
                return;
            }

            // if self.debug_mode {
            //     println!("sending EME request {request:?} for broadcast");
            // }

            if !request.is_empty()
                && let Err(e) = sender.try_send(request)
            {
                eprintln!("failed to send EME request: {e}");
            }
        }
    }

    fn mark_cc_as_updated(&self, idx: MIDICCIndex) {
        self.updated_cc_indices.borrow_mut().insert(idx);
    }

    fn mark_note_as_updated(&mut self, idx: MIDINoteIndex) {
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

        self.time_tracker = Instant::now();

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

    fn detect_pinch(&mut self) {
        let mut first_pinch = false;
        let mut second_pinch = false;

        if let Some(hands) = &self.hands.pair.first {
            let pinch = hands.get_pinch_for(Finger::Index);

            if !self.is_pinched {
                if pinch >= PINCH_ENTER_THRESHOLD {
                    self.is_pinched = true;
                    self.on_pinch_start(true);

                    first_pinch = true;
                }
            }
            else if pinch >= PINCH_RELEASE_THRESHOLD {
                first_pinch = true;
            }
        }

        if let Some(hands) = &self.hands.pair.second {
            let pinch = hands.get_pinch_for(Finger::Index);

            if !self.is_pinched {
                if pinch >= PINCH_ENTER_THRESHOLD {
                    self.is_pinched = true;
                    self.on_pinch_start(false);

                    second_pinch = true;
                }
            }
            else if pinch >= PINCH_RELEASE_THRESHOLD {
                second_pinch = true;
            }
        }

        let any_pinched = first_pinch || second_pinch;

        if !any_pinched && self.is_pinched {
            self.is_pinched = false;
            self.on_pinch_end();
        }
    }

    fn on_pinch_start(&mut self, first_hand: bool) {
        self.pinch_start_time = Instant::now();

        let hand_pos = if first_hand {
            unsafe { self.hands.com.first.unwrap_unchecked() }
        }
        else {
            unsafe { self.hands.com.second.unwrap_unchecked() }
        };

        let in_x_livezone = hand_pos.x >= MODE_PINCH_X_DEADZONE
            && hand_pos.x <= (1.0 - MODE_PINCH_X_DEADZONE);
        let in_y_livezone = hand_pos.y <= MODE_PINCH_RANGE_FROM_TOP;

        let in_pinch_zone = in_x_livezone && in_y_livezone;

        if !self.pinch_at_edge && in_pinch_zone {
            if self.print_updates || self.debug_mode {
                println!("detected edge pinch");
            }

            self.pinch_at_edge = true;
        }
    }

    fn on_pinch_end(&mut self) {
        if self.pinch_at_edge {
            self.pinch_at_edge = false;

            let goal = if self.debug_mode { 0.0 } else { PINCH_TIME_GOAL_SECS };

            if self.pinch_start_time.elapsed().as_secs_f64() >= goal {
                self.start_mode_change();
            }
        }
    }

    fn set_midi_note(
        &mut self,
        note: u8,
        channel: u8,
        velocity: u8,
        note_on: bool,
    ) {
        let idx =
            MIDINoteIndex { channel: channel as usize, note: note as usize };

        let normalized_velocity = velocity as f32 / MAX_NOTE_VELOCITY as f32;

        let mut borrow = self.midi_bank.borrow_mut();
        let note_ref = borrow.get_note_mut(&idx);

        note_ref.note = note;
        note_ref.note_on = note_on;
        note_ref.velocity = normalized_velocity;

        drop(borrow);

        self.mark_note_as_updated(idx);
    }

    fn try_queue_mode_change_note_off(&mut self) {
        if self.mode_change_midi_message.is_none() {
            return;
        }

        let msg = unsafe { self.mode_change_midi_message.unwrap_unchecked() };

        let note = unsafe { msg.note().unwrap_unchecked() };

        let idx = MIDINoteIndex {
            channel: msg.channel() as usize,
            note: note as usize,
        };

        if self.updated_note_indices.contains(&idx) {
            return;
        }

        self.mode_change_midi_message = None;

        self.set_midi_note(note, msg.channel(), MAX_NOTE_VELOCITY, false);
        self.set_midi_note(
            MODE_CHANGE_MIDI_NOTE, MIDI_CHANNEL_1, MAX_NOTE_VELOCITY, false,
        );

        if self.debug_mode {
            println!("sending note-off counterpart for mode change");
        }
    }

    fn update_gestures(&mut self) {
        if self.switch_gesture_cooldown.elapsed().as_secs_f64()
            < SWITCH_GESTURE_COOLDOWN
        {
            return;
        }

        let is_switch_gesture = self
            .hands
            .pair
            .first
            .is_some_and(|hand| hand.gesture.is_thumb_down());

        if is_switch_gesture {
            if self.switch_gesture_prev {
                if !self.switch_gesture_posted
                    && self.switch_gesture_time.elapsed().as_secs_f64()
                        >= SWITCH_GESTURE_MODE_UPDATE_TIME
                {
                    if self.debug_mode {
                        println!("thumb down detected and processed");
                    }

                    self.start_mode_change();
                    self.switch_gesture_posted = true;
                    self.switch_gesture_cooldown = Instant::now();
                }
            }
            else {
                self.switch_gesture_time = Instant::now();
            }
        }
        else {
            self.switch_gesture_posted = false;
        }

        self.switch_gesture_prev = is_switch_gesture;
    }
}
