//! Tokio (native host OS)

use crate::api::*;
use std::time::Duration;
use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};

pub struct TokioRt;

/*───────────────── implementation ───────────────────*/

impl Runtime for TokioRt {
    type Sender<T: 'static + MaybeSend> = mpsc::UnboundedSender<T>;
    type Receiver<T: 'static + MaybeSend> = mpsc::UnboundedReceiver<T>;
    type JoinHandle = JoinHandle<()>;

    fn spawn<F>(fut: F) -> Self::JoinHandle
    where
        F: core::future::Future<Output=()> + MaybeSend + 'static,
    {
        task::spawn(fut)
    }

    fn channel<T: 'static + MaybeSend>()
        -> (Self::Sender<T>, Self::Receiver<T>)
    { mpsc::unbounded_channel() }

    async fn sleep(d: Duration) { tokio::time::sleep(d).await }

    async fn timeout<F, T, E>(d: Duration, fut: F)
                              -> Result<Result<T, E>, ()>
    where
        F: core::future::Future<Output=Result<T, E>> + MaybeSend + 'static,
    {
        tokio::time::timeout(d, fut).await.map_err(|_| ())
    }

    async fn yield_now() { task::yield_now().await }
}

/*───────────────── sender helper ───────────────────*/

impl<T: 'static + MaybeSend> SendMessage<T> for mpsc::UnboundedSender<T> {
    fn try_send(&self, v: T) -> Result<(), ()> { self.send(v).map_err(|_| ()) }
}
