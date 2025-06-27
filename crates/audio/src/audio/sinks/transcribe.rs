//! Real-time Whisper-rs speech-to-text sink  (enable with `--features transcribe`)
#![cfg(feature = "transcribe")]

use std::{
    path::Path,
    sync::Arc,
    thread,
    time::Duration,
};

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use crate::audio::{codec::OpusDecoder, traits::PacketSink};

/// How often we flush partial recognition results (every N decoded packets).
const FLUSH_EVERY_PACKETS: usize = 25;

/// A sink that accepts raw Opus **packets**, decodes them to PCM and feeds
/// them to a background Whisper thread.
/// The thread prints the final transcript when the burst finishes.
pub struct WhisperSink {
    tx: Sender<Bytes>,
}

impl WhisperSink {
    /// Spawn a background worker.
    /// `model_path` = path to a `.bin` GGML model (base, small, …).
    pub fn spawn<P: AsRef<Path>>(model_path: P) -> anyhow::Result<Arc<Self>> {
        // Channel between Recorder thread (audio) and Whisper thread (blocking).
        let (tx, rx) = crossbeam_channel::unbounded::<Bytes>();

        // Heavy initialisation happens *off* the audio thread.
        let model_path = model_path.as_ref().to_path_buf();
        thread::spawn(move || run_worker(rx, model_path));

        Ok(Arc::new(Self { tx }))
    }
}

impl PacketSink for WhisperSink {
    fn on_packet(&self, pkt: Bytes) {
        // Non-blocking, never panics – drops packet if the queue is full.
        let _ = self.tx.try_send(pkt);
    }
}

/* ------------------------------------------------------------------------- */
/*  Worker thread                                                            */
/* ------------------------------------------------------------------------- */

fn run_worker(rx: Receiver<Bytes>, model_path: std::path::PathBuf) {
    // 1) Initialise Whisper context once.
    let ctx = WhisperContext::new(&model_path)
        .unwrap_or_else(|e| panic!("Whisper-rs failed to open model: {e}"));

    let mut dec = OpusDecoder::new().expect("Opus decoder init failed");
    let mut pcm = Vec::<f32>::new();      // grows as needed
    let mut cnt = 0usize;                 // packets since last flush
    let mut seg = 0usize;                 // next segment index to print

    // 2) Process packets forever (or until the program quits).
    loop {
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(pkt) => {
                pcm.extend(dec.decode(&pkt).expect("Opus decode failed"));
                cnt += 1;

                // feed partial audio every N packets to reduce latency
                if cnt >= FLUSH_EVERY_PACKETS {
                    flush_whisper(&ctx, &pcm, &mut seg);
                    cnt = 0;
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // burst likely ended – finish remaining audio, then reset.
                if !pcm.is_empty() {
                    flush_whisper(&ctx, &pcm, &mut seg);
                    println!("📝 final: {}", collect_segments(&ctx, seg));
                    pcm.clear();
                    seg = 0;
                    cnt = 0;
                }
            }
            Err(_) => break,   // channel closed → exit thread
        }
    }
}

/* ------------------------------------------------------------------------- */
/*  Helper functions                                                         */
/* ------------------------------------------------------------------------- */

/// Run Whisper on the accumulated PCM; keeps internal context alive.
fn flush_whisper(ctx: &WhisperContext, pcm: &[f32], next: &mut usize) {
    // Whisper-rs needs mutable context state.
    let mut state = ctx.create_state().expect("create_state");

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(num_cpus::get() as i32);
    // Split strategy: we stream in one chunk, so disable speed optimisations
    // (keeps latency low for small chunks).
    params.set_stride(0, 0);
    params.set_translate(false);
    params.set_language(Some("en"));

    state
        .full(params, pcm)
        .expect("whisper full failed");

    // Print any new interim segments.
    while *next < state.full_n_segments() {
        let s = state.full_get_segment_text(*next).unwrap_or_default();
        if !s.trim().is_empty() {
            println!("… {}", s);
        }
        *next += 1;
    }
}

/// Collect all segments from `start_idx` to the end.
fn collect_segments(ctx: &WhisperContext, start_idx: usize) -> String {
    let mut state = ctx.create_state().unwrap();
    let n = state.full_n_segments();
    (start_idx..n)
        .filter_map(|i| state.full_get_segment_text(i).ok())
        .collect::<Vec<_>>()
        .join(" ")
}
