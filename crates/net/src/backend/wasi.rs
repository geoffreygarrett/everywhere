//! soketto client (wasm32‑wasi or other pure‑WASI target).

use super::super::{message::{WsError, WsMessage}, options::WsOptions};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use soketto::{data::Message, handshake::Client};
use std::{pin::Pin, task::{Context, Poll}};

pub struct WsConnection {
    sink:   Pin<Box<dyn Sink  <WsMessage, Error = WsError>>>,
    stream: Pin<Box<dyn Stream<Item = Result<WsMessage, WsError>>>>,
}

impl WsConnection {
    pub(crate) async fn _connect_backend(url: &str, _opt: &WsOptions) -> Result<Self, WsError> {
        let mut client = Client::connect(url).await?;
        client.handshake().await?;
        let (sender, receiver) = client.into_builder().finish();

        let sink = Box::pin(sender.with(|m: WsMessage| async move {
            Ok(match m {
                WsMessage::Text(t)   => Message::text(t.into()),
                WsMessage::Binary(b) => Message::binary(b.into()),
                WsMessage::Close(_)  => Message::close(None),
            })
        }));

        let stream = Box::pin(
            receiver.map(|r| {
                r.map(|m| match m {
                    Message::Text(t)   => WsMessage::Text(String::from_utf8_lossy(&t).into()),
                    Message::Binary(b) => WsMessage::Binary(b.into()),
                    Message::Close(_)  => WsMessage::Close(None),
                }).map_err(WsError::from)
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
    fn poll_ready (self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>
    { unsafe { self.get_unchecked_mut().sink.as_mut().poll_ready (cx) } }
    fn start_send(self: Pin<&mut Self>, item: WsMessage) -> Result<(), Self::Error>
    { unsafe { self.get_unchecked_mut().sink.as_mut().start_send(item) } }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>
    { unsafe { self.get_unchecked_mut().sink.as_mut().poll_flush(cx) } }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>
    { unsafe { self.get_unchecked_mut().sink.as_mut().poll_close(cx) } }
}
impl Unpin for WsConnection {}
