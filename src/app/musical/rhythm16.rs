//! Bitwise rhythm representations. Unused in this device.

pub trait BitwiseRhythm16 {
    fn is_beat(&self, idx: usize) -> bool;
}

// it's bitwise time baby

impl BitwiseRhythm16 for u16 {
    fn is_beat(&self, mut idx: usize) -> bool {
        // get the correct right-shift amount
        idx = 15 - (idx % 16);
        // shift the element and compare with 1
        (self >> idx) & 1 == 1
    }
}

impl BitwiseRhythm16 for u64 {
    fn is_beat(&self, mut idx: usize) -> bool {
        // we first check to see how many bars the word covers...
        let mut shift = 16;
        idx = loop {
            // if no bits are 1, then no element is a beat.
            if shift > 64 {
                return false;
            }

            if self >> shift == 0 {
                break (shift - 1) - (idx % shift);
            }

            shift += 16;
        };

        (self >> idx) & 1 == 1
    }
}

impl BitwiseRhythm16 for [u16; 3] {
    fn is_beat(&self, mut idx: usize) -> bool {
        // first, get the index within the bounds of the array
        idx %= 16 * 3;

        // find which element to use
        let element = idx / 16;
        // find the position in that element
        let off = idx % 16;

        // get the correct right-shift amount
        idx = 15 - off;

        // shift the element and compare it with 1
        (self[element] >> idx) & 1 == 1
    }
}

/// A container of common rhythmic patterns represented as 16th notes. Each rhythm
/// is stored as bits, so it is recommended to use `is_beat() (as BitwiseRhythm16)` to
/// query whether a particular interval is a beat or not.
///
/// # Example
/// ```
/// use BitwiseRhythm16;
///
/// // an arbitrary number of intervals you're iterating through...
/// let num_beats = 128;
///
/// // this evaluates all 16th intervals...
/// for i in 0..num_beats {
///     if Rhythm16th::HALF_DOTTED.is_beat(i) {
///         // this is a note in the rhythm!
///     }
/// }
/// ```
pub struct Rhythm16th;

impl Rhythm16th {
    /// 1/1 rhythm.
    pub const WHOLE_NOTE: u16 = 0b1000_0000_0000_0000;
    /// 1/2 rhythm.
    pub const HALF_NOTE: u16 = 0b1000_0000_1000_0000;
    /// 1/4 rhythm.
    pub const QUARTER_NOTE: u16 = 0b1000_1000_1000_1000;
    /// 1/8 rhythm.
    pub const EIGHTH_NOTE: u16 = 0b1010_1010_1010_1010;
    /// 1/16 rhythm.
    pub const SIXTEENTH_NOTE: u16 = 0b1111_1111_1111_1111;

    /// 3/8 rhythm (3 bars so it wraps completely).
    pub const HALF_DOTTED: u64 = 0b1000_0010_0000_1000_0010_0000_1000_0010_0000_1000_0010_0000;
    /// 3/16 rhythm (3 bars so it wraps completely).
    pub const QUARTER_DOTTED: u64 = 0b1001_0010_0100_1001_0010_0100_1001_0010_0100_1001_0010_0100;
    // 0b1001_0010_0100_1001_0010_0100_1001_0010_0100_1001_0010_0100;
}

// TODO
// pub struct Rhythm32;
