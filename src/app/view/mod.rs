//! The view callback, i.e. "draw loop".

use crate::prelude::xfer::s_curve;
use nannou::geom::{path, Path};

use super::*;

/// The app's view callback (AKA "draw loop").
pub fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);
    let frame = &frame;
    let draw = &app.draw();

    model.hand_manager.damped_hands().draw(draw, frame);
    model.draw(draw, frame);

    _ = draw.to_frame(app, frame);
}
