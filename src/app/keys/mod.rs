use model::MIDISendMode;

use super::*;

pub fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Minus | Key::Underline => {
            if app.keys.mods.shift() && model.midi_send_value >= 10 {
                model.midi_send_value -= 10;
            }
            else if (model.midi_send_value > 0) {
                model.midi_send_value -= 1;
            }
        }
        Key::Plus | Key::Equals => {
            let inc = if app.keys.mods.shift() { 10 } else { 1 };
            model.midi_send_value =
                u8::min(model.midi_send_value + inc, (1 << 7) - 1);
        }
        Key::Comma => {
            if app.keys.mods.shift() && model.midi_send_channel >= 10 {
                model.midi_send_channel -= 10;
            }
            else if (model.midi_send_channel > 0) {
                model.midi_send_channel -= 1;
            }
        }
        Key::Period => {
            let inc = if app.keys.mods.shift() { 10 } else { 1 };
            model.midi_send_channel =
                u8::min(model.midi_send_channel + inc, (1 << 4) - 1);
        }
        Key::N => model.midi_send_mode = MIDISendMode::MIDINote,
        Key::C => model.midi_send_mode = MIDISendMode::MIDIControlChange,
        Key::Return => model.send_midi(),

        Key::T => model.send_and_update(true),
        Key::S => model.send_and_update(false),

        Key::H => model.show_state_data = !model.show_state_data,

        _ => {}
    }
}

pub fn key_released(_app: &App, model: &mut Model, key: Key) {
    //
}
