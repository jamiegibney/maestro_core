//! Value smoothers.

/// Non-atomic linear segment generation. Internal system for `Smoother`.
mod ramp;
/// Atomic linear segment generation. Internal system for `SmootherAtomic`.
mod ramp_atomic;

/// Smoothable traits and type implementations.
pub mod smoothable_types;
/// Non-atomic value smoothing.
pub mod smoother;
/// Atomic value smoothing.
pub mod smoother_atomic;
pub use smoothable_types::{Smoothable, SmoothableAtomic};
pub use smoother::Smoother;
pub use smoother_atomic::SmootherAtomic;

use super::{eps_eq, eps_eq_f32};

#[allow(clippy::suboptimal_flops)]
pub fn smooth_damp(
    current: f64,
    mut target: f64,
    current_velocity: &mut f64,
    mut smoothing_time: f64,
    delta_time: f64,
    max_speed: f64,
) -> f64 {
    if (eps_eq(current, target)) {
        *current_velocity = 0.0;
        return current;
    }

    smoothing_time = smoothing_time.max(0.0001);
    let omega = 2.0 / smoothing_time;

    let x = omega * delta_time;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let mut change = current - target;
    let original_to = target;

    let max_change = max_speed * smoothing_time;
    change = change.clamp(-max_change, max_change);
    target = current - change;

    let tmp = (*current_velocity + omega * change) * delta_time;
    *current_velocity = (*current_velocity - omega * tmp) * exp;
    let mut output = target + (change + tmp) * exp;

    if (original_to - current > 0.0) == (output > original_to) {
        output = original_to;
        *current_velocity = (output - original_to) / delta_time;
    }

    output
}

#[allow(clippy::suboptimal_flops)]
pub fn smooth_damp_f32(
    current: f32,
    mut target: f32,
    current_velocity: &mut f32,
    mut smoothing_time: f32,
    delta_time: f32,
    max_speed: f32,
) -> f32 {
    if (eps_eq_f32(current, target)) {
        *current_velocity = 0.0;
        return current;
    }

    smoothing_time = smoothing_time.max(0.0001);
    let omega = 2.0 / smoothing_time;

    let x = omega * delta_time;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let mut change = current - target;
    let original_to = target;

    let max_change = max_speed * smoothing_time;
    change = change.clamp(-max_change, max_change);
    target = current - change;

    let tmp = (*current_velocity + omega * change) * delta_time;
    *current_velocity = (*current_velocity - omega * tmp) * exp;
    let mut output = target + (change + tmp) * exp;

    if (original_to - current > 0.0) == (output > original_to) {
        output = original_to;
        *current_velocity = (output - original_to) / delta_time;
    }

    output
}
