//! Send *complete* bursts to a Tokio channel (nothing while recording).

use std::sync::Mutex;

use bytes::Bytes;
use tokio::sync::mpsc::Sender;

use crate::audio::{burst::Burst, traits::PacketSink};

pub struct BurstMpscSink {
    tx: Sender<Bytes>,
    buf: Mutex<Vec<Bytes>>,        // packets for the current burst
}

impl BurstMpscSink {
    pub fn new(tx: Sender<Bytes>) -> Self {
        Self { tx, buf: Mutex::new(Vec::new()) }
    }
}

impl PacketSink for BurstMpscSink {
    /* collect while key is pressed */
    fn on_packet(&self, p: Bytes) {
        self.buf.lock().unwrap().push(p);
    }

    /* key released → flush entire burst FIFO, then clear */
    fn on_burst(&self, _b: Burst) {
        // Swap the Vec out without panicking
        let mut local = Vec::new();
        if let Ok(mut guard) = self.buf.try_lock() {
            std::mem::swap(&mut *guard, &mut local);
        }

        // FIFO flush
        for pkt in local {
            let _ = self.tx.try_send(pkt);
        }
    }
}

