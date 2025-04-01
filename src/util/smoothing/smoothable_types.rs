//! Types supported for use with `Smoother` and `SmootherAtomic`.

use atomic_float::{AtomicF32, AtomicF64};
use std::sync::atomic::{AtomicI32, Ordering};

/// Types which may be smoothed in `SmootherAtomic`. Used to avoid explicit duplications of the
/// `SmootherAtomic` implementation.
pub trait SmoothableAtomic: Default + Clone + Copy {
    type Atomic: Default;

    fn to_f32(self) -> f32;
    fn from_f32(value: f32) -> Self;

    fn to_f64(self) -> f64;
    fn from_f64(value: f64) -> Self;

    fn atomic_new(self) -> Self::Atomic;
    fn atomic_load(this: &Self::Atomic) -> Self;
    fn atomic_store(this: &Self::Atomic, value: Self);
}

impl SmoothableAtomic for f32 {
    type Atomic = AtomicF32;

    #[inline]
    fn to_f32(self) -> f32 {
        self
    }

    #[inline]
    fn from_f32(value: f32) -> Self {
        value
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self as f64
    }

    #[inline]
    fn from_f64(value: f64) -> Self {
        value as Self
    }

    #[inline]
    fn atomic_new(self) -> Self::Atomic {
        AtomicF32::new(self)
    }

    #[inline]
    fn atomic_load(this: &Self::Atomic) -> Self {
        this.load(Ordering::Relaxed)
    }

    fn atomic_store(this: &Self::Atomic, value: Self) {
        this.store(value, Ordering::Relaxed);
    }
}

impl SmoothableAtomic for f64 {
    type Atomic = AtomicF64;

    #[inline]
    fn to_f32(self) -> f32 {
        self as f32
    }

    #[inline]
    fn from_f32(value: f32) -> Self {
        value as Self
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self
    }

    #[inline]
    fn from_f64(value: f64) -> Self {
        value
    }

    #[inline]
    fn atomic_new(self) -> Self::Atomic {
        AtomicF64::new(self)
    }

    #[inline]
    fn atomic_load(this: &Self::Atomic) -> Self {
        this.load(Ordering::Relaxed)
    }

    #[inline]
    fn atomic_store(this: &Self::Atomic, value: Self) {
        this.store(value, Ordering::Relaxed)
    }
}

impl SmoothableAtomic for i32 {
    type Atomic = AtomicI32;

    #[inline]
    fn to_f32(self) -> f32 {
        self as f32
    }

    #[inline]
    fn from_f32(value: f32) -> Self {
        value as Self
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self as f64
    }

    #[inline]
    fn from_f64(value: f64) -> Self {
        value as Self
    }

    #[inline]
    fn atomic_new(self) -> Self::Atomic {
        AtomicI32::new(self)
    }

    #[inline]
    fn atomic_load(this: &Self::Atomic) -> Self {
        this.load(Ordering::Relaxed)
    }

    #[inline]
    fn atomic_store(this: &Self::Atomic, value: Self) {
        this.store(value, Ordering::Relaxed);
    }
}

/// Types which may be smoothed by `Smoother`. Non-atomic operations.
pub trait Smoothable: Default + Clone + Copy {
    fn to_f32(self) -> f32;
    fn from_f32(value: f32) -> Self;

    fn to_f64(self) -> f64;
    fn from_f64(value: f64) -> Self;
}

impl Smoothable for f32 {
    #[inline]
    fn to_f32(self) -> f32 {
        self
    }

    #[inline]
    fn from_f32(value: f32) -> Self {
        value
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self as f64
    }

    #[inline]
    fn from_f64(value: f64) -> Self {
        value as Self
    }
}

impl Smoothable for f64 {
    #[inline]
    fn to_f32(self) -> f32 {
        self as f32
    }

    #[inline]
    fn from_f32(value: f32) -> Self {
        value as Self
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self
    }

    #[inline]
    fn from_f64(value: f64) -> Self {
        value
    }
}

impl Smoothable for i32 {
    #[inline]
    fn to_f32(self) -> f32 {
        self as f32
    }

    #[inline]
    fn from_f32(value: f32) -> Self {
        value as Self
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self as f64
    }

    #[inline]
    fn from_f64(value: f64) -> Self {
        value as Self
    }
}
