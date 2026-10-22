//! Detect the vendored WASI clock patch. `[patch.crates-io]` is dropped on
//! `cargo publish`, and the `wasi-patched-deps` feature is an empty flag, so
//! neither one can prove the patch is linked. This script sets
//! `graphlaw_wasi_clock_patched` only when `vendor/purrdf-sparql-eval` is
//! present and carries the `target_os = "unknown"` gate. The published crate
//! does not include `vendor/`, so a crates.io build leaves the cfg unset and
//! `src/lib.rs` refuses to compile for WASI.
fn main() {
    println!("cargo::rustc-check-cfg=cfg(graphlaw_wasi_clock_patched)");
    println!("cargo:rerun-if-changed=vendor/purrdf-sparql-eval/src/clock.rs");
    let clock = std::path::Path::new("vendor/purrdf-sparql-eval/src/clock.rs");
    let Ok(src) = std::fs::read_to_string(clock) else {
        return;
    };
    let patched = src.contains(r#"all(target_arch = "wasm32", target_os = "unknown")"#)
        && !src.contains("#[cfg(target_arch = \"wasm32\")]");
    if patched {
        println!("cargo:rustc-cfg=graphlaw_wasi_clock_patched");
    }
}
