//! **everywhere** – umbrella façade
//!
//! ```toml
//! everywhere = { version = "0.1", features = ["net", "runtime", "native"] }
//! ```
//!
//! gives you
//!
//! * 📡  [`everywhere::net`]      – portable WebSocket
//! * ⚙️  [`everywhere::runtime`] – `spawn / sleep / channel` on any target
//! * ⏲️  [`everywhere::timer`]   – exponential back-off helper
//! * ✅  [`everywhere::test`]    – one attribute for all async test harnesses
//! * 🔊  [`everywhere::audio`]   – PTT / voice-note goodies (host only)
//!
//! ### Choosing the target
//! Exactly **one** of `native | browser | wasi` must be enabled – just like
//! in the individual crates.

#![cfg_attr(
    not(feature = "native"),
    cfg_attr(not(feature = "browser"),
        cfg_attr(not(feature = "wasi"), no_std))
)]

/*──────────────────── target sanity check ────────────────────*/
#[cfg(any(
    all(feature = "native", feature = "browser"),
    all(feature = "native", feature = "wasi"),
    all(feature = "browser", feature = "wasi"),
    not(any(feature = "native", feature = "browser", feature = "wasi")),
))]
compile_error!(
    "Enable **exactly one** of: `native`, `browser`, or `wasi` on the \
     `everywhere` crate (they forward to all sub-crates)."
);

/*──────────────────── component re-exports ───────────────────*/
#[cfg(feature = "net")]
pub use everywhere_net     as net;
#[cfg(feature = "runtime")]
pub use everywhere_runtime as runtime;
#[cfg(feature = "timer")]
pub use everywhere_timer   as timer;
#[cfg(feature = "test")]
pub use everywhere_test    as test;
#[cfg(feature = "audio")]
pub use everywhere_audio   as audio;

/*──────────────────── convenience glob ───────────────────────*/
/// Conveniences for the common trio – pull in what 90 % of binaries need.
#[cfg(all(feature = "net", feature = "runtime", feature = "timer"))]
pub mod prelude {
    pub use super::{net, runtime::prelude::*, timer};
}
