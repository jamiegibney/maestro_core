use super::*;

/// Empty "null" byte for consistency.
const EMPTY_BYTE: u8 = 0b_0000_0000;

/// All generic MIDI values must be below this value.
const MAX_7_BIT_INT: u8 = 1 << 7;
/// All MIDI channels must be below this value.
const MAX_4_BIT_INT: u8 = 1 << 4;
/// Controllers from 120-127 are reserved for "Channel Mode Messages", which are
/// special instructions. So only controllers 0-119 are available for general
/// use.
const MAX_CONTROLLER_NUMBER: u8 = 120;

/// This mask should be used for all generic MIDI values.
const GENERIC_MIDI_VALUE_MASK: u8 = 0b_0111_1111;
/// Bit mask for MIDI status information — as part of the status byte.
const STATUS_BIT_MASK: u8 = 0b1111_0000;
/// Bit mask for MIDI channel value — as part of the status byte.
const CHANNEL_BIT_MASK: u8 = 0b0000_1111;

/// Value for MIDI note off messages.
const MIDI_NOTE_OFF: u8 = 0x80;
/// Value for MIDI note on messages.
const MIDI_NOTE_ON: u8 = 0x90;
/// Value for MIDI control change messages.
const MIDI_CONTROL_CHANGE: u8 = 0xB0;
/// Value for MIDI program change messages.
const MIDI_PROGRAM_CHANGE: u8 = 0xC0;

const MAX_14_BIT_CONTROLLER_NUMBER: u8 = 32;
const MAX_14_BIT_INT: u16 = 1 << 14;

/// Representation of a MIDI message. Use `as_bytes()` to get the message as a
/// 3-byte value.
#[derive(Clone, Copy, Debug)]
pub enum MIDIMessage {
    NoteOff { note: u8, velocity: u8, ch: u8 },
    NoteOn { note: u8, velocity: u8, ch: u8 },
    ControlChange { controller: u8, value: u8, ch: u8 },
    ControlChange14Bit { controller: u8, value: u16, ch: u8 },
    ProgramChange { program: u8, ch: u8 },
}

impl MIDIMessage {
    /// Returns a MIDI note off message with the provided information.
    ///
    /// # Panics
    ///
    /// If the provided values are invalid for MIDI messages, this function will
    /// panic.
    pub fn note_off(note_value: u8, note_vel: u8, channel: u8) -> Self {
        assert!(
            note_value < MAX_7_BIT_INT,
            "got invalid note value of {note_value}"
        );
        assert!(
            note_vel < MAX_7_BIT_INT,
            "got invalid note velocity of {note_vel}"
        );
        assert!(
            channel < MAX_4_BIT_INT,
            "got invalid MIDI channel of {channel}"
        );

        Self::NoteOff { note: note_value, velocity: note_vel, ch: channel }
    }

    /// Returns a MIDI note on message with the provided information.
    ///
    /// # Panics
    ///
    /// If the provided values are invalid for MIDI messages, this function will
    /// panic.
    pub fn note_on(note_value: u8, note_vel: u8, channel: u8) -> Self {
        assert!(
            note_value < MAX_7_BIT_INT,
            "got invalid note value of {note_value}"
        );
        assert!(
            note_vel < MAX_7_BIT_INT,
            "got invalid note velocity of {note_vel}"
        );
        assert!(
            channel < MAX_4_BIT_INT,
            "got invalid MIDI channel of {channel}"
        );

        Self::NoteOn { note: note_value, velocity: note_vel, ch: channel }
    }

    /// Returns a MIDI CC message with the controller number and value.
    ///
    /// # Panics
    ///
    /// If the provided values are invalid for MIDI messages, this function will
    /// panic.
    pub fn control_change(
        controller_number: u8,
        controller_value: u8,
        channel: u8,
    ) -> Self {
        assert!(
            controller_number < MAX_CONTROLLER_NUMBER,
            "got invalid controller number of {controller_number}"
        );
        assert!(
            controller_value < MAX_7_BIT_INT,
            "got invalid controller value of {controller_value}"
        );
        assert!(
            channel < MAX_4_BIT_INT,
            "got invalid MIDI channel of {channel}"
        );

        Self::ControlChange {
            controller: controller_number,
            value: controller_value,
            ch: channel,
        }
    }

