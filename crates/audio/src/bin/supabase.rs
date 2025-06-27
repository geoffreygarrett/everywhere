//! Send *finished* bursts to a Supabase Realtime “broadcast” channel
//! and/or receive bursts from that channel.
//!
//! Usage (sender side):
//!
//!     let chan  = client.channel("chat");
//!     chan.subscribe(Some(jwt)).await?;
//!     let sink  = SupabaseSink::new(chan.clone());
//!     Recorder::spawn(mic, cfg, ptt, sink.into())?;
//!
//! Usage (receiver side):
//!
//!     let (tx, rx) = mpsc::channel::<Bytes>(512);   // Player input
//!     SupabaseSource::attach(chan.clone(), tx).await?;
//!     Player::spawn(spk, cfg, rx)?;

use std::{sync::Arc, time::Duration};

use base64::{engine::general_purpose::STANDARD_NO_PAD as B64, Engine};
use bytes::Bytes;
use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use crate::audio::{burst::Burst, traits::PacketSink};

use super::super::supabase::realtime_channel::RealtimeChannel; // <- your helper

/*---------------------------------------------------------------------------
`SupabaseSink` – called by the audio thread
---------------------------------------------------------------------------*/

pub struct SupabaseSink {
    tx: Sender<Burst>,                     // cross‑beam (SPSC, non‑blocking)
}

impl SupabaseSink {
    pub fn new(chan: RealtimeChannel) -> Arc<Self> {
        let (tx, rx) = crossbeam_channel::unbounded::<Burst>();

        // background task: pull bursts → broadcast
        tokio::spawn(async move {
            while let Ok(burst) = rx.recv() {
                let payload = EncodedBurst::from(burst);
                // ignore errors (e.g. connection drop); user can inspect logs
                let _ = chan
                    .broadcast("opus_burst", payload)
                    .await;
            }
        });

        Arc::new(Self { tx })
    }
}

impl PacketSink for SupabaseSink {
    fn on_burst(&self, b: Burst) {
        let _ = self.tx.try_send(b);   // never blocks inside CPAL callback
    }
}

/*---------------------------------------------------------------------------
`SupabaseSource` – listening side
---------------------------------------------------------------------------*/

pub struct SupabaseSource;

impl SupabaseSource {
    /// Subscribes to `"opus_burst"` events on `chan` and pushes the contained
    /// Opus packets into `dst`. Returns when the channel is closed.
    pub async fn attach(
        mut chan: RealtimeChannel,
        dst: mpsc::Sender<Bytes>,
    ) -> anyhow::Result<()> {
        use phoenix_realtime::supabase::models::Broadcast;

        // 1) register handler
        chan.on::<Broadcast<EncodedBurst>, _>(move |bcast| {
            let EncodedBurst { packets, .. } = bcast.payload;
            // fire‑and‑forget: send every packet to the player queue
            for p in packets {
                // ignore queue overflow (player will under‑run at worst)
                let _ = dst.try_send(Bytes::from(B64.decode(p).unwrap()));
            }
        });

        // 2) keep the stream alive in a task so `.events()` is continuously polled
        tokio::spawn(async move {
            let mut evs = chan.events();
            while evs.next().await.is_some() {}
        });

        Ok(())
    }
}

/*---------------------------------------------------------------------------
Helper: JSON payload <‑‑> Rust
---------------------------------------------------------------------------*/

#[derive(Serialize, Deserialize)]
struct EncodedBurst {
    #[serde(with = "chrono::serde::ts_seconds")]
    started_at: chrono::DateTime<chrono::Utc>,
    /// Base‑64 (URL‑safe, no padding) encoded Opus packets
    packets: Vec<String>,
}

impl From<Burst> for EncodedBurst {
    fn from(b: Burst) -> Self {
        Self {
            started_at: b.started_at,
            packets: b
                .packets
                .into_iter()
                .map(|p| B64.encode(&p))
                .collect(),
        }
    }
}
