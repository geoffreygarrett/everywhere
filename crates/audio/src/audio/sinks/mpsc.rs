//! Treat a Tokio `Sender<Bytes>` as a `PacketSink`.

use bytes::Bytes;
use tokio::sync::mpsc::Sender;

use crate::audio::traits::PacketSink;

pub struct MpscSink(pub Sender<Bytes>);

impl MpscSink {
    pub fn new(tx: Sender<Bytes>) -> Self { Self(tx) }
}

impl PacketSink for MpscSink {
    fn on_packet(&self, p: Bytes) {
        let _ = self.0.try_send(p);   // non‑blocking, never panics in audio thread
    }
}
