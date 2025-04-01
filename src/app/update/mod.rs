//! The update callback, for mutating state each frame. Not for drawing.

use midi::message::MIDIMessage;

use super::*;
use std::sync::{Arc, Mutex, RwLock};

/// The app's update callback for updating state.
pub fn update(app: &App, model: &mut Model, update: Update) {
    model.update(&update);
}
