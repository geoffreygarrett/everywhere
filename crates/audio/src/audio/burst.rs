//! One push‑to‑talk session (metadata + packets)

use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Burst {
    pub started_at: DateTime<Utc>,
    pub packets:    Vec<Bytes>,
}

impl Burst {
    pub fn push(&mut self, p: Bytes)       { self.packets.push(p); }
    pub fn is_empty(&self) -> bool         { self.packets.is_empty() }
}
