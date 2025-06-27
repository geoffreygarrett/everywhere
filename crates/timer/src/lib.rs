//! Cross‑timer – fires a callback after an exponential (or custom) delay
//! sequence.  **Runtime transparent**: relies on `cross_runtime::{task,time}`.
//!
//! ```no_run
//! use everywhere_timer::Timer;
//! use core::time::Duration;
//!
//! let retries = Timer::new(|| println!("tick"), |n| Duration::from_millis(100 * (1 << n)));
//! retries.schedule_timeout();
//! ```

// #![cfg_attr(not(feature = "std"), no_std)]
#![allow(async_fn_in_trait)]

// /*──────────── prerequisites ───────────*/
// #[cfg(not(feature = "std"))]
// extern crate alloc;
// extern crate core;
// #[cfg(feature = "std")]
use std::sync::Arc;

// #[cfg(not(feature = "std"))]
// use alloc::sync::Arc;

use core::fmt::Debug;
use core::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use everywhere_runtime::{task, time, Rt, _RtTrait as Runtime};
use spin::Mutex;
// glue only!

/*──────────── handle alias ───────────*/
type Handle = <Rt as Runtime>::JoinHandle;

// /*──────────── internal helpers ───────*/
// fn spawn<F>(f: F) -> Handle
// where
//     F: Future<Output = ()> +         // Send only on native
//     cfg_if::cfg_if! { if #[cfg(target_arch = "wasm32")] { } else { Send + } } 'static,
// {
// task::spawn(f)
// }

/*──────────── timer struct ───────────*/

type Calc = dyn Fn(usize) -> Duration + Send + Sync + 'static;
type Cb = dyn FnMut() + Send + Sync + 'static;

/// Clone‑able timer with a shared retry counter.
#[derive(Clone)]
pub struct Timer {
    tries: Arc<AtomicUsize>,
    calc: Arc<Calc>,
    cb: Arc<Mutex<Box<Cb>>>,
}
impl Debug for Timer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Timer")
            .field("tries", &self.tries.load(Ordering::Relaxed))
            .finish()
    }
}

impl Timer {
    /// Create a new timer.
    pub fn new<C, F>(cb: C, calc: F) -> Self
    where
        C: FnMut() + Send + Sync + 'static,
        F: Fn(usize) -> Duration + Send + Sync + 'static,
    {
        Self {
            tries: Arc::new(AtomicUsize::new(0)),
            calc: Arc::new(calc),
            cb: Arc::new(Mutex::new(Box::new(cb))),
        }
    }

    /// Reset the retry counter.
    pub fn reset(&self) { self.tries.store(0, Ordering::Relaxed); }

    /// Expose current try count (debug/test only).
    pub fn tries(&self) -> usize { self.tries.load(Ordering::Relaxed) }

    /// Sleep once for the base delay (handy for jitter tests).
    pub async fn sleep(&self) { time::sleep((self.calc)(0)).await; }

    /// Schedule the callback after the next back‑off interval.
    pub fn schedule_timeout(&self) -> Handle {
        let tries = self.tries.clone();
        let calc = self.calc.clone();
        let cb = self.cb.clone();

        task::spawn(async move {
            let n = tries.fetch_add(1, Ordering::Relaxed) + 1;
            time::sleep((calc)(n)).await;
            if let Some(mut g) = cb.try_lock() { (g)(); }
        })
    }
}
