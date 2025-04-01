//! Shorthand atomic load and store operations for common atomic types.
use crate::app::params::*;
use atomic::Atomic;
use atomic_float::{AtomicF64, AtomicF32};
use bytemuck::NoUninit;
use std::sync::atomic::{
    AtomicBool, AtomicI32, AtomicU32, AtomicU8, AtomicUsize, Ordering::Relaxed,
};

/// Trait for shorthand implementation of Relaxed atomic load and store operations.
pub trait AtomicOps: Default {
    type NonAtomic: Default;

    /// Shorthand method for `self.load(Relaxed)`.
    fn lr(&self) -> Self::NonAtomic;
    /// Shorthand method for `self.store(value, Relaxed)`.
    fn sr(&self, value: Self::NonAtomic);
}

impl AtomicOps for AtomicI32 {
    type NonAtomic = i32;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}

impl AtomicOps for AtomicU32 {
    type NonAtomic = u32;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}

impl AtomicOps for AtomicU8 {
    type NonAtomic = u8;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}

impl AtomicOps for AtomicUsize {
    type NonAtomic = usize;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}

impl AtomicOps for AtomicBool {
    type NonAtomic = bool;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}

impl AtomicOps for AtomicF32 {
    type NonAtomic = f32;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}

impl AtomicOps for AtomicF64 {
    type NonAtomic = f64;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}

// impl AtomicLoad for Atomic<GenerativeAlgo> {
//     type NonAtomic = GenerativeAlgo;
//
//     fn lr(&self) -> Self::NonAtomic {
//         // self.Arc::new
//     }
//
//     fn sr(&self, value: Self::NonAtomic) {
//         todo!()
//     }
// }

impl<T: Default + Copy + NoUninit> AtomicOps for Atomic<T> {
    type NonAtomic = T;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}
