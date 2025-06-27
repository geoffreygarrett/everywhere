//! Write packets to a 48 kHz mono 16‑bit `.wav` (needs `--features file`)

#[cfg(feature = "file")]
use {
    crate::audio::{codec::OpusDecoder, device::OPUS_SR_HZ, traits::PacketSink},
    bytes::Bytes,
    hound::{WavSpec, WavWriter},
    std::sync::mpsc,
};

#[cfg(feature = "file")]
pub struct WavSink { tx: mpsc::Sender<Bytes> }

#[cfg(feature = "file")]
impl WavSink {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let spec = WavSpec {
            channels:        1,
            sample_rate:     OPUS_SR_HZ,
            bits_per_sample: 16,
            sample_format:   hound::SampleFormat::Int,
        };
        let writer = WavWriter::create(path, spec)?;
        let (tx, rx) = mpsc::channel::<Bytes>();

        std::thread::spawn(move || {
            let mut writer = writer;
            let mut dec = OpusDecoder::new().unwrap();
            for pkt in rx {
                for s in dec.decode(&pkt).unwrap() {
                    writer.write_sample((s * i16::MAX as f32) as i16).unwrap();
                }
            }
            let _ = writer.finalize();
        });

        Ok(Self { tx })
    }
}

#[cfg(feature = "file")]
impl PacketSink for WavSink {
    fn on_packet(&self, p: Bytes) { let _ = self.tx.send(p); }
}
