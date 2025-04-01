use atomic::Atomic;

use super::*;
use std::{
    ops::DerefMut,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

pub struct TimerThread {
    cb: Arc<Mutex<dyn FnMut() + Send + Sync + 'static>>,
    thread: Option<JoinHandle<()>>,

    counter_secs: Arc<Atomic<f64>>,
    interval_secs: Arc<Atomic<f64>>,

    sentinel: Arc<AtomicBool>,
}

impl TimerThread {
    pub fn new<F: FnMut() + Send + Sync + 'static>(cb: F) -> Self {
        Self {
            cb: Arc::new(Mutex::new(cb)),
            thread: None,

            counter_secs: Arc::new(Atomic::new(0.0)),
            interval_secs: Arc::new(Atomic::new(0.0)),

            sentinel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self, interval_secs: f64) {
        if self.thread.is_some() {
            return;
        }

        self.interval_secs.store(interval_secs, Ordering::Release);
        self.sentinel.store(true, Ordering::Release);

        let counter = Arc::clone(&self.counter_secs);
        let interval = Arc::clone(&self.interval_secs);
        let sentinel = Arc::clone(&self.sentinel);
        let cb = Arc::clone(&self.cb);

        let thread = std::thread::spawn(move || {
            let mut now = std::time::Instant::now();
            let mut dt = || {
                // NOTE(jamie): to improve the precision of the timer we sleep
                // for a short period of time to allow more time to accumulate,
                // which reduces errors from a lack of nanosecond precision.
                std::thread::sleep(Duration::from_micros(20));

                let elapsed = now.elapsed().as_secs_f64();
                now = std::time::Instant::now();
                elapsed
            };

            while sentinel.load(Ordering::Acquire) {
                let curr_count = counter.load(Ordering::Acquire);
                let curr_interval = interval.load(Ordering::Acquire);

                if (curr_count >= curr_interval) {
                    if let Ok(mut guard) = cb.lock() {
                        guard.deref_mut()();
                    }

                    counter
                        .store(curr_count - curr_interval, Ordering::Release);
                    _ = dt();

                    continue;
                }

                counter.store(curr_count + dt(), Ordering::Release);
            }
        });

        self.thread = Some(thread);
    }

    pub fn start_hz(&mut self, interval_rate_hz: f64) {
        self.start(interval_rate_hz.recip());
    }

    // TODO: the timeout is technically useless - separate to different method?
    pub fn stop(&mut self, timeout_secs: Option<f64>) {
        if let Some(thread) = self.thread.take() {
            self.sentinel.store(false, Ordering::Release);

            let mut elapsed = 0.0;
            let now = std::time::Instant::now();

            if let Some(timeout) = timeout_secs {
                #[allow(clippy::while_float)]
                while elapsed < timeout {
                    if thread.is_finished() {
                        break;
                    }

                    elapsed = now.elapsed().as_secs_f64();
                }
            }

            _ = thread.join();
        }
    }

    pub const fn is_running(&self) -> bool {
        self.thread.is_some()
    }

    pub fn interval(&self) -> f64 {
        self.interval_secs.load(Ordering::Acquire)
    }
}
