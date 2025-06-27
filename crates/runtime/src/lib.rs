//! **everywhere‑runtime**
//!
//! Portable async primitives on **Tokio** (native), **wasm‑bindgen**
//! (browser) and **async‑std** (WASI).

#![allow(async_fn_in_trait)]
#![cfg_attr(
    not(feature = "native"),
    cfg_attr(not(feature = "browser"),
        cfg_attr(not(feature = "wasi"), no_std))
)]

mod api;               /* contract & helpers */
mod rt;                /* concrete back‑ends */

pub mod task;          /* spawn / channel façade   */
pub mod time;          /* sleep  / timeout façade  */

/*──────────────────────── re‑exports ───────────────────────*/
pub use api::{Runtime as _RtTrait, SenderExt, ReceiverExt};
pub use rt::Rt;

/* Common glob import */
pub mod prelude {
    pub use super::{Rt, SenderExt, ReceiverExt};
    pub use crate::api::Runtime;
}
