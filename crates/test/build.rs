fn main() {
    // Re-run the build script if features change (so the cfg flag updates).
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(feature = "native") {
        println!("cargo:rustc-cfg=cross_test_rt=\"native\"");
    } else if cfg!(feature = "browser") {
        println!("cargo:rustc-cfg=cross_test_rt=\"browser\"");
    } else if cfg!(feature = "wasi") {
        println!("cargo:rustc-cfg=cross_test_rt=\"wasi\"");
    } else {
        // Force at least one runtime feature to be chosen.
        // (Compile error earlier than the proc-macro would catch it.)
        println!("cargo:warning=⚠  Enable one of: native | browser | wasi");
    }
}
