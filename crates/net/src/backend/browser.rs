//! Browser / Web‑Worker backend – uses **`gloo-net`** `WebSocket`.

use super::super::{message::{WsError, WsMessage}, options::WsOptions};
use anyhow::anyhow;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use gloo_net::websocket::{futures::WebSocket as GlooWs, Message};
use std::{pin::Pin, task::{Context, Poll}};

pub struct WsConnection {
    sink: Pin<Box<dyn Sink<WsMessage, Error=WsError>>>,
    stream: Pin<Box<dyn Stream<Item=Result<WsMessage, WsError>>>>,
}

impl WsConnection {
    pub(crate) async fn _connect_backend(url: &str, opt: &WsOptions) -> Result<Self, WsError> {
        /* choose correct `open*` flavour ------------------------------- */
        let ws = match opt.protocols.len() {
            0 => GlooWs::open(url)?,
            1 => GlooWs::open_with_protocol(url, &opt.protocols[0])?,
            _ => {
                let vec_ref: Vec<&str> = opt.protocols.iter().map(|s| s.as_str()).collect();
                GlooWs::open_with_protocols(url, &vec_ref)?
            }
        };
        let (sink_raw, stream_raw) = ws.split();

        /* outbound ----------------------------------------------------- */
        let sink = Box::pin(
            sink_raw.with(|m: WsMessage| async move {
                match m {
                    WsMessage::Text(t) => Ok(Message::Text(t)),
                    WsMessage::Binary(b) => Ok(Message::Bytes(b.into())),
                    WsMessage::Close(_) => Err(anyhow!(
                        "Browser backend: close via `WebSocket::close()` or drop the handle")),
                }
            })
        );

        /* inbound ------------------------------------------------------ */
        let stream = Box::pin(
            stream_raw.map(|r| {
                r.map(|m| match m {
                    Message::Text(t) => WsMessage::Text(t),
                    Message::Bytes(b) => WsMessage::Binary(b.into()),
                })
                    .map_err(|e| WsError::from(anyhow!("{e:?}")))
            })
        );

        Ok(Self { sink, stream })
    }
}

/*──── passthroughs ────────────────────────────────────────────────────*/

impl Stream for WsConnection {
    type Item = Result<WsMessage, WsError>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
                 -> Poll<Option<Self::Item>>
    { unsafe { self.get_unchecked_mut().stream.as_mut().poll_next(cx) } }
}
impl Sink<WsMessage> for WsConnection {
    type Error = WsError;
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>
    { unsafe { self.get_unchecked_mut().sink.as_mut().poll_ready(cx) } }
    fn start_send(self: Pin<&mut Self>, item: WsMessage) -> Result<(), Self::Error>
    { unsafe { self.get_unchecked_mut().sink.as_mut().start_send(item) } }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>
    { unsafe { self.get_unchecked_mut().sink.as_mut().poll_flush(cx) } }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>
    { unsafe { self.get_unchecked_mut().sink.as_mut().poll_close(cx) } }
}
impl Unpin for WsConnection {}
