//! The view callback, i.e. "draw loop".

use crate::prelude::xfer::s_curve;
use nannou::geom::{path, Path};

use super::{hands::LIGHT_MODE, *};

/// The app's view callback (AKA "draw loop").
pub fn view(app: &App, model: &Model, frame: Frame) {
    let bg_col = if LIGHT_MODE { WHITE } else { BLACK };
    frame.clear(bg_col);
    let frame = &frame;
    let draw = &app.draw();

    model.hand_manager.damped_hands().draw(draw, frame);
    model.draw(draw, frame);

    _ = draw.to_frame(app, frame);
}
