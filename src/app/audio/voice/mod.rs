//! Polyphonic voices.

pub mod audio_note;
pub mod voice;

pub use audio_note::{NoteEvent, NoteHandler};
pub use voice::{Voice, VoiceEvent, VoiceHandler};
