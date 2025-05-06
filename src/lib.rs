#![feature(let_chains)]
#![allow(
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    clippy::return_self_not_must_use,
    clippy::redundant_closure_for_method_calls,
    unused
)]

// GUI and program related
pub mod app;

// Signal processing
pub mod dsp;

// General utilities
pub mod util;

// Some widely-used re-exports
pub mod prelude;

// Program-wide settings
pub mod settings;
