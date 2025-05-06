use super::*;

#[derive(Clone, Copy, Debug)]
pub struct ParameterState {
    pub mode: Mode,
}

impl Default for ParameterState {
    fn default() -> Self {
        Self { mode: Mode::default() }
    }
}
