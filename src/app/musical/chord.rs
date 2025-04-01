//! Musical chord representations. Unused in this project.

use crate::prelude::*;

const MAX: usize = NUM_VOICES as usize;

/// A simple struct for generating the notes of a common chords.
pub struct ChordGen {
    voices: Vec<f64>,
}

impl ChordGen {
    const MAJOR: [f64; 3] = [0.0, 4.0, 7.0];
    const MINOR: [f64; 3] = [0.0, 3.0, 7.0];

    const SUS2: [f64; 3] = [0.0, 2.0, 7.0];
    const SUS4: [f64; 3] = [0.0, 5.0, 7.0];

    const MAJOR7: [f64; 4] = [0.0, 4.0, 7.0, 11.0];
    const MINOR7: [f64; 4] = [0.0, 3.0, 7.0, 10.0];

    const MAJOR9: [f64; 5] = [0.0, 4.0, 7.0, 11.0, 14.0];
    const MINOR9: [f64; 5] = [0.0, 3.0, 7.0, 10.0, 14.0];

    const MAJOR_ADD2: [f64; 4] = [0.0, 2.0, 4.0, 7.0];
    const MINOR_ADD2: [f64; 4] = [0.0, 2.0, 3.0, 7.0];

    const MAJOR_ADD4: [f64; 4] = [0.0, 4.0, 5.0, 7.0];
    const MINOR_ADD4: [f64; 4] = [0.0, 3.0, 5.0, 7.0];

    const MAJOR_ADD9: [f64; 4] = [0.0, 4.0, 7.0, 14.0];
    const MINOR_ADD9: [f64; 4] = [0.0, 3.0, 7.0, 14.0];

    pub fn new() -> Self {
        Self::default()
    }

    /// Generates the notes of a major chord with `root_note` as the root.
    pub fn gen_major(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MAJOR)
    }

    /// Generates the notes of a minor chord with `root_note` as the root.
    pub fn gen_minor(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MINOR)
    }

    /// Generates the notes of a sus2 chord with `root_note` as the root.
    pub fn gen_sus2(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::SUS2)
    }

    /// Generates the notes of a sus4 chord with `root_note` as the root.
    pub fn gen_sus4(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::SUS4)
    }

    /// Generates the notes of a major 7th chord with `root_note` as the root.
    pub fn gen_major7(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MAJOR7)
    }

    /// Generates the notes of a minor 7th chord with `root_note` as the root.
    pub fn gen_minor7(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MINOR7)
    }

    /// Generates the notes of a major 9th chord with `root_note` as the root.
    pub fn gen_major9(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MAJOR9)
    }

    /// Generates the notes of a minor 9th chord with `root_note` as the root.
    pub fn gen_minor9(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MINOR9)
    }

    /// Generates the notes of a major add 2 chord with `root_note` as the root.
    pub fn gen_major_add2(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MAJOR_ADD2)
    }

    /// Generates the notes of a minor add 2 chord with `root_note` as the root.
    pub fn gen_minor_add2(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MINOR_ADD2)
    }

    /// Generates the notes of a major add 4 chord with `root_note` as the root.
    pub fn gen_major_add4(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MAJOR_ADD4)
    }

    /// Generates the notes of a minor add 4 chord with `root_note` as the root.
    pub fn gen_minor_add4(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MINOR_ADD4)
    }

    /// Generates the notes of a major add 9 chord with `root_note` as the root.
    pub fn gen_major_add9(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MAJOR_ADD9)
    }

    /// Generates the notes of a minor add 9 chord with `root_note` as the root.
    pub fn gen_minor_add9(&mut self, root_note: f64) -> &[f64] {
        self.gen(root_note, &Self::MINOR_ADD9)
    }

    /// Generates the notes of a custom chord arrangement.
    ///
    /// Only considers [`NUM_VOICES`] elements of the `chord` slice.
    pub fn gen_custom(&mut self, root_note: f64, chord: &[f64]) -> &[f64] {
        self.gen(
            root_note,
            if chord.len() > MAX {
                &chord[..MAX]
            } else {
                chord
            },
        )
    }

    /// Returns the previously-generated chord.
    pub fn previous_chord(&self) -> &[f64] {
        &self.voices
    }

    /// Inverts (transposes down one octave) a random note of the previously-generated
    /// chord and returns the new inversion.
    pub fn invert_random(&mut self) -> &[f64] {
        let len = self.voices.len();
        // we start at 1 to avoid the root note - wouldn't be an inversion otherwise!
        self.voices[random_range(1, len)] -= 12.0;

        &self.voices
    }

    /// Randomly transposes the notes of the previously-generated chord up octaves.
    /// It is possible that a voice may be transposed up multiple octaves.
    ///
    /// The root note is always retained.
    pub fn spread_voicing(&mut self) -> &[f64] {
        let len = self.voices.len();
        let num_iters = random_range(1, len);

        for _ in 0..num_iters {
            self.voices[random_range(1, len)] += 12.0;
        }

        &self.voices
    }

    /// Offsets each note in the previously-generated chord by a small, random amount.
    ///
    /// `max_variation` controls the maximum amount of variation per note, though is
    /// clamped in the range `0.0` to `0.5`.
    pub fn randomise_pitch(&mut self, mut max_variation: f64) -> &[f64] {
        max_variation = max_variation.clamp(0.0, 0.5);

        for voice in &mut self.voices {
            *voice += scale(random_f64(), -max_variation, max_variation);
        }

        &self.voices
    }

    /// The method for generating values in the internal voice buffer.
    fn gen(&mut self, root_note: f64, chord: &[f64]) -> &[f64] {
        for (i, interval) in chord.iter().enumerate() {
            self.voices[i] = root_note + interval;
        }

        // Safety: chord.len() will never be beyond the allocated
        // capacity of the vector.
        // doing this allows us to track the number of notes in a chord
        // after it is generated.
        unsafe {
            self.voices.set_len(chord.len());
        }

        &self.voices
    }
}

impl Default for ChordGen {
    fn default() -> Self {
        Self {
            voices: vec![0.0; MAX],
        }
    }
}
