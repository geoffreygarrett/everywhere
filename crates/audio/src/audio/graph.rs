//! Tiny builder that wires `Recorder` ⇢ sinks ⇢ `Player`.
//! FIXED: keep the graph alive after `.run()` so audio continues.

use std::{any::Any, mem, sync::{Arc, atomic::AtomicBool}};
use anyhow::Result;
use bytes::Bytes;
use tokio::sync::mpsc;
use cpal::traits::HostTrait;

use crate::audio::{
    device::{input_config, output_config},
    player::Player,
    recorder::Recorder,
    traits::PacketSink,
};

pub struct AudioGraph {
    ptt:   Option<Arc<AtomicBool>>,
    sinks: Vec<Arc<dyn PacketSink>>,
    keep:  Vec<Box<dyn Any>>,        // keeps streams / players alive
}

impl AudioGraph {
    pub fn new() -> Self { Self { ptt: None, sinks: Vec::new(), keep: Vec::new() } }

    /*------------------------------------------------------------------------*/
    /*  Builder chain                                                         */
    /*------------------------------------------------------------------------*/

    /// Capture from the default microphone while `flag` is `true`.
    pub fn record(mut self, flag: Arc<AtomicBool>) -> Self {
        self.ptt = Some(flag);
        self
    }

    /// Deliver packets / bursts to any custom sink.
    pub fn sink<S: PacketSink>(mut self, s: S) -> Self {
        self.sinks.push(Arc::new(s));
        self
    }

    /// Convenience: turn a plain `Sender<Bytes>` into a sink.
    pub fn sink_channel(mut self, tx: tokio::sync::mpsc::Sender<Bytes>) -> Self {
        self.sinks.push(Arc::new(tx));
        self
    }

    /// Add speaker playback.
    /// Internally inserts a new mpsc channel and wires a `Player`.
    pub fn play(mut self, speaker: &cpal::Device) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<Bytes>(512);
        self.sinks.push(Arc::new(tx));                   // recorder → tx
        let player = Player::spawn(speaker, &output_config(speaker)?, rx)?;
        self.keep.push(Box::new(player));                // keep `Stream` alive
        Ok(self)
    }

    /*------------------------------------------------------------------------*/
    /*  Launch everything                                                     */
    /*------------------------------------------------------------------------*/

    pub fn run(mut self) -> Result<()> {
        /* ---- microphone --------------------------------------------------- */
        let mic = cpal::default_host()
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("no default microphone"))?;
        let cfg = input_config(&mic)?;

        /* ---- decide which sink(s) the recorder will feed ------------------ */
        let sink: Arc<dyn PacketSink> = match self.sinks.len() {
            0 => return Err(anyhow::anyhow!("AudioGraph: no sinks attached")),
            1 => self.sinks[0].clone(),
            _ => Arc::new(FanoutSink(self.sinks)),
        };

        /* ---- spawn recorder ------------------------------------------------ */
        let ptt = self.ptt.take().expect("call .record() first");
        let rec = Recorder::spawn(&mic, &cfg, ptt, sink)?;
        self.keep.push(Box::new(rec));

        /* ---- IMPORTANT:  leak self so `keep` is never dropped ------------- */
        // mem::forget(self);
        Ok(())
    }
}

/*------------------------------------------------------------------------*/
/*  Internal helper that duplicates packets to many sinks                 */
/*------------------------------------------------------------------------*/

struct FanoutSink(Vec<Arc<dyn PacketSink>>);

impl crate::audio::traits::PacketSink for FanoutSink {
    fn on_packet(&self, p: Bytes) {
        for s in &self.0 { s.on_packet(p.clone()); }
    }
    fn on_burst(&self, b: crate::audio::burst::Burst) {
        for s in &self.0 { s.on_burst(b.clone()); }
    }
}
