//! 48 kHz mono Opus helpers – encode one frame / decode one packet

use anyhow::Result;
use audiopus::{coder, packet::Packet, Application, Channels, MutSignals, SampleRate};

use crate::audio::device::FRAME_SAMPLES;

pub struct OpusEncoder(coder::Encoder);
pub struct OpusDecoder(coder::Decoder);

impl OpusEncoder {
    pub fn new() -> Result<Self> {
        Ok(Self(coder::Encoder::new(
            SampleRate::Hz48000, Channels::Mono, Application::Voip)?))
    }
    pub fn encode(&mut self, pcm: &[i16]) -> Result<Vec<u8>> {
        let mut buf = [0u8; 400];
        let n = self.0.encode(pcm, &mut buf)?;
        Ok(buf[..n].to_vec())
    }
}

impl OpusDecoder {
    pub fn new() -> Result<Self> {
        Ok(Self(coder::Decoder::new(
            SampleRate::Hz48000, Channels::Mono)?))
    }
    pub fn decode(&mut self, pkt: &[u8]) -> Result<Vec<f32>> {
        let packet = Packet::try_from(pkt)?;
        let mut pcm_i16 = vec![0i16; FRAME_SAMPLES];
        let mut sig      = MutSignals::try_from(&mut pcm_i16[..]).unwrap();
        let n = self.0.decode(Some(packet), sig, false)?;
        Ok(pcm_i16[..n].iter().map(|&s| s as f32 / i16::MAX as f32).collect())
    }
}
