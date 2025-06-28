//! Back‑end selector – decide once at compile‑time which concrete audio
//! implementation to pull in.  The chosen module re‑exports **Recorder**,
//! **Player**, `input_config`, `output_config`, `FRAME_SAMPLES`, `OPUS_SR_HZ`.

cfg_if::cfg_if! {
    if #[cfg(all(feature = "cpal_backend", not(target_arch = "wasm32")))] {
        mod native;
        pub use native::*;
    } else if #[cfg(all(feature = "web_backend", target_arch = "wasm32"))] {
        mod web;
        pub use web::*;
    } else {
        compile_error!("Enable exactly one of the mutually‑exclusive features \
                       `cpal_backend` (native) or `web_backend` (browser wasm32)");
    }
}
