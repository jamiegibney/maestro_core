use nannou::geom::Range;

use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    #[default]
    A,
    B,
    C,
    // D,
}

impl Mode {
    /// Returns a random `Mode`.
    pub fn random() -> Self {
        let r = random::<u32>() % 3;

        match r {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            // 3 => Self::D,
            _ => unreachable!(),
        }
    }

    /// Returns the next `Mode` in sequence.
    pub const fn to_next(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::C,
            Self::C => Self::A,
            // Self::D => Self::A,
        }
    }

    /// Returns the previous `Mode` in sequence.
    pub const fn to_prev(self) -> Self {
        match self {
            Self::A => Self::C,
            Self::B => Self::A,
            Self::C => Self::B,
            // Self::D => Self::C,
        }
    }

    /// Sets the `Mode` to its next value.
    pub fn progress_next(&mut self) {
        *self = (*self).to_next();
    }

    /// Sets the `Mode` to its previous value.
    pub fn progress_prev(&mut self) {
        *self = (*self).to_prev();
    }

    /// Returns the EME XY bounds for the `Mode`.
    pub fn eme_bounds(self) -> Rect<f32> {
        let mut x = -1.0;
        let mut y = 0.0;

        match self {
            // top-left
            Self::A => y = 0.5,
            // top-right
            Self::B => {
                x = 0.0;
                y = 0.5;
            }
            // bottom-left
            Self::C => {}
            // bottom-right
            // Self::D => x = 0.0,
        }

        Rect::from_xy_wh(Point2::new(x, y), Vec2::new(1.0, 0.5))
    }

    pub const fn get_midi_note_value(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            // Self::D => 0,
        }
    }
}
