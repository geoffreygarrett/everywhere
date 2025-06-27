//! Shared message + trait aliases (no more overlapping impls).
use bytes::Bytes;
use core::fmt;
use futures_util::{Sink, Stream};

/// Cheap‑to‑clone WebSocket message.
#[derive(Clone, PartialEq, Eq)]
pub enum WsMessage {

    Text(String),
    Binary(Bytes),
    Close(Option<(u16, String)>),
}
impl fmt::Debug for WsMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(t)   => f.debug_tuple("Text").field(t).finish(),
            Self::Binary(b) => f.debug_tuple("Binary").field(&b.len()).finish(),
            Self::Close(c)  => f.debug_tuple("Close").field(c).finish(),
        }
    }
}

/// Back‑ends bubble up anything as `anyhow::Error`.
pub type WsError = anyhow::Error;

/*──────────────────────── trait alias ────────────────────────*/
#[cfg(not(target_arch = "wasm32"))]
pub trait WebSocketLike:
Stream<Item = Result<WsMessage, WsError>>
+ Sink<WsMessage, Error = WsError>
+ Unpin + Send + 'static {}
#[cfg(not(target_arch = "wasm32"))]
impl<T> WebSocketLike for T where
    T: Stream<Item = Result<WsMessage, WsError>>
    + Sink<WsMessage, Error = WsError>
    + Unpin + Send + 'static {}

#[cfg(target_arch = "wasm32")]
pub trait WebSocketLike:
Stream<Item = Result<WsMessage, WsError>>
+ Sink<WsMessage, Error = WsError>
+ Unpin + 'static {}
#[cfg(target_arch = "wasm32")]
impl<T> WebSocketLike for T where
    T: Stream<Item = Result<WsMessage, WsError>>
    + Sink<WsMessage, Error = WsError>
    + Unpin + 'static {}
