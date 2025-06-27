//! **everywhere‑net**
//!
//! Enable exactly **one** runtime feature
//! (`native`, `browser`, or `wasi`) in *Cargo.toml* and then:
//!
//! ```no_run
//! use everywhere_net::prelude::*;
//!
//! # async fn demo() -> anyhow::Result<()> {
//! // default backend settings
//! let mut ws = WsConnection::connect("wss://echo.websocket.events").await?;
//! ws.send(WsMessage::Text("ping".into())).await?;
//!
//! // caller‑supplied tweaks
//! let opts = WsOptions::new()
//!     .protocol("json")
//!     .max_frame_size(64 << 10);
//!
//! let mut ws = WsConnection::connect_with("wss://echo.websocket.events", &opts).await?;
//! # Ok(()) }
//! ```
//#![deny(missing_docs)]

pub mod message;
mod backend;
mod options;

pub use message::{WsError, WsMessage};
pub use options::WsOptions;

/*──── re‑export the active backend struct ───────────────────────────────*/
cfg_if::cfg_if! {
    if #[cfg(feature = "native")]   { pub use backend::native  ::WsConnection; }
    else if #[cfg(feature = "browser")] { pub use backend::browser::WsConnection; }
    else if #[cfg(feature = "wasi")]    { pub use backend::wasi   ::WsConnection; }
    else { compile_error!("Enable exactly ONE of: native | browser | wasi"); }
}

/*──── front‑porch facade (same on every target) ─────────────────────────*/

impl WsConnection {
    /// Connect with **backend defaults**.
    pub async fn connect(url: &str) -> Result<Self, WsError> {
        Self::_connect_backend(url, &WsOptions::default()).await
    }

    /// Connect with **caller‑supplied** [`WsOptions`].
    pub async fn connect_with(url: &str, opts: &WsOptions) -> Result<Self, WsError> {
        Self::_connect_backend(url, opts).await
    }
}

/*──── convenience glob ──────────────────────────────────────────────────*/
pub mod prelude {
    pub use crate::{WsConnection, WsMessage, WsOptions};
    pub use futures_util::{SinkExt, StreamExt};
}
