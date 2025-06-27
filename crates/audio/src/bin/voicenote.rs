//! Hold Space – record – release → saves a timestamped .wav (needs --features file)

use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use cross_audio::{prelude::*, sinks::wav::WavSink};

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let _raw = scopeguard::guard((), |_| { let _ = disable_raw_mode(); });

    let host = cpal::default_host();
    let mic = host.default_input_device().unwrap();

    let ptt = Arc::new(AtomicBool::new(false));
    let sink = Arc::new(WavSink::new(format!("note-{}.wav", Local::now().format("%Y%m%d_%H%M%S")))?);

    let _rec = Recorder::spawn(&mic, &input_config(&mic)?, ptt.clone(), sink)?;

    println!("🎙  Hold <Space> to record (Ctrl‑C quits)");
    let mut evs = EventStream::new();
    while let Some(Ok(Event::Key(k))) = evs.next().await {
        if k.code == KeyCode::Char(' ') && matches!(k.kind, KeyEventKind::Press) {
            let on = !ptt.load(Ordering::Relaxed);
            ptt.store(on, Ordering::Relaxed);
            println!("{}", if on { "● REC…" } else { "💾 saved" });
        }
        if k.code == KeyCode::Char('c')
            && k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            break;
        }
    }
    Ok(())
}
