//! Capture microphone → assemble exact 20 ms frames → Opus encode → emit.

use anyhow::Result;
use bytes::Bytes;
use cpal::{traits::*, Stream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::audio::{
    burst::Burst,
    codec::OpusEncoder,
    device::FRAME_SAMPLES,
    frame::Frame,
    traits::PacketSink,
};

pub struct Recorder {
    _stream: Stream,    // kept alive for the lifetime of self
}

impl Recorder {
    pub fn spawn(
        dev:  &cpal::Device,
        cfg:  &cpal::StreamConfig,
        ptt:  Arc<AtomicBool>,
        sink: Arc<dyn PacketSink>,
    ) -> Result<Self> {
        /* ------------------------------------------------------------------ */
        /* Persistent state captured by the CPAL callback                      */
        /* ------------------------------------------------------------------ */
        let mut enc     = OpusEncoder::new()?;                 // Opus @ 48 kHz mono
        let mut frame   = Frame::<FRAME_SAMPLES>::default();   // 960‑sample staging buf
        let mut filled  = 0usize;                              // how many samples in `frame`
        let mut burst   = Burst::default();                    // accumulates packets

        /* ------------------------------------------------------------------ */
        /* CPAL input stream                                                  */
        /* ------------------------------------------------------------------ */
        let stream = dev.build_input_stream(
            cfg,
            move |input: &[f32], _| {
                let pressed = ptt.load(Ordering::Relaxed);

                /* ---------------- key released → flush the finished burst ---------------- */
                if !pressed && !burst.is_empty() {
                    sink.on_burst(std::mem::take(&mut burst));
                    filled = 0;                               // drop any partial frame
                }

                /* ---------------- key held → gather samples & encode -------------------- */
                if pressed {
                    for &sample in input {
                        frame[filled] = (sample * i16::MAX as f32) as i16;
                        filled += 1;

                        if filled == FRAME_SAMPLES {
                            let pkt_vec = enc.encode(&frame[..]).unwrap();
                            let pkt     = Bytes::copy_from_slice(&pkt_vec);

                            sink.on_packet(pkt.clone());
                            burst.push(pkt);

                            filled = 0;                       // start a new frame
                        }
                    }
                }
            },
            |err| eprintln!("❌ Recorder error: {err}"),
            None,
        )?;

        stream.play()?;
        Ok(Self { _stream: stream })
    }
}
