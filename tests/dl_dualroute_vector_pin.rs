//! Vector-byte pin for the DL dual-route generator (K10 defect class).
//!
//! Defect (K10): `src/bin/dl_dualroute.rs` silently added a `graphlaw_version`
//! provenance field — the emitted vector bytes changed with no pin to catch
//! it. This court closes that class: it runs the REAL generator binary end to
//! end (Chicago; no reimplementation of the emission logic), reads back the
//! four emitted vector files, normalizes ONLY the per-commit-volatile
//! `graphlaw_rev` field, and asserts a BLAKE3 digest pin per vector.
//!
//! Serialization fields covered by the pin (the `emit_vector` format string):
//! `vector_id`, `breed`, `provenance.{graphlaw_version, purrdf, eyeron,
//! generator, stratified}`, `breed_input.{intent, facts, rules, goals,
//! candidates, cases, state}`, `expected.{derived_facts_sorted, verdicts,
//! notes}`. ANY field addition/removal/rename or any value-format change
//! flips the digest and fails this test loudly.
//!
//! Re-pin protocol: a digest here changes ONLY by an intentional
//! serialization change to `emit_vector`. Update the pinned digest in the
//! same diff as the generator change, with the reason in the commit message.
//! (A `graphlaw_version` bump is such a change — the version lives inside the
//! digest on purpose: provenance changes must be receipted, not silent.)

use blake3::Hasher;
use std::path::PathBuf;
use std::process::Command;

/// (slug, pinned BLAKE3 of the rev-normalized emitted vector bytes).
const PINS: &[(&str, &str)] = &[
    (
        "a-paper-baader-el",
        "fb7f2781f25fb82951cf98055b3e469e5f8ece1a49e67e5c4b7aa258542f61de",
    ),
    (
        "b-chain3-conj",
        "13e99303c35548d809bc2fb7c05f81f2b8866d91c4ec240f6bdbd118bab90093",
    ),
    (
        "c-exists-pair",
        "5cdb06402530732800ea9c9b9295f8dc0995fa6646cbbe40799d647e642d5ca2",
    ),
    (
        "d-exists-conj-interlock",
        "93cf4d71252f1f720df7218021e98dd37fb322e6847d825ebfc2bc2d4bc7e28c",
    ),
];

// vectors are written to and read from the wasm4pm cognition fixtures dir.
// MACHINE-PINNED: sibling canonical checkout (one-canonical-checkout law).
const OUT_DIR: &str = "/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw";

/// The per-commit-volatile provenance field, normalized before hashing so the
/// pin asserts serialization stability, not "HEAD did not move."
const REV_LINE: &str = "graphlaw_rev";

fn normalize(bytes: &[u8]) -> Vec<u8> {
    let text = String::from_utf8(bytes.to_vec()).expect("vector is UTF-8");
    // Replace only the rev value; keep the field present so its removal is
    // still caught by the pin. Untouched lines stay byte-exact.
    let mut rebuilt = Vec::with_capacity(bytes.len() + 32);
    for line in text.split_inclusive('\n') {
        if line.contains(REV_LINE) && line.contains("\": \"") {
            let trimmed = line.trim_end();
            let indent = &trimmed[..trimmed.len() - trimmed.trim_start().len()];
            rebuilt.extend_from_slice(indent.as_bytes());
            rebuilt.extend_from_slice(b"\"graphlaw_rev\": \"<REV-NORMALIZED>\",\n");
        } else {
            rebuilt.extend_from_slice(line.as_bytes());
        }
    }
    rebuilt
}

fn hash(bytes: &[u8]) -> String {
    let mut h = Hasher::new();
    h.update(bytes);
    h.finalize().to_string()
}

#[test]
fn dl_dualroute_vector_bytes_are_pinned() {
    // Run the real generator binary (Chicago: real collaborator, real bytes).
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let status = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest.join("Cargo.toml"))
        .arg("--bin")
        .arg("dl_dualroute")
        .status()
        .expect("spawn cargo run --bin dl_dualroute");
    assert!(
        status.success(),
        "dl_dualroute generator failed: {status} (dual-route disagreement or write failure)"
    );

    for (slug, pin) in PINS {
        let path = format!("{OUT_DIR}/description_logic-{slug}.json");
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("read emitted vector {path}: {e}"));
        let digest = hash(&normalize(&bytes));
        assert_eq!(
            &digest, *pin,
            "vector serialization drift for '{slug}': emitted bytes no longer match the \
             pinned digest.\n  pinned:   {pin}\n  emitted:  {digest}\n\
             If the change to src/bin/dl_dualroute.rs emit_vector is intentional, \
             re-pin this digest in the same diff and state the reason."
        );
    }
}
