//! Blanket traits that make senders / sinks plug‑n‑play.

use bytes::Bytes;

use crate::audio::burst::Burst;

/// Anything that wants Opus **packets** or complete **bursts** implements this.
pub trait PacketSink: Send + Sync + 'static {
    fn on_packet(&self, _pkt: Bytes) {}
    fn on_burst(&self, _b: Burst) {}
}

/* -------------------------------------------------------------------------- */
/*  Ready‑made adapters so most code needs zero boiler‑plate.                 */
/* -------------------------------------------------------------------------- */

impl PacketSink for tokio::sync::mpsc::Sender<Bytes> {
    fn on_packet(&self, p: Bytes) { let _ = self.try_send(p); }
}

#[cfg(feature = "transcribe")]
impl PacketSink for crossbeam_channel::Sender<Bytes> {
    fn on_packet(&self, p: Bytes) { let _ = self.try_send(p); }
}

/// Pull‑model counterpart — **not** used by the graph yet (kept for parity).
pub trait PacketSource: Send + 'static {
    fn try_recv(&mut self) -> Option<Bytes>;
}
