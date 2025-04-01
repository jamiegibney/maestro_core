use std::collections::HashSet;

use super::*;

pub(super) const NUM_MIDI_CHANNELS: usize = 16;
pub(super) const NUM_MIDI_CCS: usize = 119;
pub(super) const NUM_MIDI_NOTES: usize = 128;

fn f32_to_7bit(val: f32) -> u8 {
    scale_f32(val.clamp(0.0, 1.0), 0.0, 127.0).round() as u8
}

fn f32_to_14bit(val: f32) -> u16 {
    scale_f32(val.clamp(0.0, 1.0), 0.0, 16383.0).round() as u16
}

fn f64_to_7bit(val: f64) -> u8 {
    scale(val.clamp(0.0, 1.0), 0.0, 127.0).round() as u8
}

fn f64_to_14bit(val: f64) -> u16 {
    scale(val.clamp(0.0, 1.0), 0.0, 16383.0).round() as u16
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MIDICCParameter {
    pub cc: u8,
    pub value: f32,
    pub is_14_bit: bool,
    pub update_threshold: f32,
}

impl MIDICCParameter {
    pub fn to_midi_message(self, channel: u8) -> MIDIMessage {
        if self.is_14_bit {
            MIDIMessage::control_change_14_bit(
                self.cc,
                f32_to_14bit(self.value),
                channel,
            )
        }
        else {
            MIDIMessage::control_change(
                self.cc,
                f32_to_7bit(self.value),
                channel,
            )
        }
    }
}

// *** *** *** //

pub trait MIDIParameterIndex {
    fn get_channel(&self) -> usize;
    fn get_number(&self) -> usize;
}

// *** *** *** //

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MIDINoteParameter {
    pub note: u8,
    pub note_on: bool,
    pub velocity: f32,
}

impl MIDINoteParameter {
    pub fn to_midi_message(self, channel: u8) -> MIDIMessage {
        if self.note_on {
            MIDIMessage::note_on(self.note, f32_to_7bit(self.velocity), channel)
        }
        else {
            MIDIMessage::note_off(
                self.note,
                f32_to_7bit(self.velocity),
                channel,
            )
        }
    }
}

// *** *** *** //

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct MIDINoteIndex {
    pub(super) channel: usize,
    pub(super) note: usize,
}

impl MIDINoteIndex {
    pub fn new(channel: u8, note: u8) -> Self {
        let channel = channel as usize;
        let note = note as usize;

        assert!(
            channel < NUM_CHANNELS,
            "exceeded number of channels (got {channel})"
        );
        assert!(note < NUM_MIDI_NOTES, "exceeded number of notes (got {note})");

        Self { channel, note }
    }
}

impl MIDIParameterIndex for MIDINoteIndex {
    fn get_channel(&self) -> usize {
        self.channel
    }

    fn get_number(&self) -> usize {
        self.note
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct MIDICCIndex {
    pub(super) channel: usize,
    pub(super) cc: usize,
}

impl MIDICCIndex {
    pub fn new(channel: u8, cc: u8) -> Self {
        let channel = channel as usize;
        let cc = cc as usize;

        assert!(
            channel < NUM_CHANNELS,
            "exceeded number of channels (got {channel})"
        );
        assert!(cc < NUM_MIDI_CCS, "exceeded number of ccs (got {cc})");

        Self { channel, cc }
    }
}

impl MIDIParameterIndex for MIDICCIndex {
    fn get_channel(&self) -> usize {
        self.channel
    }

    fn get_number(&self) -> usize {
        self.cc
    }
}

// *** *** *** //

pub struct MIDIParameterBank {
    pub cc_params: Box<[[MIDICCParameter; NUM_MIDI_CCS]]>,
    pub note_params: Box<[[MIDINoteParameter; NUM_MIDI_NOTES]]>,

    cc_params_cache: Box<[[MIDICCParameter; NUM_MIDI_CCS]]>,
    force_update_ccs: HashSet<MIDICCIndex>,
}

impl MIDIParameterBank {
    pub fn new() -> Self {
        let cc_inner: [MIDICCParameter; NUM_MIDI_CCS] =
            std::array::from_fn(|i| MIDICCParameter {
                cc: i as u8,
                value: 0.0,
                is_14_bit: false,
                update_threshold: DEFAULT_MIDI_CC_UPDATE_THRESHOLD,
            });

        let note_inner: [MIDINoteParameter; NUM_MIDI_NOTES] =
            std::array::from_fn(|i| MIDINoteParameter {
                note: i as u8,
                note_on: false,
                velocity: 0.0,
            });

        Self {
            cc_params: vec![cc_inner; NUM_MIDI_CHANNELS].into_boxed_slice(),
            note_params: vec![note_inner; NUM_MIDI_CHANNELS].into_boxed_slice(),

            cc_params_cache: vec![cc_inner; NUM_MIDI_CHANNELS]
                .into_boxed_slice(),
            force_update_ccs: HashSet::new(),
        }
    }

    pub const fn get_cc(&self, idx: &MIDICCIndex) -> &MIDICCParameter {
        &self.cc_params[idx.channel][idx.cc]
    }

    pub fn get_cc_mut(&mut self, idx: &MIDICCIndex) -> &mut MIDICCParameter {
        &mut self.cc_params[idx.channel][idx.cc]
    }

    pub const fn get_note(&self, idx: &MIDINoteIndex) -> &MIDINoteParameter {
        &self.note_params[idx.channel][idx.note]
    }

    pub fn get_note_mut(
        &mut self,
        idx: &MIDINoteIndex,
    ) -> &mut MIDINoteParameter {
        &mut self.note_params[idx.channel][idx.note]
    }

    pub fn force_update_cc_at(&mut self, idx: MIDICCIndex) {
        self.force_update_ccs.insert(idx);
    }

    pub fn get_ccs_outside_of_threshold(&self) -> HashSet<MIDICCIndex> {
        let mut result = HashSet::new();

        for ch in 0..NUM_MIDI_CHANNELS {
            for cc in 0..NUM_MIDI_CCS {
                let update_threshold = self.cc_params[ch][cc].update_threshold;
                let curr_val = self.cc_params[ch][cc].value;
                let cached_val = self.cc_params_cache[ch][cc].value;

                let idx = MIDICCIndex { channel: ch, cc };

                let force = self.force_update_ccs.contains(&idx);
                let outside_of_threshold =
                    (curr_val - cached_val).abs() > update_threshold;

                if force || outside_of_threshold {
                    result.insert(idx);
                }
            }
        }

        result
    }

    pub fn cache_cc(&mut self, idx: &MIDICCIndex) {
        self.force_update_ccs.remove(idx);

        let ch = idx.channel;
        let cc = idx.cc;

        self.cc_params_cache[ch][cc] = self.cc_params[ch][cc];
    }
}
