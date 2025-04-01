//! Module containing various window functions.
use std::f64::consts::{PI, TAU};

// TODO simd?
// TODO check that the final sample for each window is correct? might be
//  missed at the moment
// TODO all of these windows are symmetrical; is it not more efficient
//  to only compute one half and mirror it?
// TODO add documentation to cosine-sum functions
// TODO add tests

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WindowType {
    Hann,
    Hamming,
    Nuttall,
    Blackman,
    BlackmanNuttall,
    BlackmanHarris,
    FlatTop,
    Tukey,
    Sine,
    Parzen,
    Welch,
}

/// Multiplies each element of both buffers together.
///
/// Requires `target.len() <= other.len()`
pub fn multiply_buffers(target: &mut [f64], other: &[f64]) {
    // assert!(target.len() <= other.len());
    target.iter_mut().zip(other).for_each(|(a, b)| *a *= *b);
}

/// A Parzen window, AKA de la Vallée Poussin window.
///
/// Good side-lobe reduction, with strong rippling.
pub fn parzen(size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    parzen_in_place(&mut vec);
    vec
}

/// In-place variant of `parzen()`.
pub fn parzen_in_place(slice: &mut [f64]) {
    let size = slice.len();
    let l = size + 1;

    let w0 = |n: f64| {
        let n_abs = n.abs();
        let l_2 = l as f64 / 2.0;
        let l_4 = l as f64 / 4.0;

        if 0.0 <= n_abs && n_abs <= l_4 {
            let s1 = 6.0 * (n / l_2).powi(2);
            let s2 = 1.0 - (n_abs / l_2);
            1.0 - s1 * s2
        } else if l_4 <= n_abs && n_abs <= l_2 {
            2.0 * (1.0 - (n_abs / l_2)).powi(3)
        }
        // this is surely unreachable?
        else {
            panic!("bad condition in parzen window");
        }
    };

    for (n, x) in slice.iter_mut().enumerate() {
        *x = w0((n - size / 2) as f64);
    }
}

/// A Welch window.
///
/// Simple computation, but sub-par side-lobe level.
pub fn welch(size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    welch_in_place(&mut vec);
    vec
}

/// In-place variant of `welch()`.
pub fn welch_in_place(slice: &mut [f64]) {
    let half_size = (slice.len() / 2) as f64;

    for (n, x) in slice.iter_mut().enumerate() {
        let n = n as f64;
        *x = 1.0 - ((n - half_size) / half_size).powi(2);
    }
}

/// A sine window.
///
/// Very fast to compute, and decent side-lobe level.
pub fn sine(size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    sine_in_place(&mut vec);
    vec
}

/// In-place variant of `sine()`.
pub fn sine_in_place(slice: &mut [f64]) {
    let size = slice.len() as f64;

    for (n, x) in slice.iter_mut().enumerate() {
        let n = n as f64;
        *x = ((PI * n) / size).sin();
    }
}

/// A Tukey window, AKA a cosine-tapered window.
///
/// This window has two cosine lobes with 1.0 in the centre.
/// The lobe width is determined by `(width * size) / 2.0`,
/// which is variable via the `width` argument.
///
/// Requires that `0.0 <= width && width <= 1.0`.
pub fn tukey(size: usize, width: f64) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    tukey_in_place(&mut vec, width);
    vec
}

/// In-place variant of `tukey()`.
pub fn tukey_in_place(slice: &mut [f64], width: f64) {
    debug_assert!((0.0..=1.0).contains(&width));
    let size = slice.len() as f64;
    let half_size = size / 2.0;

    for i in 0..slice.len() / 2 {
        let n = i as f64;
        let wn = 0.5 * (1.0 - ((TAU * n) / (width * size)).cos());

        let c1 = (width * size) / 2.0;

        if 0.0 <= n && n < c1 {
            slice[i] = wn;
        } else if c1 <= n && n <= half_size {
            slice[i] = 1.0;
        }

        slice[slice.len() - i] = wn;
    }
}

/// This function is used for all the below window functions, which are
/// known as "cosine sum" functions.
fn cosine_sum(slice: &mut [f64], coeffs: &[f64]) {
    let size = slice.len() as f64;

    for (n, x) in slice.iter_mut().enumerate() {
        let mut sum = 0.0;

        // we skip 1 because the first element is only used later
        // and so that we don't multiply τ by 0
        for (i, coeff) in coeffs.iter().enumerate().skip(1) {
            let s1 = ((i as f64 * TAU) * (n as f64) / size).cos();
            sum += coeff * s1;
        }

        *x = coeffs[0] - sum;
    }
}

/// doc
pub fn hann(size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    hann_in_place(&mut vec);
    vec
}

/// In-place variant of `hann()`.
pub fn hann_in_place(slice: &mut [f64]) {
    cosine_sum(slice, &[0.5, 0.5]);
}

/// doc
pub fn hamming(size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    hamming_in_place(&mut vec);
    vec
}

/// In-place variant of `hamming()`.
pub fn hamming_in_place(slice: &mut [f64]) {
    // apparently approximating these coefficients to two decimal places
    // leads to cleaner side-lobes?
    cosine_sum(slice, &[0.54347826, 0.45652174]);
}

/// doc
pub fn nuttall(size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    nuttall_in_place(&mut vec);
    vec
}

/// In-place variant of `nuttall()`.
pub fn nuttall_in_place(slice: &mut [f64]) {
    cosine_sum(slice, &[0.355768, 0.487396, 0.144232, -0.012604]);
}

/// doc
pub fn blackman(size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    blackman_in_place(&mut vec);
    vec
}

/// In-place variant of `blackman()`.
pub fn blackman_in_place(slice: &mut [f64]) {
    // apparently these truncated values lead to better side-lobe
    // fall-off, but the third and fourth side-lobes are not attenuated
    // as well.
    cosine_sum(slice, &[0.42, 0.5, 0.08]);
}

/// doc
pub fn blackman_nuttall(size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    blackman_nuttall_in_place(&mut vec);
    vec
}

/// In-place variant of `blackman_nuttall()`.
pub fn blackman_nuttall_in_place(slice: &mut [f64]) {
    cosine_sum(slice, &[0.3635819, 0.4891775, 0.1365995, -0.0106411]);
}

/// doc
pub fn blackman_harris(size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    blackman_harris_in_place(&mut vec);
    vec
}

/// In-place variant of `blackman_harris()`.
pub fn blackman_harris_in_place(slice: &mut [f64]) {
    cosine_sum(slice, &[0.35875, 0.48829, 0.14128, -0.01168]);
}

/// doc
pub fn flat_top(size: usize) -> Vec<f64> {
    let mut vec = vec![0.0; size];
    flat_top_in_place(&mut vec);
    vec
}

/// In-place variant of `flat_top()`.
pub fn flat_top_in_place(slice: &mut [f64]) {
    cosine_sum(
        slice,
        &[
            0.21557895,
            0.41663158,
            0.27726316,
            -0.083578947,
            -0.006947368,
        ],
    );
}

#[cfg(test)]
mod tests {
    // TODO
}
