//! Browser / Web‑Audio back‑end (wasm32‑unknown‑unknown).

#![cfg(target_arch = "wasm32")]

use anyhow::Result;
use bytes::Bytes;
use std::{
    collections::VecDeque,
    sync::{Arc, atomic::{AtomicBool, Ordering}},
};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    AudioBuffer, AudioBufferSourceNode, AudioContext, AudioProcessingEvent, MediaStream,
    MediaStreamAudioSourceNode, MediaStreamConstraints,
};

use crate::audio::{
    burst::Burst,
    codec::{OpusDecoder, OpusEncoder},
    traits::PacketSink,
};

/*──── shared constants ─────────────────────────────────────────────*/
pub const OPUS_SR_HZ:    u32   = 48_000;
pub const FRAME_SAMPLES: usize = 960;

/*──── dummy config helpers (keep API identical) ───────────────────*/
pub type DummyCfg = cpal::StreamConfig;
pub fn input_config(_: &cpal::Device)  -> Result<DummyCfg> { Ok(dummy(1)) }
pub fn output_config(_: &cpal::Device) -> Result<DummyCfg> { Ok(dummy(2)) }
fn dummy(ch: u16) -> DummyCfg {
    use cpal::{SampleRate as CpalSR, BufferSize};
    DummyCfg { channels: ch, sample_rate: CpalSR(OPUS_SR_HZ), buffer_size: BufferSize::Fixed(FRAME_SAMPLES as u32) }
}

/*────────────────────────────── Recorder ──────────────────────────*/

pub struct Recorder { _ctx: AudioContext, _node: web_sys::ScriptProcessorNode }

impl Recorder {
    pub fn spawn(
        _dev:  &cpal::Device,      // ignored
        _cfg:  &cpal::StreamConfig,
        ptt:   Arc<AtomicBool>,
        sink:  Arc<dyn PacketSink>,
    ) -> Result<Self> {
        let ctx   = AudioContext::new().map_err(anyhow::Error::msg)?;
        let node  = ctx.create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
            FRAME_SAMPLES as u32, 1, 1,
        ).map_err(anyhow::Error::msg)?;

        // hook microphone → ScriptProcessor
        {
            let ctx2  = ctx.clone();
            let node2 = node.clone();
            spawn_local(async move {
                let stream = request_mic().await.unwrap();
                let src: MediaStreamAudioSourceNode =
                    ctx2.create_media_stream_source(&stream).unwrap();
                src.connect_with_audio_node(&node2).unwrap();
            });
        }

        // encode inside audio callback (runs on main JS thread)
        {
            let ptt  = ptt.clone();
            let sink = sink.clone();
            let mut enc   = OpusEncoder::new()?;
            let mut burst = Burst::default();

            let cb = Closure::<dyn FnMut(_)>::new(move |evt: AudioProcessingEvent| {
                let buf  = evt.input_buffer().unwrap();
                let data = buf.get_channel_data(0).unwrap();

                if ptt.load(Ordering::Relaxed) {
                    let mut frame_i16 = [0i16; FRAME_SAMPLES];
                    for i in 0..FRAME_SAMPLES {
                        frame_i16[i] = (data.get_index(i as u32) * i16::MAX as f32) as i16;
                    }
                    let pkt   = enc.encode(&frame_i16).unwrap();
                    let bytes = Bytes::copy_from_slice(&pkt);
                    sink.on_packet(bytes.clone());
                    burst.push(bytes);
                } else if !burst.is_empty() {
                    sink.on_burst(std::mem::take(&mut burst));
                }
            });
            node.set_onaudioprocess(Some(cb.as_ref().unchecked_ref()));
            cb.forget();
        }

        Ok(Self { _ctx: ctx, _node: node })
    }
}

/*────────────────────────────── Player ───────────────────────────*/

pub struct Player { _ctx: AudioContext }

impl Player {
    pub fn spawn(
        _dev: &cpal::Device,
        _cfg: &cpal::StreamConfig,
        mut rx: tokio::sync::mpsc::Receiver<Bytes>,
    ) -> Result<Self> {
        let ctx = AudioContext::new().map_err(anyhow::Error::msg)?;

        spawn_local({
            let ctx = ctx.clone();
            async move {
                let mut dec = OpusDecoder::new().unwrap();
                while let Some(pkt) = rx.recv().await {
                    let pcm = dec.decode(pkt.as_ref()).unwrap();
                    let buf  = AudioBuffer::new_with_context_options(
                        &ctx,
                        web_sys::AudioBufferOptions::new(FRAME_SAMPLES as f32)
                            .number_of_channels(1).sample_rate(OPUS_SR_HZ as f32)
                    ).unwrap();
                    let f32arr = js_sys::Float32Array::from(&pcm[..]);
                    buf.copy_to_channel(&f32arr, 0).unwrap();

                    let src: AudioBufferSourceNode = ctx.create_buffer_source().unwrap();
                    src.set_buffer(Some(&buf));
                    src.connect_with_audio_node(&ctx.destination()).unwrap();
                    src.start().unwrap();
                }
            }
        });

        Ok(Self { _ctx: ctx })
    }
}

/*────────────────────────── helpers ───────────────────────────────*/

async fn request_mic() -> Result<MediaStream> {
    use wasm_bindgen_futures::JsFuture;
    let nav   = web_sys::window().unwrap().navigator();
    let md    = nav.media_devices().map_err(anyhow::Error::msg)?;
    let mut c = MediaStreamConstraints::new();
    c.audio(&JsValue::TRUE);
    let p    = md.get_user_media_with_constraints(&c).map_err(anyhow::Error::msg)?;
    let js   = JsFuture::from(p).await.map_err(anyhow::Error::msg)?;
    Ok(js.dyn_into::<MediaStream>().unwrap())
}
