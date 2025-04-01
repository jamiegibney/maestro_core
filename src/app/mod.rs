//! All app-related state and logic.

use crate::prelude::*;
use nannou::prelude::*;
use nannou::LoopMode::RefreshSync;
use nannou_audio;

pub mod args;
pub mod audio;
pub mod hands;
pub mod keys;
pub mod midi;
mod model;
pub mod musical;
pub mod osc;
pub mod params;
pub mod update;
pub mod view;

pub use model::Model;
pub use musical::*;
pub use params::*;
use update::update;

/// Runs the app via Nannou.
pub fn run_app() {
    nannou::app(model::Model::build)
        .loop_mode(RefreshSync)
        .update(update)
        .run();
}

pub trait Updatable {
    fn update(&mut self, update: &Update);
}

pub trait Drawable {
    fn draw(&self, draw: &Draw, frame: &Frame);
}
