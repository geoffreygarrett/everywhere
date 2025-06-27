//! Sharable test helpers for every runtime.
//
//  ✅  Browser      → wasm-bindgen-test
//  ✅  Tokio/native → #[tokio::test]
//  ✅  WASI         → #[async_std::test]

#[macro_export]
macro_rules! cross_test {
    ($name:ident $body:block) => {
        /* ---------- Browser (wasm32 + feature=browser) ---------- */
        #[cfg(all(target_arch = "wasm32", feature = "browser"))]
        mod $name {
            use super::*;
            use wasm_bindgen_test::*;

            wasm_bindgen_test_configure!(run_in_browser);

            #[wasm_bindgen_test]
            async fn run() $body
        }

        /* ---------- WASI (wasm32 + feature=wasi) ---------------- */
        #[cfg(all(target_arch = "wasm32", feature = "wasi"))]
        #[async_std::test]
        async fn $name() $body

        /* ---------- Native / Tokio ------------------------------ */
        #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
        #[tokio::test]
        async fn $name() $body
    };
}
