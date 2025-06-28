//! CPAL helper – always 48 kHz f32; no more manual config fiddling.

use anyhow::Result;
use cpal::{traits::*, BufferSize, SampleFormat, SampleRate as CpalSR, StreamConfig};

pub const OPUS_SR_HZ:    u32   = 48_000;
pub const FRAME_SAMPLES: usize = 960;          // 20 ms @48 kHz

/// Profiles we might want later (e.g. 16 kHz narrow‑band).
pub enum DeviceProfile {
    MonoIn48,
    StereoOut48,
}

impl DeviceProfile {
    pub fn open(&self, dev: &cpal::Device) -> Result<StreamConfig> {
        match self {
            Self::MonoIn48  => input_config(dev),
            Self::StereoOut48 => output_config(dev),
        }
    }
}

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

// //! Public, backend‑agnostic view of the low‑level device helpers.
// //
// //! All the real work is done in `audio::backend::*`.
//
// pub use crate::audio::backend::{
//     OPUS_SR_HZ, FRAME_SAMPLES,
//     input_config, output_config,
// };
//
// /// Convenience enum kept for source compatibility.
// pub enum DeviceProfile {
//     MonoIn48,
//     StereoOut48,
// }
// impl DeviceProfile {
//     pub fn open(&self, dev: &cpal::Device) -> anyhow::Result<cpal::StreamConfig> {
//         match self {
//             Self::MonoIn48    => input_config(dev),
//             Self::StereoOut48 => output_config(dev),
//         }
//     }
// }
