//! Tokio + async‑tungstenite backend (desktop / server).

use super::super::{message::{WsError, WsMessage}, options::WsOptions};
use anyhow::Context;
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use std::{pin::Pin, task::{Context as Cx, Poll}};

/// TLS‑over‑TCP WebSocket.
pub struct WsConnection {
    sink: Pin<Box<dyn Sink<WsMessage, Error=WsError> + Send>>,
    stream: Pin<Box<dyn Stream<Item=Result<WsMessage, WsError>> + Send>>,
}

impl WsConnection {
    /// Backend–internal entry point (called by the public façade in `lib.rs`).
    pub(crate) async fn _connect_backend(url: &str, _opt: &WsOptions) -> Result<Self, WsError> {
        let (ws, _) = connect_async(url).await
            .with_context(|| format!("connect {url}"))?;
        let (sink_raw, stream_raw) = ws.split();

        /* outbound ----------------------------------------------------- */
        let sink = Box::pin(
            sink_raw.with(|m: WsMessage| async move {
                let msg = match m {
                    WsMessage::Text(t) => Message::Text(t),
                    WsMessage::Binary(b) => Message::Binary(b.into()),
                    WsMessage::Close(Some(c)) => Message::Close(Some(
                        async_tungstenite::tungstenite::protocol::CloseFrame {
                            code: c.0.into(),
                            reason: std::borrow::Cow::Owned(c.1),
                        })),
                    WsMessage::Close(None) => Message::Close(None),
                };
                Ok::<_, WsError>(msg)
            })
        );

        /* inbound ------------------------------------------------------ */
        let stream = Box::pin(
            stream_raw.map(|r| {
                r.map(|m| match m {
                    Message::Text(t) => WsMessage::Text(t),
                    Message::Binary(b) => WsMessage::Binary(b.into()),
                    Message::Close(c) => WsMessage::Close(
                        c.map(|f| (f.code.into(), f.reason.into_owned()))
                    ),
                    _ => WsMessage::Close(None),
                }).map_err(WsError::from)
            })
        );

        Ok(Self { sink, stream })
    }
}

/*──── Stream / Sink passthroughs ───────────────────────────────────────*/

impl Stream for WsConnection {
    type Item = Result<WsMessage, WsError>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Cx<'_>)
                 -> Poll<Option<Self::Item>>
    { unsafe { self.get_unchecked_mut().stream.as_mut().poll_next(cx) } }
}
impl Sink<WsMessage> for WsConnection {
    type Error = WsError;
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Cx<'_>) -> Poll<Result<(), Self::Error>>
    { unsafe { self.get_unchecked_mut().sink.as_mut().poll_ready(cx) } }
    fn start_send(self: Pin<&mut Self>, item: WsMessage) -> Result<(), Self::Error>
    { unsafe { self.get_unchecked_mut().sink.as_mut().start_send(item) } }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Cx<'_>) -> Poll<Result<(), Self::Error>>
    { unsafe { self.get_unchecked_mut().sink.as_mut().poll_flush(cx) } }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Cx<'_>) -> Poll<Result<(), Self::Error>>
    { unsafe { self.get_unchecked_mut().sink.as_mut().poll_close(cx) } }
}
impl Unpin for WsConnection {}