    /// Returns a MIDI CC message with the controller number and value.
    ///
    /// # Panics
    ///
    /// If the provided values are invalid for MIDI messages, this function will
    /// panic.
    pub fn control_change_14_bit(
        controller_number: u8,
        controller_value: u16,
        channel: u8,
    ) -> Self {
        assert!(
            controller_number < MAX_14_BIT_CONTROLLER_NUMBER,
            "got invalid controller number of {controller_number}"
        );
        assert!(
            controller_value < MAX_14_BIT_INT,
            "got invalid controller value of {controller_value}"
        );
        assert!(
            channel < MAX_4_BIT_INT,
            "got invalid MIDI channel of {channel}"
        );

        Self::ControlChange14Bit {
            controller: controller_number,
            value: controller_value,
            ch: channel,
        }
    }
    /// Returns a MIDI program change message with the provided program number.
    ///
    /// # Panics
    ///
    /// If the provided values are invalid for MIDI messages, this function will
    /// panic.
    pub fn program_change(program_number: u8, channel: u8) -> Self {
        assert!(
            program_number < MAX_7_BIT_INT,
            "got invalid program number of {program_number}"
        );
        assert!(
            channel < MAX_4_BIT_INT,
            "got invalid MIDI channel of {channel}"
        );

        Self::ProgramChange { program: program_number, ch: channel }
    }

    pub const fn channel(self) -> u8 {
        match self {
            Self::NoteOff { ch, .. }
            | Self::NoteOn { ch, .. }
            | Self::ControlChange { ch, .. }
            | Self::ControlChange14Bit { ch, .. }
            | Self::ProgramChange { ch, .. } => ch,
        }
    }

    pub const fn note(self) -> Option<u8> {
        match self {
            Self::NoteOff { note, .. } | Self::NoteOn { note, .. } => {
                Some(note)
            }
            _ => None,
        }
    }

    /// Whether the MIDI message is a 14-bit CC message.
    pub const fn is_14_bit(self) -> bool {
        matches!(self, Self::ControlChange14Bit { .. })
    }

    pub const fn size_bytes(self) -> usize {
        if self.is_14_bit() {
            6
        }
        else {
            3
        }
    }

    /// Returns the MIDI message as a 3-byte array. This method guarantees that
    /// the provided bytes are valid for MIDI messages.
    ///
    /// # Panics
    ///
    /// This method will panic if the MIDI message is a 14-bit CC message. Query
    /// this with [`MIDIMessage::is_14_bit()`], and use
    /// [`MIDIMessage::as_bytes_double()`] to obtain the bytes.
    pub fn as_bytes(mut self) -> [u8; 3] {
        assert!(
            !self.is_14_bit(),
            "cannot convert 14-bit midi CC to 3-byte message"
        );

        self.sanitize();

        let status_byte = self.to_status_byte();

        let mut data = [EMPTY_BYTE; 2];

        match self {
            Self::NoteOff { note, velocity, .. }
            | Self::NoteOn { note, velocity, .. } => {
                data[0] = note;
                data[1] = velocity;
            }
            Self::ControlChange { controller, value, .. } => {
                data[0] = controller;
                data[1] = value;
            }
            Self::ProgramChange { program, .. } => {
                data[0] = program;
            }
            Self::ControlChange14Bit { .. } => {}
        }

        [status_byte, data[0], data[1]]
    }

