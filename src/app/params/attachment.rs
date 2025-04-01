use super::*;
use hands::hand_types::SignificantHandValues;
use midi_types::*;
use state::ParameterState;

pub type MIDICCFn = fn(&SignificantHandValues, &mut f32);
pub type MIDICCPredicate = fn(&ParameterState) -> bool;

#[derive(Clone, Debug)]
struct CCSmoother {
    time: f32,
    curr: f32,
    velocity: f32,
}

impl CCSmoother {
    pub const fn with_time(time: f32) -> Self {
        Self { time, curr: 0.0, velocity: 0.0 }
    }

    pub fn get_next(&mut self, target: f32, delta_time: f32) -> f32 {
        self.curr = smooth_damp_f32(
            self.curr, target, &mut self.velocity, self.time, delta_time, 100.0,
        );
        self.curr
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MIDICCSize {
    CC7Bit,
    CC14Bit,
}

#[derive(Clone, Debug)]
pub struct MIDICCAttachment {
    name: String,
    callback: MIDICCFn,
    predicate: MIDICCPredicate,
    smoother: Option<CCSmoother>,
    size: MIDICCSize,
    update_threshold: f32,
}

impl MIDICCAttachment {
    pub fn new(
        name: &str,
        callback: MIDICCFn,
        predicate: MIDICCPredicate,
        smoothing_time: Option<f32>,
        size: MIDICCSize,
        update_threshold: f32,
    ) -> Self {
        Self {
            name: String::from(name),
            callback,
            predicate,
            smoother: smoothing_time.map(CCSmoother::with_time),
            size,
            update_threshold: DEFAULT_MIDI_CC_UPDATE_THRESHOLD,
        }
    }

    pub fn with_smoothing_time(&mut self, smoothing_time: f32) -> &mut Self {
        self.smoother = Some(CCSmoother::with_time(smoothing_time));
        self
    }

    pub const fn with_update_threshold(&mut self, threshold: f32) -> &mut Self {
        self.update_threshold = threshold;
        self
    }

    pub fn with_size(&mut self, size: MIDICCSize) -> &mut Self {
        self.size = size;
        self
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn callback(
        &mut self,
        significant_values: &SignificantHandValues,
        cc_value: &mut f32,
        delta_time: f32,
    ) {
        if let Some(smoother) = &mut self.smoother {
            let mut tmp = *cc_value;
            (self.callback)(significant_values, &mut tmp);

            *cc_value = smoother.get_next(tmp, delta_time);
        }
        else {
            (self.callback)(significant_values, cc_value);
        }
    }

    pub fn is_active_for(&self, state: &ParameterState) -> bool {
        (self.predicate)(state)
    }

    pub const fn is_14_bit(&self) -> bool {
        matches!(self.size, MIDICCSize::CC14Bit)
    }

    pub const fn update_threshold(&self) -> f32 {
        self.update_threshold
    }
}
