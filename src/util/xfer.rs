//! Transfer functions and types.
use super::map;
use crate::prelude::*;
use std::f64::consts::PI;

#[derive(Debug, Copy, Clone, Default)]
pub enum SmoothingType {
    /// Linear mapping from `a -> b`
    #[default]
    Linear,
    /// Cosine function mapping from `a -> b`
    Cosine,
    /// Quarter-sine function mapping from `a -> b`, biased towards b
    SineTop,
    /// Quarter-sine function mapping from `a -> b`, biased towards a
    SineBottom,
    /// Standard curve mapping from `a -> b` with tension argument
    CurveNormal(f64),
    /// Curved mapping from `a -> b` with tension argument and a linear start
    CurveLinearStart(f64),
    /// Rounder curve mapping from `a -> b` with tension argument
    CurveRounder(f64),
}

/// Returns an s-curve function.
///
/// Negative tension values produce curves which "skew inwards" (like y = x²),
/// whereas positive values produce curves which "skew outwards", like an s-curve.
///
/// `input` and `tension` are clamped between `-1.0` and `1.0`.
pub fn s_curve(mut input: f64, tension: f64) -> f64 {
    input = input.clamp(-1.0, 1.0);
    let c = scale(tension, 1.0, 0.05).recip();

    if tension.is_sign_positive() {
        if input.is_sign_positive() {
            -(1.0 - input).powf(c) + 1.0
        }
        else {
            (input + 1.0).powf(c) - 1.0
        }
    }
    else if input.is_sign_positive() {
        input.powf(c)
    }
    else {
        1.0 - (-input).powf(c) - 1.0
    }
}

/// Returns a rounded s-curve function with a linear centre.
///
/// `input` is clamped between `-1.0` and `1.0`.
///
/// `tension` is clamped between `0.0` and `1.0`.
pub fn s_curve_linear_centre(input: f64, tension: f64) -> f64 {
    let x = input.clamp(-1.0, 1.0);
    let c = tension.clamp(0.0, 1.0);

    let x2 = 2.0 * x;
    let sq = x * x;
    let k = 1.0 - c;
    let t_min1_sq = (c - 1.0) * (c - 1.0);

    if k < x && x <= 1.0 {
        (sq - x2 + t_min1_sq) / (c * (c - 2.0))
    }
    else if -1.0 <= x && x <= -k {
        (sq + x2 + t_min1_sq) / (c * (2.0 - c))
    }
    else {
        x
    }
}

/// Returns a more round s-curve function.
///
/// Negative `tension` values produce curves which "skew inwards" (like y = x²),
/// whereas positive values produce curves which "skew outwards", like an s-curve.
///
/// `input` and `tension` are clamped between `-1.0` and `1.0`.
// TODO: this does not work properly, and is essentially x^2.
pub fn s_curve_round(input: f64, tension: f64) -> f64 {
    let x = input.clamp(-1.0, 1.0);

    // this maps the curvature similarly for positive and negative inputs
    let c = if tension < 0.0 {
        tension.clamp(-1.0, 0.0) * 0.907
    }
    else {
        tension.clamp(0.0, 1.0) * 10.0
    };

    if 0.0 < x && x <= 1.0 {
        x * (1.0 + c) / c.mul_add(x, 1.0)
    }
    else {
        -x * (1.0 + c) / c.mul_add(x, -1.0)
    }
}

/* TODO check these sine functions for correctness */
/// Returns a whole sine function `(-π -> π)`. The output is normalised.
///
/// `input` is clamped between `0.0` and `1.0`.
pub fn sine(input: f64) -> f64 {
    let input = input.clamp(0.0, 1.0);
    (input.mul_add(PI, PI).cos() + 1.0) * 0.5
}

/// Returns the "upper part" of a sine function `(0 -> π)`. The output is normalised.
///
/// `input` is clamped between `0.0` and `1.0`.
pub fn sine_upper(input: f64) -> f64 {
    let input = input.clamp(0.0, 1.0);
    (input * PI / 2.0).sin()
}

/// Returns the "lower part" of a sine function `(-π -> 0)`. The output is normalised.
///
/// `input` is clamped between `0.0` and `1.0`.
pub fn sine_lower(input: f64) -> f64 {
    let input = input.clamp(0.0, 1.0);
    (input * PI / 2.0 + PI).cos() + 1.0
}

/// Returns a hyperbolic tangent curve function.
///
/// `tension` is clamped between `0.0` and `1.0`, and is mapd between
/// `1.0` and `10.0` for a more natural range of curves.
///
/// `input` is clamped between `-1.0` and `1.0`.
pub fn tanh(input: f64, tension: f64) -> f64 {
    let x = input.clamp(-1.0, 1.0);
    let c = map(tension.clamp(0.0, 1.0), 0.0, 1.0, 1.0, 10.0);

    (x * c).tanh()
}

/// Clamps the input range between `0.0` and `1.0`.
pub fn gentle_under(mut input: f64) -> f64 {
    input = input.clamp(0.0, 1.0);

    1.0 - ((PI / 2.0) * (1.0 - input)).sin()
}

/// Clamps the input range between `0.0` and `1.0`.
pub fn gentle_over(mut input: f64) -> f64 {
    input = input.clamp(0.0, 1.0);

    (PI / 2.0 * input).sin()
}

/// Clamps the input range between `0.0` and `1.0`.
pub fn strong_under(mut input: f64) -> f64 {
    input = input.clamp(0.0, 1.0);

    input.powi(3)
}

/// Clamps the input range between `0.0` and `1.0`.
pub fn strong_over(mut input: f64) -> f64 {
    input = input.clamp(0.0, 1.0);

    1.0 - (1.0 - input).powi(3)
}

/// Clamps the input range between `0.0` and `1.0`.
pub fn skewed_sine(mut input: f64) -> f64 {
    input = input.clamp(0.0, 1.0);

    0.5f64.mul_add((PI * input.mul_add(input, -0.5)).sin(), 0.5)
}

/// Smooth soft saturation function. `input` is clamped between `-1.0` and `1.0`,
/// and `c` is clamped between `0.0` and `1.0`. Outputs in the range `-1.0` to `1.0`.
///
/// Raising `c` will increase the amount of saturation.
///
/// From <https://www.musicdsp.org/en/latest/Effects/42-soft-saturation.html>
///
/// Visualisation: <https://www.desmos.com/calculator/6kc511fbhe>
pub fn smooth_soft_clip(mut input: f64, mut c: f64) -> f64 {
    input = input.clamp(-1.0, 1.0);
    c = 1.0 - c.clamp(0.0, 1.0);

    let abs = input.abs();
    let sign = if input.is_sign_positive() { 1.0 } else { -1.0 };

    if abs > 1.0 {
        ((c + 1.0) / 2.0) * sign
    }
    else if abs > c {
        let inner = (abs - c) / (1.0 - c);

        (c + (abs - c) / (1.0 + (inner.powi(2)))) * sign
    }
    else {
        input
    }
}
