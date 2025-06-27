//! Small, cross‑platform set of connection tweaks.

use std::time::Duration;

/// Builder for run‑time settings.
///
/// Back‑ends silently ignore knobs they cannot honour.
#[derive(Clone, Debug, Default)]
pub struct WsOptions {
    pub protocols:         Vec<String>,
    pub max_frame_size:    Option<usize>,       // native / WASI
    pub max_message_size:  Option<usize>,       // native / WASI
    pub ping_interval:     Option<Duration>,    // native / WASI
}

impl WsOptions {
    pub fn new() -> Self { Self::default() }

    /*── fluent helpers ────────────────────────────────────────────────*/
    pub fn protocol(mut self, p: impl Into<String>) -> Self {
        self.protocols.push(p.into()); self
    }
    pub fn max_frame_size   (mut self, n: usize   ) -> Self { self.max_frame_size   = Some(n); self }
    pub fn max_message_size (mut self, n: usize   ) -> Self { self.max_message_size = Some(n); self }
    pub fn ping_interval    (mut self, d: Duration) -> Self { self.ping_interval    = Some(d); self }
}
