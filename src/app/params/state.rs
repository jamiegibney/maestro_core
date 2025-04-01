use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ProgramMode {
    #[default]
    A,
    B,
    C,
    D,
}

pub struct ParameterState {
    pub mode: ProgramMode,
}

impl Default for ParameterState {
    fn default() -> Self {
        Self { mode: Default::default() }
    }
}
