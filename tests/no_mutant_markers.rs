//! `cargo mutants --in-place` edits the real checkout and marks every edit with a
//! comment. A run killed mid-mutant (OOM, SIGKILL) leaves the mutation behind, and
//! a later `git add -A` commits it. No tracked source may carry that marker.

use std::fs;
use std::path::Path;

fn walk(dir: &Path, hits: &mut Vec<String>) {
    let marker = concat!("changed by ", "cargo-mutants");
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() {
            if !matches!(name.as_str(), "target" | "vendor" | "mutants.out" | ".git")
                && !name.starts_with("target-")
            {
                walk(&path, hits);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            for (n, line) in text.lines().enumerate() {
                if line.contains(marker) {
                    hits.push(format!("{}:{}", path.display(), n + 1));
                }
            }
        }
    }
}

#[test]
fn no_leaked_cargo_mutants_edits_in_source() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut hits = Vec::new();
    for d in ["src", "tests", "wasm", "crates"] {
        walk(&root.join(d), &mut hits);
    }
    assert!(hits.is_empty(), "leaked mutant edits:\n{}", hits.join("\n"));
}