    pub fn as_bytes_double(mut self) -> [u8; 6] {
        const LSB_MASK: u16 = GENERIC_MIDI_VALUE_MASK as u16;
        const MSB_MASK: u16 = LSB_MASK << 7;

        if let Self::ControlChange14Bit { controller, value, ch } = self {
            let msb = ((value & MSB_MASK) >> 7) as u8;
            let lsb = (value & LSB_MASK) as u8;

            let msb_cc = controller;
            let lsb_cc = controller + MAX_14_BIT_CONTROLLER_NUMBER;

            let first = Self::control_change(msb_cc, msb, ch);
            let second = Self::control_change(lsb_cc, lsb, ch);

            let first_bytes = first.as_bytes();
            let second_bytes = second.as_bytes();

            [
                second_bytes[0], second_bytes[1], second_bytes[2],
                first_bytes[0], first_bytes[1], first_bytes[2],
            ]
        }
        else {
            panic!("cannot convert non-14-bit midi CC to 3-byte message");
        }
    }

    pub const fn as_inverse_note_message(self) -> Self {
        match self {
            Self::NoteOff { note, velocity, ch } => {
                Self::NoteOn { note, velocity, ch }
            }
            Self::NoteOn { note, velocity, ch } => {
                Self::NoteOff { note, velocity, ch }
            }
            _ => self,
        }
    }

    fn sanitize(&mut self) {
        match self {
            Self::NoteOff { note, velocity, ch }
            | Self::NoteOn { note, velocity, ch } => {
                *note &= GENERIC_MIDI_VALUE_MASK;
                *velocity &= GENERIC_MIDI_VALUE_MASK;
                *ch &= CHANNEL_BIT_MASK;
            }
            Self::ControlChange { controller, value, ch } => {
                *controller &= GENERIC_MIDI_VALUE_MASK;
                *value &= GENERIC_MIDI_VALUE_MASK;
                *ch &= CHANNEL_BIT_MASK;
            }
            Self::ProgramChange { program, ch } => {
                *program &= GENERIC_MIDI_VALUE_MASK;
                *ch &= CHANNEL_BIT_MASK;
            }
            Self::ControlChange14Bit { .. } => {}
        }
    }

    const fn to_status_byte(self) -> u8 {
        let channel = match self {
            Self::NoteOff { ch, .. }
            | Self::NoteOn { ch, .. }
            | Self::ControlChange { ch, .. }
            | Self::ControlChange14Bit { ch, .. }
            | Self::ProgramChange { ch, .. } => ch,
        } & CHANNEL_BIT_MASK;

        let status = match self {
            Self::NoteOff { .. } => MIDI_NOTE_OFF,
            Self::NoteOn { .. } => MIDI_NOTE_ON,
            Self::ControlChange { .. } => MIDI_CONTROL_CHANGE,
            Self::ControlChange14Bit { .. } => MIDI_CONTROL_CHANGE,
            Self::ProgramChange { .. } => MIDI_PROGRAM_CHANGE,
        } & STATUS_BIT_MASK;

        channel | status
    }
}

impl std::fmt::Display for MIDIMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoteOff { note, velocity, ch } => {
                write!(
                    f,
                    "MIDI note #{} OFF with velocity {} (channel {})",
                    note,
                    velocity,
                    ch + 1
                )
            }
            Self::NoteOn { note, velocity, ch } => {
                write!(
                    f,
                    "MIDI note #{} ON with velocity {} (channel {})",
                    note,
                    velocity,
                    ch + 1
                )
            }
            Self::ControlChange { controller, value, ch } => {
                write!(
                    f,
                    "MIDI CC #{} with value {} (channel {})",
                    controller,
                    value,
                    ch + 1
                )
            }
            Self::ControlChange14Bit { controller, value, ch } => {
                write!(
                    f,
                    "MIDI CC 14-bit #{} with value {} (channel {})",
                    controller,
                    value,
                    ch + 1
                )
            }
            Self::ProgramChange { program, ch } => {
                write!(
                    f,
                    "MIDI program change to #{} (channel {})",
                    program,
                    ch + 1
                )
            }
        }
    }
}
