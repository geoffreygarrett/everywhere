//! Hold <Space> → speak → release → playback starts.

use anyhow::Result;
use bytes::Bytes;
use cpal::traits::HostTrait;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use tokio::sync::mpsc;

use cross_audio::{prelude::*, sinks::burst_mpsc::BurstMpscSink};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let _guard = scopeguard::guard((), |_| { let _ = disable_raw_mode(); });

    let host = cpal::default_host();
    let mic = host.default_input_device().unwrap();
    let spk = host.default_output_device().unwrap();

    let ptt = Arc::new(AtomicBool::new(false));

    /* channel between Recorder-side sink and Player */
    let (tx, rx) = mpsc::channel::<Bytes>(512);

    /* build the pipeline */
    let _rec = Recorder::spawn(&mic, &input_config(&mic)?, ptt.clone(), Arc::new(BurstMpscSink::new(tx)))?;
    let _play = Player::spawn(&spk, &output_config(&spk)?, rx)?;

    println!("🎤 {}\n🔈 {}\n—— Hold <Space> to talk — releases play ——", mic.name()?, spk.name()?);

    /* UI */
    let mut evs = EventStream::new();
    while let Some(Ok(Event::Key(k))) = evs.next().await {
        if k.code == KeyCode::Char(' ') && matches!(k.kind, KeyEventKind::Press) {
            let on = !ptt.load(Ordering::Relaxed);
            ptt.store(on, Ordering::Relaxed);
            println!("PTT {}", if on { "ON " } else { "OFF" });
        }
        if k.code == KeyCode::Char('c')
            && k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) { break; }
    }
    Ok(())
}
