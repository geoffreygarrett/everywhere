//! Push‑to‑talk loop‑back demo (Space toggles TX / RX)
//! — stores microphone frames while PTT is ON, plays them AFTER release —
use anyhow::Result;
use audiopus::{coder, packet::Packet, Application, Channels, MutSignals, SampleRate};
use bytes::Bytes;
use cpal::{traits::*, BufferSize, SampleFormat, SampleRate as CpalSR, Stream, StreamConfig};
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Once,
    },
};
use tokio::sync::mpsc;

/*── constants ────────────────────────────────────────────────────*/
const OPUS_SR: u32          = 48_000;        // fixed 48 kHz
const FRAME_SAMPLES: usize  = 960;           // 20 ms @ 48 kHz (Opus canonical):contentReference[oaicite:3]{index=3}
const QUEUE: usize          = 512;           // encoded packets buffered

/*── global flags / counters ──────────────────────────────────────*/
static PTT:    AtomicBool = AtomicBool::new(false);    // push‑to‑talk
static SENT:   AtomicU64  = AtomicU64::new(0);         // frames delivered to player
static PLAYED: AtomicU64  = AtomicU64::new(0);         // frames rendered
static FIRST:  Once       = Once::new();               // first‑buffer marker

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    /*── terminal raw‑mode so we get key‑up events ─────────────────────*/
    enable_raw_mode()?;
    let _raw_guard = scopeguard::guard((), |_| { let _ = disable_raw_mode(); });

    /*── devices & configs ─────────────────────────────────────────────*/
    let host = cpal::default_host();
    let mic  = host.default_input_device().expect("no microphone");
    let spk  = host.default_output_device().expect("no speakers");
    println!("🎤  {}", mic.name()?);
    println!("🔈  {}", spk.name()?);

    let in_cfg  = cfg_input (&mic )?;
    let out_cfg = cfg_output(&spk )?;
    println!("✅  48 kHz f32 – <Space> to talk, <Ctrl‑C> quits");

    /*── Opus coder @ 48 kHz / mono ────────────────────────────────────*/
    let enc = coder::Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip)?;
    let mut dec = coder::Decoder::new(SampleRate::Hz48000, Channels::Mono)?;

    /*── channel carrying encoded frames ───────────────────────────────*/
    let (tx, mut rx) = mpsc::channel::<Bytes>(QUEUE);

    /*── heartbeat (once per second) ───────────────────────────────────*/
    {
        let tx_stats = tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                let s = SENT.swap  (0, Ordering::Relaxed);
                let p = PLAYED.swap(0, Ordering::Relaxed);
                eprintln!("⏱️  {s} sent | {p} played | queue {}", QUEUE - tx_stats.capacity());
            }
        });
    }

    /*── 1️⃣  Space toggles PTT ----------------------------------------*/
    tokio::spawn(async {
        let mut evs = EventStream::new();                // crossterm async stream:contentReference[oaicite:4]{index=4}
        while let Some(Ok(Event::Key(k))) = evs.next().await {
            if k.code == KeyCode::Char(' ') && matches!(k.kind, KeyEventKind::Press) {
                let on = !PTT.load(Ordering::Relaxed);
                PTT.store(on, Ordering::Relaxed);
                eprintln!("🔊  PTT {}", if on { "ON" } else { "OFF" });
            }
        }
    });

    /*── 2️⃣  Recorder — encodes frames & stores them while PTT is ON ──*/
    let capture: Stream = {
        let mut frame_i16: Vec<i16>  = Vec::with_capacity(FRAME_SAMPLES);
        let mut session  : Vec<Bytes> = Vec::new();      // encoded frames for this TX burst
        let tx = tx.clone();

        mic.build_input_stream(
            &in_cfg,
            move |data: &[f32], _| {
                FIRST.call_once(|| eprintln!("✅ first buffer ({})", data.len()));
                let ptt = PTT.load(Ordering::Relaxed);

                // if key released → flush buffered frames to playback queue
                if !ptt && !session.is_empty() {
                    for pkt in session.drain(..) {
                        let _ = tx.blocking_send(pkt);
                        SENT.fetch_add(1, Ordering::Relaxed);
                    }
                    frame_i16.clear();
                    return;
                }

                if ptt {
                    for &s in data {
                        frame_i16.push((s * i16::MAX as f32) as i16);
                        if frame_i16.len() == FRAME_SAMPLES {
                            let mut buf = [0u8; 400];
                            if let Ok(n) = enc.encode(&frame_i16, &mut buf) {
                                session.push(Bytes::copy_from_slice(&buf[..n])); // store
                            }
                            frame_i16.clear();
                        }
                    }
                }
            },
            |e| eprintln!("❌ capture: {e}"),
            None,
        )?
    };
    capture.play()?;

    /*── 3️⃣  Player — pulls from queue, smooths timing with ring‑buf ──*/
    let playback: Stream = spk.build_output_stream(
        &out_cfg,
        move |out: &mut [f32], _| {
            // static ring buffer living for entire program run
            static mut RING: Option<VecDeque<f32>> = None;
            let ring = unsafe { RING.get_or_insert_with(VecDeque::new) };

            /* fill ring until we have enough samples for this callback */
            while ring.len() < out.len() {
                match rx.try_recv() {
                    Ok(frame) => {
                        if let Ok(pkt) = Packet::try_from(frame.as_ref()) {
                            let mut pcm = vec![0i16; FRAME_SAMPLES];
                            let mut sig = MutSignals::try_from(&mut pcm[..]).unwrap();    // safe conversion:contentReference[oaicite:5]{index=5}
                            let n = dec.decode(Some(pkt), sig, false).unwrap_or(0);
                            for &s in &pcm[..n] {
                                ring.push_back(s as f32 / i16::MAX as f32);
                            }
                            PLAYED.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(_) => break, // channel empty
                }
            }

            /* transfer to CPAL’s stereo buffer (interleaved L/R) */
            for frame in out.chunks_exact_mut(2) {
                let sample = ring.pop_front().unwrap_or(0.0);
                frame[0] = sample;   // L
                frame[1] = sample;   // R
            }
        },
        |e| eprintln!("❌ playback: {e}"),
        None,
    )?;
    playback.play()?;

    tokio::signal::ctrl_c().await?;
    drop((capture, playback));
    Ok(())
}

/*── device helpers ──────────────────────────────────────────────────*/
fn cfg_input(dev: &cpal::Device) -> Result<StreamConfig> {
    let sup = dev.supported_input_configs()?
        .find(|c| c.channels() == 1
            && c.sample_format() == SampleFormat::F32
            && c.min_sample_rate().0 <= OPUS_SR
            && c.max_sample_rate().0 >= OPUS_SR)           // CPAL capability inspect:contentReference[oaicite:6]{index=6}
        .ok_or_else(|| anyhow::anyhow!("mic lacks 48 kHz mono‑f32"))?;

    let mut cfg = sup.with_sample_rate(CpalSR(OPUS_SR)).config();
    cfg.buffer_size = BufferSize::Fixed(FRAME_SAMPLES as u32);     // explicit 20 ms block
    Ok(cfg)
}

fn cfg_output(dev: &cpal::Device) -> Result<StreamConfig> {
    let sup = dev.supported_output_configs()?
        .find(|c| c.channels() == 2
            && c.sample_format() == SampleFormat::F32
            && c.min_sample_rate().0 <= OPUS_SR
            && c.max_sample_rate().0 >= OPUS_SR)
        .ok_or_else(|| anyhow::anyhow!("speaker lacks 48 kHz stereo‑f32"))?;

    let mut cfg = sup.with_sample_rate(CpalSR(OPUS_SR)).config();
    cfg.buffer_size = BufferSize::Fixed(FRAME_SAMPLES as u32);
    Ok(cfg)
}
