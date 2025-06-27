//! Private per‑platform implementations.
#[cfg(feature = "native")]   pub(crate) mod native;
#[cfg(feature = "browser")]  pub(crate) mod browser;
#[cfg(feature = "wasi")]     pub(crate) mod wasi;
