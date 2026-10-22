//! TC-MOCK-00 build gate: no banned test-double markers in tracked source.
//!
//! Scans `src/`, `tests/`, `wasm/`, `crates/` for the mock-laundering markers
//! banned by the Chicago doctrine (`[[testing-chicago-style]]`):
//!   * the `mockall` crate (mockall!/automock!/any `mockall` reference)
//!   * `jest.mock(`-class patterns (`jest.mock`, `vi.mock`, `sinon.stub`)
//!   * `#[cfg(test)]`-gated `struct Mock*` doubles (marker = `struct Mock` /
//!     `impl Mock` text) — adjudicated against the ALLOWLIST below.
//!
//! `#[cfg(test)]` itself is NOT a violation: gated unit tests exercising real
//! collaborators are ordinary Rust, not test doubles.
//!
//! Hand-written real-interface implementations are NOT mocks. A type that
//! implements the production trait over real data (a real map of real parsed
//! AST statements, no call recording, no scripted expectations) is a real
//! collaborator; only types that *record or script interactions* are doubles.

use std::fs;
use std::path::Path;

/// Banned marker substrings, split so this file never matches itself.
const BANNED: &[(&str, &str)] = &[
    ("mock", "all"),          // mockall crate / macro
    ("auto", "mock!"),        // automock! attribute macro
    ("jest", ".mock"),        // jest.mock( class-of patterns
    ("vi", ".mock"),          // vitest vi.mock(
    ("sinon", ".stub"),       // sinon.stub(
    ("struct Mock", ""),      // cfg(test) Mock* double structs
];

/// Documented non-violations. Key = path relative to the manifest root,
/// value = justification (Chicago doctrine: hand-written real-interface
/// implementation of the real `RdfAstGraphReader` trait over real data).
const ALLOWLIST: &[(&str, &str)] = &[
    (
        "tests/no_mock_markers.rs",
        "this gate: marker literals are split so the scanner cannot match itself.",
    ),
];

fn walk(dir: &Path, root: &Path, hits: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() {
            if !matches!(name.as_str(), "target" | "vendor" | ".git" | "receipts")
                && !name.starts_with("target-")
            {
                walk(&path, root, hits);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            for (n, line) in text.lines().enumerate() {
                for (a, b) in BANNED {
                    let marker = format!("{a}{b}");
                    if line.contains(&marker) && !ALLOWLIST.iter().any(|(p, _)| *p == rel) {
                        hits.push(format!(
                            "{path:?}:{}: banned test-double marker {marker:?} — no allowlist \
                             entry; hand-written real-interface impls are lawful (rename or \
                             allowlist with justification)",
                            n + 1
                        ));
                    }
                }
            }
        }
    }
}

#[test]
fn no_banned_mock_markers_in_source() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut hits = Vec::new();
    for d in ["src", "tests", "wasm", "crates"] {
        let dir = root.join(d);
        if dir.is_dir() {
            walk(&dir, root, &mut hits);
        }
    }
    assert!(
        hits.is_empty(),
        "banned test-double markers (Chicago doctrine; allowlist \
         non-violations with justification, never silently):\n{}",
        hits.join("\n")
    );
}
