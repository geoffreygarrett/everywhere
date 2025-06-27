//! Runtime contract + tiny cross‑runtime helpers.
#![allow(async_fn_in_trait)]

use core::{future::Future, time::Duration};

/*───────────────────────────────────────────────────────*
 * 1.  Channel helpers                                   *
 *───────────────────────────────────────────────────────*/

pub trait SendMessage<T> { fn try_send(&self, v: T) -> Result<(), ()>; }

pub trait SenderExt<T>: SendMessage<T> {
    #[inline]
    fn send_owned(&self, v: T) { let _ = self.try_send(v); }
}
impl<T, S: SendMessage<T>> SenderExt<T> for S {}

/* unified ‘recv’ helper (works on every backend) */
pub trait ReceiverExt<T>: Sized {
    async fn recv(&mut self) -> Option<T>;
}

/* Tokio receiver ------------------------------------ */
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
impl<T: Send + 'static> ReceiverExt<T>
for tokio::sync::mpsc::UnboundedReceiver<T>
{
    async fn recv(&mut self) -> Option<T> { self.recv().await }
}

/* futures‑mpsc receiver (browser + WASI) ------------- */
#[cfg(any(feature = "browser", feature = "wasi"))]
impl<T: 'static> ReceiverExt<T>
for futures_channel::mpsc::UnboundedReceiver<T>
{
    async fn recv(&mut self) -> Option<T> {
        use futures_util::StreamExt;
        self.next().await
    }
}

/*───────────────────────────────────────────────────────*
 * 2.  ‘MaybeSend’ – `Send` on native, empty on wasm     *
 *───────────────────────────────────────────────────────*/

#[cfg(not(target_arch = "wasm32"))]
pub use std::marker::Send as MaybeSend;

#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> MaybeSend for T {}

/*───────────────────────────────────────────────────────*
 * 3.  Core `Runtime` trait                              *
 *───────────────────────────────────────────────────────*/

pub trait Runtime: 'static {
    type Sender<T: 'static + MaybeSend>: SenderExt<T> + Clone + 'static;
    type Receiver<T: 'static + MaybeSend>: 'static;
    type JoinHandle: 'static;

    fn spawn<F>(fut: F) -> Self::JoinHandle
    where
        F: Future<Output=()> + MaybeSend + 'static;

    fn channel<T: 'static + MaybeSend>() -> (Self::Sender<T>, Self::Receiver<T>);

    async fn sleep(d: Duration);

    async fn timeout<F, T, E>(d: Duration, fut: F)
                              -> Result<Result<T, E>, ()>
    where
        F: Future<Output=Result<T, E>> + MaybeSend + 'static;

    async fn yield_now();
}
