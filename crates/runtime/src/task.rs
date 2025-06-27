//! High-level **spawn / channel façade** (+ generic helpers)
//
//! Works on all three runtime flavours (native / browser / WASI).
//!
//! * `spawn(…)`, `channel(…)`                → use crate-level [`Rt`].
//! * `spawn_with::<MyRt>(…)`, `channel_with::<MyRt, _>(…)`
//!                                             → pick any `R: Runtime`.

use crate::{api::Runtime, Rt};

/*──────────────────────── spawn ───────────────────────────────*/

// ─ default runtime ─
#[cfg(not(target_arch = "wasm32"))]           // multithread
pub fn spawn<F>(fut: F) -> <Rt as Runtime>::JoinHandle
where
    F: core::future::Future<Output = ()> + Send + 'static,
{
    Rt::spawn(fut)
}
#[cfg(target_arch = "wasm32")]                // single-thread
pub fn spawn<F>(fut: F) -> <Rt as Runtime>::JoinHandle
where
    F: core::future::Future<Output = ()> + 'static,
{
    Rt::spawn(fut)
}

// ─ generic runtime ─
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_with<R, F>(fut: F) -> <R as Runtime>::JoinHandle
where
    R: Runtime,
    F: core::future::Future<Output = ()> + Send + 'static,
{
    R::spawn(fut)
}
#[cfg(target_arch = "wasm32")]
pub fn spawn_with<R, F>(fut: F) -> <R as Runtime>::JoinHandle
where
    R: Runtime,
    F: core::future::Future<Output = ()> + 'static,
{
    R::spawn(fut)
}

/*──────────────────────── channels ───────────────────────────*/

// default runtime
#[cfg(not(target_arch = "wasm32"))]
pub fn channel<T>() -> (
    <Rt as Runtime>::Sender<T>,
    <Rt as Runtime>::Receiver<T>,
)
where
    T: Send + 'static,                // native needs `Send`
{
    Rt::channel()
}
#[cfg(target_arch = "wasm32")]
pub fn channel<T>() -> (
    <Rt as Runtime>::Sender<T>,
    <Rt as Runtime>::Receiver<T>,
)
where
    T: 'static,
{
    Rt::channel()
}

// generic runtime
#[cfg(not(target_arch = "wasm32"))]
pub fn channel_with<R, T>() -> (
    <R as Runtime>::Sender<T>,
    <R as Runtime>::Receiver<T>,
)
where
    R: Runtime,
    T: Send + 'static,                // native needs `Send`
{
    R::channel()
}
#[cfg(target_arch = "wasm32")]
pub fn channel_with<R, T>() -> (
    <R as Runtime>::Sender<T>,
    <R as Runtime>::Receiver<T>,
)
where
    R: Runtime,
    T: 'static,
{
    R::channel()
}

/*──────────────────────── yield helpers ─────────────────────*/

pub async fn yield_now() { Rt::yield_now().await }

pub async fn yield_now_with<R: Runtime>() { R::yield_now().await }

/*──────────────────────── convenience re-export ─────────────*/

pub use crate::SenderExt;   // lets users call `tx.send_owned(v)`
