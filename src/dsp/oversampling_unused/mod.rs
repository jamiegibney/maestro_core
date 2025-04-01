//! Oversampling methods and filtering. Based on the JUCE implementation.
//!
//! Unused in this project, but kept for future reference.

use super::filters::filter_design::*;
use super::*;

mod equiripple_fir;
mod block;
pub mod oversampler;

pub use oversampler::Oversampler;

#[derive(Clone)]
pub enum OversamplingFilterType {
    HalfBandEquiripple(u32),
    HaldBandPolyphase,
    NumFilterTypes,
}

pub(super) trait OversamplingStage {
    fn latency_samples(&self) -> f64 { 0.0 }
    fn initialize(&mut self, max_samples_before_oversampling: usize) {}
    fn reset(&mut self) {}
    /// IMPORTANT: This assumes `block` is a block of interleaved samples.
    fn process_samples_up(&mut self, block: &[f64]);
    /// IMPORTANT: This assumes `block` is a block of interleaved samples.
    fn process_samples_down(&mut self, block: &mut [f64]);
}

// #[derive(Clone)]
// pub struct Oversampler {
//     filtering: OversamplingFilterType,
//
//     pub factor: usize,
//     stages: Vec<OversamplingStage>,
//
//     pub num_channels: usize,
//
//     is_ready: bool,
//     should_use_integer_latency: bool,
//
//     delay: RingBuffer,
//     fractional_delay: f64,
//     // vec: Vec<f64>,
//     //
//     // write_pos: Vec<usize>,
//     // read_pos: Vec<usize>,
//     //
//     // delay: f64,
//     // delay_frac: f64,
//     // delay_int: i32,
//     //
//     // total_size: i32,
//     //
//     // alpha: f64,
// }
//
// impl Oversampler {
//     pub fn new(num_channels: usize, factor: usize) -> Self {
//         Self {
//             filtering: todo!(),
//
//             factor,
//             stages: vec![],
//
//             num_channels,
//
//             is_ready: false,
//             should_use_integer_latency: false,
//
//             delay: RingBuffer::new(8),
//             fractional_delay: 0.0,
//         }
//     }
//
//     pub fn set_int_latency(&mut self, should_use_integer_latency: bool) {}
//
//     pub fn latency_samples(&self) -> f64 {
//         0.0
//     }
//
//     pub fn factor(&self) -> usize {
//         self.factor
//     }
//
//     pub fn initialize(&mut self, max_samples_before_oversampling: usize) {}
//
//     pub fn reset(&mut self) {}
//
//     pub fn process_samples_up(&mut self, input_block: &[f64]) -> Vec<f64> {
//         vec![]
//     }
//
//     pub fn process_samples_down(&mut self, output_block: &mut [f64]) {}
//
//     pub fn add_oversampling_stage(
//         &mut self,
//         filter_type: OversamplingFilterType,
//         norm_transition_width_up: f64,
//         stopband_amplitude_db_up: f64,
//         norm_transition_width_down: f64,
//         stopband_amplitude_db_down: f64,
//     ) {
//     }
//
//     pub fn add_dummy_stage(&mut self) {}
//
//     pub fn clear_stages(&mut self) {}
//
//     fn uncompensated_latency(&self) -> f64 {
//         0.0
//     }
//
//     fn update_delay(&mut self) {}
// }
//
// trait Oversampling {
//     fn new(num_channels: usize) -> Oversampler;
//
//     fn latency_samples(&self) -> f64;
//
//     fn initialize(&mut self, max_samples_before_oversampling: usize) {}
//
//     fn reset(&mut self) {}
//
//     fn get_processed_samples(&mut self, num_samples: usize) -> Vec<f64> {
//         vec![]
//     }
//
//     fn process_samples_up(&mut self, input_samples: &[f64]);
//
//     fn process_samples_down(&mut self, output_samples: &mut [f64]);
// }
//
// struct Dummy;
//
// impl Oversampling for Dummy {
//     fn new(num_channels: usize) -> Oversampler {
//         Oversampler::new(num_channels, 1)
//     }
//
//     fn latency_samples(&self) -> f64 {
//         0.0
//     }
//
//     fn process_samples_up(&mut self, input_samples: &[f64]) {}
//
//     fn process_samples_down(&mut self, output_samples: &mut [f64]) {
//         todo!()
//     }
// }
