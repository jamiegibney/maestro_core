//! Interpolation functions and types.
use std::f64::consts::PI;

/// Various interpolation types, including `NoInterp`.
#[derive(Debug, Clone, Copy, Default)]
pub enum InterpolationType {
    /// No interpolation.
    NoInterp,
    /// Linear interpolation from a -> b.
    #[default]
    Linear,
    /// Cosine interpolation from a -> b.
    Cosine,
    /// Standard cubic interpolation from b -> c, given samples a, b, c and d.
    DefaultCubic,
    /// Catmull-Rom cubic interpolation from b -> c, given samples a, b, c and d.
    CatmullCubic,
    /// Hermite cubic interpolation from b -> c, given samples a, b, c and d.
    /// The values correspond to `tension` and `bias` arguments.
    HermiteCubic(f64, f64),
}

/// Shorthand for the `Interp::linear` function.
///
/// `t` is clamped between `0` and `1`.
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    linear(a, b, t)
}

/// Linearly interpolates between `a` and `b` based on the value of `t`.
///
/// `t` is clamped between `0` and `1`.
pub fn linear(a: f64, b: f64, t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        return a;
    } else if t == 1.0 {
        return b;
    }

    t.mul_add(b - a, a)
}

/// Linearly interpolates between `a` and `b` based on the value of `t`.
///
/// The output may be an extrapolation of the input if `t` exceeds `0` or `1`.
pub fn linear_unclamped(a: f64, b: f64, t: f64) -> f64 {
    t.mul_add(b - a, a)
}

/// "Inverse linear interpolation": finds the interpolation value
/// within a range.
pub fn ilerp(a: f64, b: f64, val: f64) -> f64 {
    if b == a {
        return 0.0;
    }

    (val - a) / (b - a)
}

/// Interpolates between `a` and `b` based on the value of `t`, using
/// a cosine wave as the transfer function.
///
/// `t` is clamped between `0` and `1`.
pub fn cosine(a: f64, b: f64, t: f64) -> f64 {
    let t = (1.0 - (PI * t.clamp(0.0, 1.0)).cos()) * 0.5;

    linear(a, b, t)
}

/// Interpolates between `p1` and `p2` based on the value of `t` using
/// cubic interpolation, which requires four samples.
///
/// `t` is clamped between `0` and `1`.
pub fn cubic(p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    let t2 = t * t;
    let t3 = t2 * t;

    let a = p3 - p2 - p0 + p1;
    let b = p0 - p1 - a;
    let c = p2 - p0;
    let d = p1;

    c.mul_add(t, a.mul_add(t3, b * t2)) + d
}

/// Performs `cubic()`, but with a vector of four points.
///
/// # Panics
///
/// `panic!`s if `points` holds less than four values.
pub fn cubic_vec(points: &[f64], t: f64) -> f64 {
    assert!(
        points.len() >= 4,
        "pa::interp::cubic_vec passed a vector containing {} values, but the function needs 4",
        points.len()
    );

    cubic(points[0], points[1], points[2], points[3], t)
}

/// Interpolates between `p1` and `p2` based on the value of t using Catmull-Rom
/// cubic interpolation, which requires four samples.
/// This is more expensive than the `Interp::cubic` function, but has a smoother
/// linear response, i.e. where the difference between points is similar.
///
/// `t` is clamped between `0` and `1`.
pub fn cubic_catmull(p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    let t2 = t * t;
    let t3 = t2 * t;

    // let a = 0.5f64.mul_add(p3, 1.5f64.mul_add(-p2, (-0.5f64).mul_add(p0, 1.5 * p1)));
    // let b = 0.5f64.mul_add(-p3, 2.0f64.mul_add(p2, 2.5f64.mul_add(-p1, p0)));
    // let c = (-0.5f64).mul_add(p0, 0.5 * p2);
    // let d = p1;

    let a = -0.5 * p0 + 1.5 * p1 - 1.5 * p2 + 0.5 * p3;
    let b = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
    let c = -0.5 * p0 + 0.5 * p2;
    let d = p1;

    c.mul_add(t, a.mul_add(t3, b * t2)) + d
}

/// Performs `Interp::cubic_catmull`, but with a vector of four points.
///
/// # Panics
///
/// `panic!`s if `points` holds less than four values.
pub fn cubic_catmull_vec(points: Vec<f64>, t: f64) -> f64 {
    assert!(points.len() >= 4, "pa::Interp::cubic_catmull_vec passed a vector containing {} values, but the function needs 4",
               points.len());

    cubic_catmull(points[0], points[1], points[2], points[3], t)
}

/// Interpolates between `p1` and `p2` based on the value of `t` using Hermite
/// cubic interpolation, which requires four samples. This is more expensive than
/// the `Interp::cubic_catmull` function, but allows for extra control via the `tension`
/// and `bias` arguments.
///
/// `t` is clamped between `0` and `1`.
///
/// `tension`: positive values increase curve "tension", negative values reduce it. `0` is
/// the default, "unaffected" value.
///
/// `bias`: positive values "skew" toward the last point, and negative values toward the first.
/// `0` is the default, "unaffected" value.
pub fn cubic_hermite(p0: f64, p1: f64, p2: f64, p3: f64, t: f64, tension: f64, bias: f64) -> f64 {
    // this is used to prevent unnecessary computations in specific cases
    if bias == 0.0 {
        if tension == 0.0 {
            return cubic_catmull(p0, p1, p2, p3, t);
        }
        if tension == 1.0 {
            return cubic(p0, p1, p2, p3, t);
        }
    }

    let t = t.clamp(0.0, 1.0);
    let t2 = t * t;
    let t3 = t2 * t;

    let a = 2.0 * t3 - 3.0 * t2 + 1.0;
    let b = t3 - 2.0 * t2 + t;
    let c = t3 - t2;
    let d = -2.0 * t3 + 3.0 * t2;

    let mut m0 = ((p1 - p0) * (1.0 + bias) + (1.0 - tension)) * 0.5;
    m0 += ((p2 - p1) * (1.0 - bias) * (1.0 - tension)) * 0.5;

    let mut m1 = ((p3 - p2) * (1.0 + bias) * (1.0 - tension)) * 0.5;
    m1 += ((p3 - p2) * (1.0 - bias) * (1.0 - tension)) * 0.5;

    a * p1 + b * m0 + c * m1 + d * p2
}

/// Performs `Interp::cubic_hermite`, but with a vector of four points.
///
/// `panic!`s if `points` holds less than four values.
pub fn cubic_hermite_vec(points: Vec<f64>, t: f64, tension: f64, bias: f64) -> f64 {
    if points.len() < 4 {
        panic!("pa::Interp::cubic_hermite_vec passed a vector containing {} values, but the function needs 4",
               points.len());
    }

    cubic_hermite(points[0], points[1], points[2], points[3], t, tension, bias)
}

// TODO: unit tests for interpolation algorithms...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interp() {
        todo!()
    }

    #[test]
    fn test_cosine_interp() {
        todo!()
    }

    #[test]
    fn test_cubic_interp() {
        todo!()
    }

    #[test]
    fn test_camull_rom_interp() {
        todo!()
    }

    #[test]
    fn test_hermite_interp() {
        todo!()
    }
}
