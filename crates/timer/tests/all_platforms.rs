//! Comprehensive tests for `cross‑timer` on every runtime.

use core::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use std::sync::Arc;

use everywhere_test::cross_test;
use everywhere_timer::Timer;
use everywhere_runtime::time;

/// Convenience awaitable sleep.
async fn wait(ms: u64) { time::sleep(Duration::from_millis(ms)).await; }

/*──────────── single‑shot ───────────*/

#[cross_test]
async fn fires_exactly_once() {
    let hit = Arc::new(AtomicUsize::new(0));
    let h   = hit.clone();

    Timer::new(move || { h.fetch_add(1, Ordering::Relaxed); },
               |_| Duration::from_millis(5))
        .schedule_timeout();

    wait(10).await;
    assert_eq!(hit.load(Ordering::Relaxed), 1);
}

/*──────────── reset() ───────────*/

#[cross_test]
async fn reset_clears_counter() {
    let t = Timer::new(|| {}, |_| Duration::ZERO);
    t.schedule_timeout();
    t.reset();
    assert_eq!(t.tries(), 0);
}

/*──────────── multiple schedules ───────────*/

#[cross_test]
async fn tries_and_hits_match_multiple_calls() {
    let hit = Arc::new(AtomicUsize::new(0));
    let t   = {
        let h = hit.clone();
        Timer::new(move || { h.fetch_add(1, Ordering::Relaxed); },
                   |_| Duration::from_millis(1))
    };

    for _ in 0..3 { t.schedule_timeout(); }

    assert_eq!(t.tries(), 3);     // counter increments immediately
    wait(10).await;
    assert_eq!(hit.load(Ordering::Relaxed), 3);
}

/*──────────── clone shares state ───────────*/

#[cross_test]
async fn clone_shares_counter() {
    let t1 = Timer::new(|| {}, |_| Duration::from_millis(1));
    let t2 = t1.clone();

    t1.schedule_timeout();
    t2.schedule_timeout();
    wait(5).await;

    assert_eq!(t1.tries(), 2);
    assert_eq!(t2.tries(), 2);
}

/*──────────── base sleep helper ───────────*/

#[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
#[cross_test]
async fn base_sleep_waits_long_enough() {
    let base = Duration::from_millis(4);
    let t    = Timer::new(|| {}, move |_| base);

    let before = std::time::Instant::now();
    t.sleep().await;
    assert!(before.elapsed() >= base);
}

#[cfg(all(feature = "browser", target_arch = "wasm32"))]
#[cross_test]
async fn base_sleep_does_not_panic() {
    let t = Timer::new(|| {}, |_| Duration::from_millis(4));
    t.sleep().await;          // Instant::now() is unavailable in wasm32
}

/*──────────── monotonic back‑off ───────────*/

#[cross_test]
async fn backoff_is_monotonic() {
    use std::sync::Mutex;
    let stamps = Arc::new(Mutex::new(Vec::<u128>::new()));

    let timer = {
        let s = stamps.clone();
        Timer::new(move || s.lock().unwrap().push(now_ms()),
                   |n| Duration::from_millis(2 * n as u64))
    };

    for _ in 0..3 { timer.schedule_timeout(); }
    wait(15).await;

    let v = stamps.lock().unwrap();
    assert!(v[0] < v[1] && v[1] < v[2]);
}
fn now_ms() -> u128 { std::time::Instant::now().elapsed().as_millis() }

/*──────────── JoinHandle (native only) ───────────*/

#[cfg(feature = "native")]
#[cross_test]
async fn joinhandle_completes_ok() {
    let done = Arc::new(AtomicUsize::new(0));
    let d    = done.clone();

    let jh = Timer::new(move || { d.store(1, Ordering::Relaxed); },
                        |_| Duration::from_millis(1))
        .schedule_timeout();

    jh.await.unwrap();
    assert_eq!(done.load(Ordering::Relaxed), 1);
}

/*──────────── Debug impl ───────────*/

#[cross_test]
async fn debug_fmt_contains_tries() {
    let t = Timer::new(|| {}, |_| Duration::ZERO);
    t.schedule_timeout();
    assert!(format!("{t:?}").contains("tries"));
}
