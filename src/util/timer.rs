use atomic::Atomic;

use super::*;
use std::{
    ops::DerefMut,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

/// A type for asynchronous, periodic callbacks. You may provide any
/// (thread-safe) callback and let this type invoke periodically with any time
/// interval.
///
/// For more details, see the type's methods (e.g. [`TimerThread::start()`].
pub struct TimerThread {
    cb: Arc<Mutex<dyn FnMut() + Send + Sync + 'static>>,
    thread: Option<JoinHandle<()>>,

    counter_secs: Arc<Atomic<f64>>,
    interval_secs: Arc<Atomic<f64>>,

    continue_sentinel: Arc<AtomicBool>,
    timeout_counter: Arc<AtomicU32>,
}

impl TimerThread {
    /// Creates a new `TimerThread` with the provided callback, which can be
    /// periodically and asynchronously invoked by a separate thread.
    pub fn new<F: FnMut() + Send + Sync + 'static>(cb: F) -> Self {
        Self {
            cb: Arc::new(Mutex::new(cb)),
            thread: None,

            counter_secs: Arc::new(Atomic::new(0.0)),
            interval_secs: Arc::new(Atomic::new(0.0)),

            continue_sentinel: Arc::new(AtomicBool::new(false)),
            timeout_counter: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Starts the timer thread with the provided interval in seconds. This is
    /// the amount of time *between* callbacks — for a per-second rate, use
    /// [`TimerThread::start_hz()`].
    ///
    /// Please note that the actual timer interval will not always exactly match
    /// the requested interval. The shorter the timer interval, the lower the
    /// precision of the actual callback interval.
    ///
    /// To stop the thread, use either the [`TimerThread::stop()`] or
    /// [`TimerThread::stop_after_num_callbacks()`]  method.
    pub fn start(&mut self, interval_secs: f64) {
        if self.thread.is_some() {
            return;
        }

        self.interval_secs.store(interval_secs, Ordering::Release);
        self.continue_sentinel.store(true, Ordering::Release);

        let counter = Arc::clone(&self.counter_secs);
        let interval = Arc::clone(&self.interval_secs);
        let sentinel = Arc::clone(&self.continue_sentinel);
        let timeout = Arc::clone(&self.timeout_counter);
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

            while sentinel.load(Ordering::Acquire)
                || timeout.load(Ordering::Acquire) > 0
            {
                let curr_count = counter.load(Ordering::Acquire);
                let curr_interval = interval.load(Ordering::Acquire);

                if (curr_count >= curr_interval) {
                    if !sentinel.load(Ordering::Acquire)
                        && timeout.load(Ordering::Acquire) > 0
                    {
                        let to = timeout.fetch_sub(1, Ordering::Release);
                    }

                    if let Ok(mut guard) = cb.lock() {
                        let callback = &mut *guard;
                        callback();
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

    /// Starts the timer thread with a rate interval in Hz (i.e. the number of
    /// times the callback should be invoked per second).
    ///
    /// See [`TimerThread::start()`] for more details.
    pub fn start_hz(&mut self, interval_rate_hz: f64) {
        self.start(interval_rate_hz.recip());
    }

    /// Signals the timer thread to stop, and waits for it to join.
    ///
    /// Please note that this method does not guarantee that the timer thread
    /// will join immediately, nor does it guarantee that its callback will not
    /// be called again before it joins.
    pub fn stop(&mut self) {
        if let Some(thread) = self.thread.take() {
            self.continue_sentinel.store(false, Ordering::Release);

            let mut elapsed = 0.0;
            let now = std::time::Instant::now();

            _ = thread.join();
        }
    }

    /// Signals the timer thread to stop after it has invoked its callback at
    /// least `num_callbacks` times. This is useful if you need to stop the
    /// timer thread, but need to process some existing information in a queue,
    /// for instance.
    pub fn stop_after_num_callbacks(
        &mut self,
        num_callbacks: u32,
        timeout_secs: Option<f64>,
    ) {
        self.timeout_counter.store(num_callbacks, Ordering::Release);
        self.stop();
    }

    /// Whether the timer thread is currently running.
    pub const fn is_running(&self) -> bool {
        self.thread.is_some()
    }

    /// The timer interval — the amount of time between timer callbacks.
    pub fn interval_secs(&self) -> f64 {
        self.interval_secs.load(Ordering::Acquire)
    }
}
