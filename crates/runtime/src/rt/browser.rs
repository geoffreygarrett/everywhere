//! wasm‑bindgen (browser / worker)

use crate::api::*;
use core::time::Duration;
use futures_channel::mpsc;
use futures_util::future::{select, Either};
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen_futures::spawn_local;

async fn js_yield() { TimeoutFuture::new(0).await }

pub struct BrowserRt;

/*───────────────── implementation ───────────────────*/

impl Runtime for BrowserRt {
    type Sender<T: 'static + MaybeSend> = mpsc::UnboundedSender<T>;
    type Receiver<T: 'static + MaybeSend> = mpsc::UnboundedReceiver<T>;
    type JoinHandle = ();

    fn spawn<F>(fut: F) -> Self::JoinHandle
    where
        F: core::future::Future<Output=()> + MaybeSend + 'static,
    {
        spawn_local(fut)
    }

    fn channel<T: 'static + MaybeSend>()
        -> (Self::Sender<T>, Self::Receiver<T>)
    { mpsc::unbounded() }

    async fn sleep(d: Duration)
    { TimeoutFuture::new(d.as_millis() as u32).await }

    async fn yield_now() { js_yield().await }

    async fn timeout<F, T, E>(d: Duration, fut: F)
                              -> Result<Result<T, E>, ()>
    where
        F: core::future::Future<Output=Result<T, E>> + MaybeSend + 'static,
    {
        let timer = TimeoutFuture::new(d.as_millis() as u32);
        futures_util::pin_mut!(fut, timer);
        Ok(match select(fut, timer).await {
            Either::Left((out, _)) => out,
            Either::Right((_, _)) => return Err(()),
        })
    }
}

/*───────────────── sender helper ───────────────────*/

impl<T: 'static + MaybeSend> SendMessage<T> for mpsc::UnboundedSender<T> {
    fn try_send(&self, v: T) -> Result<(), ()> { self.unbounded_send(v).map_err(|_| ()) }
}
