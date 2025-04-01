//! Contextual audio data.

use super::*;
use crate::app::audio::VoiceEvent;
use std::sync::{mpsc::Receiver, Arc};

/// TODO this is a bit of a weird intermediate struct used for building
/// and holding data, which could could be extracted elsewhere...
#[derive(Debug)]
pub struct AudioContext {
    // pub note_handler: NoteHandlerRef,
    pub note_channel_receiver: Receiver<NoteEvent>,
    pub sample_rate: f64,
    pub spectral_mask_output: Option<triple_buffer::Output<SpectralMask>>,
    pub reso_bank_data_output: Option<triple_buffer::Output<ResoBankData>>,
    pub voice_event_sender: Sender<VoiceEvent>,
    pub voice_event_receiver: Option<Receiver<VoiceEvent>>,
}
