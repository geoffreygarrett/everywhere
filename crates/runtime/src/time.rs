use crate::{api::Runtime, Rt};
use core::time::Duration;
use instant::Instant;

/*──────────── sleep ───────────*/

pub async fn sleep(d: Duration) { Rt::sleep(d).await }
pub async fn sleep_with<R: Runtime>(d: Duration) { R::sleep(d).await }

/*──────────── now() ───────────*/

pub fn now_ms() -> u128 {
    Instant::now().elapsed().as_millis()
}
pub fn now() -> Duration {
    Instant::now().elapsed()
}

/// Milliseconds since the Unix epoch **across all runtimes**.
///
/// * Native / WASI → `SystemTime`.
/// * Browser Wasm → `js_sys::Date::now()`.
/// * `no_std` embedded → compile-error unless you disable `epoch` helpers.
pub fn epoch_ms() -> u128 {
    #[cfg(all(target_arch = "wasm32", feature = "browser"))]
    {
        // JS `Date.now()` returns f64 milliseconds since epoch
        js_sys::Date::now() as u128
    }

    #[cfg(all(target_arch = "wasm32", feature = "wasi"))]
    {
        // WASI preview-2 pipes SystemTime; preview-1 still OK in most runtimes
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }

    #[cfg(all(not(target_arch = "wasm32")))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }
}


/*──────────── timeout (default runtime) ───────────*/

// native needs `Send`
#[cfg(not(target_arch = "wasm32"))]
pub async fn timeout<F, T, E>(d: Duration, fut: F)
                              -> Result<Result<T, E>, ()>
where
    F: core::future::Future<Output=Result<T, E>> + Send + 'static,
{ Rt::timeout(d, fut).await }

#[cfg(target_arch = "wasm32")]
pub async fn timeout<F, T, E>(d: Duration, fut: F)
                              -> Result<Result<T, E>, ()>
where
    F: core::future::Future<Output=Result<T, E>> + 'static,
{ Rt::timeout(d, fut).await }

/*──────────── timeout_with (generic runtime) ───────────*/

#[cfg(not(target_arch = "wasm32"))]
pub async fn timeout_with<R, F, T, E>(d: Duration, fut: F)
                                      -> Result<Result<T, E>, ()>
where
    R: Runtime,
    F: core::future::Future<Output=Result<T, E>> + Send + 'static,
{ R::timeout(d, fut).await }

#[cfg(target_arch = "wasm32")]
pub async fn timeout_with<R, F, T, E>(d: Duration, fut: F)
                                      -> Result<Result<T, E>, ()>
where
    R: Runtime,
    F: core::future::Future<Output=Result<T, E>> + 'static,
{ R::timeout(d, fut).await }
