//! Public façade re-exporting the attribute macro.
pub use everywhere_test_macro::cross_test;

/*──────────────────────── shim used by the macro ───────────────────────*/
#[doc(hidden)]
pub mod __rt {
    /*── native / Tokio ────────────────────────────────────────────────*/
    #[cfg(not(target_arch = "wasm32"))]
    pub mod tokio {
        pub use ::tokio::*;
    }

    /*── browser / wasm-bindgen-test ───────────────────────────────────*/
    #[cfg(all(feature = "browser", target_arch = "wasm32"))]
    pub mod wasm_bindgen_test {
        pub use ::wasm_bindgen_test::*;
    }

    /*── WASI / async-std ──────────────────────────────────────────────*/
    #[cfg(target_os = "wasi")]
    pub mod async_std {
        pub use ::async_std::*;
    }
}
