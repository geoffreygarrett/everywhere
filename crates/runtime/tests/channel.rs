use core::time::Duration;
use everywhere_runtime::{task, time, prelude::*};
use everywhere_test::cross_test;
use futures_util::StreamExt;

/// Simple round‑trip channel test.
#[cross_test]
async fn channel_roundtrip() {
    let (tx, mut rx) = task::channel::<u32>();

    task::spawn(async move { tx.send_owned(42) });

    let v = rx.recv().await.unwrap();

    assert_eq!(v, 42);

    /* smoke‑test the timeout helper too */
    let _ = time::timeout(Duration::from_millis(20), async { Ok::<(), ()>(()) }).await;
}
