use core::{sync::atomic::{AtomicU8, Ordering}, time::Duration};
use everywhere_runtime::{prelude::*, task, time};
use everywhere_test::cross_test;
use std::sync::Arc;

/// Spawn a task, sleep, and check side effects.
#[cross_test]
async fn spawn_and_sleep() {
    let flag = Arc::new(AtomicU8::new(0));
    let flag2 = flag.clone();

    task::spawn(async move {
        time::sleep(Duration::from_millis(50)).await;
        flag2.store(1, Ordering::SeqCst);
    });

    time::sleep(Duration::from_millis(120)).await;
    assert_eq!(flag.load(Ordering::SeqCst), 1);
}
