//! Detect the vendored WASI clock patch. `[patch.crates-io]` is dropped on
//! `cargo publish`, and the `wasi-patched-deps` feature is an empty flag, so
//! neither one can prove the patch is linked. This script sets
//! `graphlaw_wasi_clock_patched` only when `vendor/purrdf-sparql-eval` is
//! present and carries the `target_os = "unknown"` gate. The published crate
//! does not include `vendor/`, so a crates.io build leaves the cfg unset and
//! `src/lib.rs` refuses to compile for WASI.
//!
//! Fail-closed: when the `wasi-patched-deps` feature is requested but the
//! vendored patch is absent or does not carry the gate, the build script
//! aborts the build (the build-script analogue of `compile_error!`). A bare
//! early-return would let the feature lie: clean compile, runtime trap on
//! the first clock read.
fn main() {
    println!("cargo::rustc-check-cfg=cfg(graphlaw_wasi_clock_patched)");
    println!("cargo:rerun-if-changed=vendor/purrdf-sparql-eval/src/clock.rs");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_WASI_PATCHED_DEPS");
    let clock = std::path::Path::new("vendor/purrdf-sparql-eval/src/clock.rs");
    let src = std::fs::read_to_string(clock);
    let patched = match src {
        Ok(src) => {
            src.contains(r#"all(target_arch = "wasm32", target_os = "unknown")"#)
                && !src.contains("#[cfg(target_arch = \"wasm32\")]")
        }
        Err(_) => false,
    };
    if patched {
        println!("cargo:rustc-cfg=graphlaw_wasi_clock_patched");
    } else if std::env::var_os("CARGO_FEATURE_WASI_PATCHED_DEPS").is_some() {
        panic!(
            "WASI_PATCH_BUILD_REFUSED: the `wasi-patched-deps` feature is enabled but the \
             vendored WASI clock patch is missing or unpatched (expected \
             vendor/purrdf-sparql-eval/src/clock.rs gating its clock on \
             `all(target_arch = \"wasm32\", target_os = \"unknown\")`). Restore the vendored \
             tree / reapply vendor/upstream/purrdf-sparql-eval.patch (see vendor/README.md), \
             or drop the feature. A clean compile here would trap at runtime on WASI."
        );
    }
}
