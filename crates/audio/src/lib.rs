//! Public surface.  Most users can just `use cross_audio::prelude::*;`

pub mod audio;
pub use audio::*;

pub mod prelude {
    pub use crate::audio::{
        codec::{OpusDecoder, OpusEncoder},
        device::{input_config, output_config, FRAME_SAMPLES, OPUS_SR_HZ, DeviceProfile},
        frame::Frame,
        burst::Burst,
        graph::AudioGraph,
        player::Player,
        recorder::Recorder,
        sinks::mpsc::MpscSink,
        traits::{PacketSink, PacketSource},
    };
    pub use cpal::traits::HostTrait;
    pub use cpal::traits::DeviceTrait;
    #[cfg(feature = "file")]
    pub use crate::audio::sinks::wav::WavSink;
    #[cfg(feature = "transcribe")]
    pub use crate::audio::sinks::transcribe::WhisperSink;
}
