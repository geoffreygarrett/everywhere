
//! Native (desktop / mobile / server) audio using **CPAL**.

use anyhow::Result;
use cpal::{traits::*, BufferSize, SampleFormat, SampleRate as CpalSR, StreamConfig};
use std::{collections::VecDeque, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use bytes::Bytes;

use crate::audio::{
    burst::Burst,
    codec::{OpusDecoder, OpusEncoder},
    frame::Frame,
    traits::PacketSink,
};

/*──── shared constants ───────────────────────────────────────────────*/
pub const OPUS_SR_HZ:    u32   = 48_000;
pub const FRAME_SAMPLES: usize = 960;

/*──── helpers – identical to the old device.rs ───────────────────────*/
pub fn input_config(dev: &cpal::Device) -> Result<StreamConfig> {
    let sup = dev.supported_input_configs()?
        .find(|c| c.channels() == 1
            && c.sample_format() == SampleFormat::F32
            && c.min_sample_rate().0 <= OPUS_SR_HZ
            && c.max_sample_rate().0 >= OPUS_SR_HZ)
        .ok_or_else(|| anyhow::anyhow!("mic lacks 48 kHz mono‑f32"))?;
    let mut cfg = sup.with_sample_rate(CpalSR(OPUS_SR_HZ)).config();
    cfg.buffer_size = BufferSize::Fixed(FRAME_SAMPLES as u32);
    Ok(cfg)
}
pub fn output_config(dev: &cpal::Device) -> Result<StreamConfig> {
    let sup = dev.supported_output_configs()?
        .find(|c| c.channels() == 2
            && c.sample_format() == SampleFormat::F32
            && c.min_sample_rate().0 <= OPUS_SR_HZ
            && c.max_sample_rate().0 >= OPUS_SR_HZ)
        .ok_or_else(|| anyhow::anyhow!("speaker lacks 48 kHz stereo‑f32"))?;
    let mut cfg = sup.with_sample_rate(CpalSR(OPUS_SR_HZ)).config();
    cfg.buffer_size = BufferSize::Fixed(FRAME_SAMPLES as u32);
    Ok(cfg)
}

/*──────────────────────────────── Recorder ─────────────────────────────────*/

pub struct Recorder { _stream: cpal::Stream }

impl Recorder {
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        dev:  &cpal::Device,
        cfg:  &cpal::StreamConfig,
        ptt:  Arc<AtomicBool>,
        sink: Arc<dyn PacketSink>,
    ) -> Result<Self> {
        let mut enc     = OpusEncoder::new()?;
        let mut frame   = Frame::<FRAME_SAMPLES>::default();
        let mut filled  = 0usize;
        let mut burst   = Burst::default();

        let stream = dev.build_input_stream(
            cfg,
            move |input: &[f32], _| {
                let pressed = ptt.load(Ordering::Relaxed);

                if !pressed && !burst.is_empty() {
                    sink.on_burst(std::mem::take(&mut burst));
                    filled = 0;
                }
                if pressed {
                    for &s in input {
                        frame[filled] = (s * i16::MAX as f32) as i16;
                        filled += 1;

                        if filled == FRAME_SAMPLES {
                            let pkt   = enc.encode(&frame[..]).unwrap();
                            let bytes = Bytes::copy_from_slice(&pkt);
                            sink.on_packet(bytes.clone());
                            burst.push(bytes);
                            filled = 0;
                        }
                    }
                }
            },
            |e| eprintln!("❌ Recorder error: {e}"),
            None,
        )?;
        stream.play()?;
        Ok(Self { _stream: stream })
    }
}

/*──────────────────────────────── Player ─────────────────────────────────*/

pub struct Player { _stream: cpal::Stream }

impl Player {
    pub fn spawn(
        dev: &cpal::Device,
        cfg: &cpal::StreamConfig,
        mut rx: tokio::sync::mpsc::Receiver<Bytes>,
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
                        Err(_)  => break,
                    }
                }
                for frame in out.chunks_exact_mut(2) {
                    let s = ring.pop_front().unwrap_or(0.0);
                    frame[0] = s; frame[1] = s;
                }
            },
            |_| {},
            None,
        )?;
        stream.play()?;
        Ok(Self { _stream: stream })
    }
}
