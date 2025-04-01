//! Musical note representation.

use nannou::prelude::*;
use std::fmt::Display;

// there is no intention of changing the variants of these enums
// so the wildcard import is fine.
use Note::*;
use Octave::*;

pub fn midi_note_value_from(octave: Octave, note: Note) -> f64 {
    octave.starting_midi_note() + note.note_value()
}

pub fn midi_note_to_string(value: u8) -> String {
    let oct = Octave::from_note(value);
    let note = Note::from_value(value as i32);

    format!("{note}{oct}")
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Octave {
    /// Octave covering C-1 - B-1 (MIDI note range 0 - 11)
    Cneg1,
    /// Octave covering C0 - B0 (MIDI note range 12 - 23)
    C0,
    /// Octave covering C1 - B1 (MIDI note range 24 - 35)
    C1,
    /// Octave covering C2 - B2 (MIDI note range 36 - 47)
    C2,
    /// Octave covering C3 - B3 (MIDI note range 48 - 59)
    #[default]
    C3,
    /// Octave covering C4 - B4 (MIDI note range 60 - 71)
    C4,
    /// Octave covering C5 - B5 (MIDI note range 72 - 83)
    C5,
    /// Octave covering C6 - B6 (MIDI note range 84 - 95)
    C6,
    /// Octave covering C7 - B7 (MIDI note range 96 - 107)
    C7,
    /// Octave covering C8 - B8 (MIDI note range 108 - 119)
    C8,
    /// Octave covering C9 - B9 (MIDI note range 120 - 131)
    C9,
}

impl Display for Octave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cneg1 => write!(f, "-1"),
            C0 => write!(f, "0"),
            C1 => write!(f, "1"),
            C2 => write!(f, "2"),
            C3 => write!(f, "3"),
            C4 => write!(f, "4"),
            C5 => write!(f, "5"),
            C6 => write!(f, "6"),
            C7 => write!(f, "7"),
            C8 => write!(f, "8"),
            C9 => write!(f, "9"),
        }
    }
}

impl Octave {
    /// Returns the value of the starting note of this octave.
    #[must_use]
    pub fn starting_midi_note(&self) -> f64 {
        match self {
            Cneg1 => 0.0,
            C0 => 12.0,
            C1 => 24.0,
            C2 => 36.0,
            C3 => 48.0,
            C4 => 60.0,
            C5 => 72.0,
            C6 => 84.0,
            C7 => 96.0,
            C8 => 108.0,
            C9 => 120.0,
        }
    }

    /// Returns the `Octave` which covers the provided MIDI note.
    ///
    /// # Panics
    ///
    /// Panics if `note` is outside of the range `0` to `132`.
    #[must_use]
    pub fn from_note(note: u8) -> Self {
        match note {
            0..=11 => Cneg1,
            12..=23 => C0,
            24..=35 => C1,
            36..=47 => C2,
            48..=59 => C3,
            60..=71 => C4,
            72..=83 => C5,
            84..=95 => C6,
            96..=107 => C7,
            108..=119 => C8,
            120..=131 => C9,
            _ => panic!(
                "value provided ({note}) is outside of the acceptable range"
            ),
        }
    }

    /// Increases the octave by one. Does not exceed C9.
    pub fn increase(&mut self) {
        *self = match self {
            Cneg1 => C0,
            C0 => C1,
            C1 => C2,
            C2 => C3,
            C3 => C4,
            C4 => C5,
            C5 => C6,
            C6 => C7,
            C7 => C8,
            C8 | C9 => C9,
        };
    }

    /// Increases the octave by `amount`. Does not exceed C9.
    pub fn increase_by(&mut self, amount: i32) {
        for _ in 0..amount {
            self.increase();
        }
    }

    /// Decreases the octave by one. Does not exceed C-1.
    pub fn decrease(&mut self) {
        *self = match self {
            Cneg1 | C0 => Cneg1,
            C1 => C0,
            C2 => C1,
            C3 => C2,
            C4 => C3,
            C5 => C4,
            C6 => C5,
            C7 => C6,
            C8 => C7,
            C9 => C8,
        };
    }

