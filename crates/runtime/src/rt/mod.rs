//! Select exactly **one** concrete backend; re‑export it as `Rt`.

cfg_if::cfg_if! {
    if #[cfg(all(feature = "native", not(target_arch = "wasm32")))] {
        mod native;
        pub use native::TokioRt as Rt;
    } else if #[cfg(all(feature = "browser", target_arch = "wasm32"))] {
        mod browser;
        pub use browser::BrowserRt as Rt;
    } else if #[cfg(all(feature = "wasi", target_arch = "wasm32"))] {
        mod wasi;
        pub use wasi::WasiRt as Rt;
    } else {
        compile_error!("Enable **exactly one** of: native | browser | wasi");
    }
}
