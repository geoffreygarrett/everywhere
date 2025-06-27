use core::time::Duration;
use everywhere_runtime::{time, prelude::*};
use everywhere_test::cross_test;

/// Exercises `Rt::timeout`.
#[cross_test]
async fn timeout_behaviour() {
    /* fast path – must succeed */
    let out = time::timeout(
        Duration::from_millis(100),
        async {
            time::sleep(Duration::from_millis(10)).await;
            Ok::<_, ()>(123)
        },
    )
        .await
        .unwrap();
    assert_eq!(out.unwrap(), 123);

    /* slow path – must time‑out */
    let res = time::timeout(
        Duration::from_millis(10),
        async {
            time::sleep(Duration::from_millis(100)).await;
            Ok::<_, ()>(())
        },
    )
        .await;
    assert!(res.is_err());
}
