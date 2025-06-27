use crate::{api::Runtime, Rt};
use core::time::Duration;

/*──────────── sleep ───────────*/

pub async fn sleep(d: Duration) { Rt::sleep(d).await }
pub async fn sleep_with<R: Runtime>(d: Duration) { R::sleep(d).await }

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
