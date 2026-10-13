// Guard for the `wasi-patched-deps` feature (Cargo.toml): the feature name
// asserts the build applied the vendored WASI clock patch (vendor/README.md,
// [patch.crates-io] purrdf-sparql-eval). Until now nothing verified that
// assertion — enabling the feature without the vendored tree in place would
// compile clean and trap at runtime on the first clock read outside the
// browser. This file makes the claim checkable.
//
// The feature IS enabled in-tree (wasm/Cargo.toml enables it for graphlaw-wasm),
// so the gated test below is live, not a permanent skip. Under the default
// feature set only the named skip notice runs.

/// Verifies the vendored purrdf-sparql-eval actually carries the patch hunk:
/// upstream crates.io 2.0.2 gates its clock on `target_arch = "wasm32"` alone
/// (browser/js_sys path); the vendored copy must instead gate on
/// `all(target_arch = "wasm32", target_os = "unknown")` so WASI keeps the real
/// std clock. Absence of the vendored tree or of the gate is a typed flunk.
#[cfg(feature = "wasi-patched-deps")]
#[test]
fn wasi_patched_deps_feature_implies_vendored_patch_present() {
    let clock = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vendor/purrdf-sparql-eval/src/clock.rs");
    let src = std::fs::read_to_string(&clock).unwrap_or_else(|e| {
        panic!(
            "WASI_PATCH_UNVERIFIED: vendored crate missing at {} ({e}). \
             Restore vendor/purrdf-sparql-eval (see vendor/README.md) or drop the \
             wasi-patched-deps feature — the crates.io tree has no clock patch.",
            clock.display()
        )
    });
    assert!(
        src.contains(r#"all(target_arch = "wasm32", target_os = "unknown")"#),
        "WASI_PATCH_UNVERIFIED: vendor/purrdf-sparql-eval/src/clock.rs no longer carries the \
         `target_os = \"unknown\"` gate — the vendored copy matches unpatched crates.io 2.0.2. \
         Reapply vendor/upstream/purrdf-sparql-eval.patch."
    );
    // Upstream 2.0.2's browser-only gate must be gone from the vendored file.
    assert!(
        !src.contains("#[cfg(target_arch = \"wasm32\")]"),
        "WASI_PATCH_UNVERIFIED: unpatched upstream gate `#[cfg(target_arch = \"wasm32\")]` found \
         in vendor/purrdf-sparql-eval/src/clock.rs; WASI would take the browser clock path and \
         trap. Reapply vendor/upstream/purrdf-sparql-eval.patch."
    );
    println!(
        "NOTICE [wasi-patch-verified]: vendored purrdf-sparql-eval carries the WASI clock gate."
    );
}

/// Named skip when the feature is off (default build): the guard's coverage is
/// not applicable there, but the skip is witnessed rather than silent.
#[cfg(not(feature = "wasi-patched-deps"))]
#[test]
fn wasi_patch_guard_skipped_without_the_feature() {
    println!(
        "NOTICE [wasi-patch-guard-skip]: wasi-patched-deps is off; the vendored-patch guard \
         runs only under `--features wasi-patched-deps` (CI and wasm/Cargo.toml enable it)."
    );
}