    /// Decreases the octave by `amount`. Does not exceed C-1.
    pub fn decrease_by(&mut self, amount: i32) {
        for _ in 0..amount {
            self.decrease();
        }
    }

    /// Transposes the octave, returning the new octave.
    #[must_use]
    pub fn transpose(&self, num_octaves: i32) -> Self {
        let mut s = *self;
        s.increase_by(num_octaves);
        s
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Note {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            C => write!(f, "C"),
            Cs => write!(f, "C#"),
            D => write!(f, "D"),
            Ds => write!(f, "D#"),
            E => write!(f, "E"),
            F => write!(f, "F"),
            Fs => write!(f, "F#"),
            G => write!(f, "G"),
            Gs => write!(f, "G#"),
            A => write!(f, "A"),
            As => write!(f, "A#"),
            B => write!(f, "B"),
        }
    }
}

impl Note {
    /// Returns the note with a given transposition.
    #[must_use]
    pub fn transpose(&self, semitones: i32) -> Self {
        let mut value = (self.note_value() as i32 + semitones) % 12;
        while value < 0 {
            value += 12;
        }

        Self::from_value(value)
    }

    /// Returns the key associated with a specific key on the keyboard.
    pub fn from_key(key: &Key) -> Option<Self> {
        match key {
            Key::A | Key::K => Some(C),
            Key::W | Key::O => Some(Cs),
            Key::S | Key::L => Some(D),
            Key::E => Some(Ds),
            Key::D => Some(E),
            Key::F => Some(F),
            Key::T => Some(Fs),
            Key::G => Some(G),
            Key::Y => Some(Gs),
            Key::H => Some(A),
            Key::U => Some(As),
            Key::J => Some(B),
            _ => None,
        }
    }

    /// Returns the value of the note for any octave.
    ///
    /// `C` is represented as 0, and `B` as 11.
    pub fn note_value(&self) -> f64 {
        match self {
            C => 0.0,
            Cs => 1.0,
            D => 2.0,
            Ds => 3.0,
            E => 4.0,
            F => 5.0,
            Fs => 6.0,
            G => 7.0,
            Gs => 8.0,
            A => 9.0,
            As => 10.0,
            B => 11.0,
        }
    }

    pub fn key_value(key: &Key) -> Option<f64> {
        if let Some(note) = Self::from_key(key) {
            return Some(note.note_value());
        }

        None
    }

    /// Returns the note associated with the provided MIDI note value.
    ///
    /// # Panics
    ///
    /// Panics if `value` is out of the range `0` to `132`.
    #[must_use]
    pub fn from_value(value: i32) -> Self {
        assert!((0..=132).contains(&value));

        let value = value % 12;
        match value {
            0 => C,
            1 => Cs,
            2 => D,
            3 => Ds,
            4 => E,
            5 => F,
            6 => Fs,
            7 => G,
            8 => Gs,
            9 => A,
            10 => As,
            11 => B,
            _ => unreachable!(),
        }
    }
}

/// All the keyboard values which are used to trigger MIDI note messages.
pub const KEYBOARD_MIDI_NOTES: [Key; 16] = [
    Key::A,
    Key::S,
    Key::D,
    Key::F,
    Key::G,
    Key::H,
    Key::J,
    Key::K,
    Key::L,
    Key::W,
    Key::E,
    Key::T,
    Key::Y,
    Key::U,
    Key::O,
    Key::P,
];

/// The intervals of notes in a major scale for a single octave.
pub const MAJOR_SCALE_INTERVALS: [u32; 7] = [0, 2, 4, 5, 7, 9, 11];
/// The intervals of notes in a minor scale for a single octave.
pub const MINOR_SCALE_INTERVALS: [u32; 7] = [0, 2, 3, 5, 7, 8, 10];
/// The intervals of notes in a major pentatonic scale for a single octave.
pub const MAJOR_PENTATONIC_SCALE_INTERVALS: [u32; 5] = [0, 2, 4, 7, 9];
/// The intervals of notes in a minor pentatonic scale for a single octave.
pub const MINOR_PENTATONIC_SCALE_INTERVALS: [u32; 5] = [0, 3, 5, 7, 10];
