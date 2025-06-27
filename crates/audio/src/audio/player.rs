//! Pulls Opus packets from an mpsc channel and plays them.

use anyhow::Result;
use bytes::Bytes;
use cpal::{traits::*, Stream};
use std::collections::VecDeque;
use tokio::sync::mpsc::Receiver;

use crate::audio::codec::OpusDecoder;

pub struct Player {
    _stream: Stream,
}

impl Player {
    pub fn spawn(
        dev: &cpal::Device,
        cfg: &cpal::StreamConfig,
        mut rx: Receiver<Bytes>,
    ) -> Result<Self> {
        let mut dec = OpusDecoder::new()?;
        static mut RING: Option<VecDeque<f32>> = None;

        let stream = dev.build_output_stream(
            cfg,
            move |out: &mut [f32], _| {
                let ring = unsafe { RING.get_or_insert_with(VecDeque::new) };

                while ring.len() < out.len() {
                    match rx.try_recv() {
                        Ok(pkt) => ring.extend(dec.decode(pkt.as_ref()).unwrap()),
                        Err(_) => break,
                    }
                }

                for frame in out.chunks_exact_mut(2) {
                    let s = ring.pop_front().unwrap_or(0.0);
                    frame[0] = s;
                    frame[1] = s;
                }
            },
            |_| {},
            None,
        )?;
        stream.play()?;
        Ok(Self { _stream: stream })
    }
}
