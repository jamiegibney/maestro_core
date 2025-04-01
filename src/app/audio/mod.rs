//! Audio state and processing.

use super::*;
use crate::dsp::*;

use nannou_audio::Buffer;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use thread_pool::ThreadPool;

pub mod context;
pub mod model;
pub mod process;
pub mod voice;

pub use context::AudioContext;
pub use model::*;
pub use process::process;
pub use voice::*;

pub const DSP_LOAD_AVERAGING_SAMPLES: usize = 32;
