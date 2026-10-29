//! `graphlaw-registry`: emit or check the capability registry files.
//!
//! ```text
//! graphlaw-registry --write | --check | --print-json | --print-ttl | --print-mcp | --print-digests
//! ```
//!
//! `--write` writes `registry/capability-registry.json`, `.ttl` and
//! `mcp-manifest.json` under the crate root, and re-pins the ARTIFACTS.blake3 rows
//! this bin owns (`registry`, `surface`, `receipt-surface`, `mcp-manifest`) from
//! live computed digests. The `wasm` row pins a build artifact and is NOT touched
//! by this bin — it is re-pinned by the wasm rebuild flow.
//! `--check` exits 1 with a difference summary when the committed files differ
//! from what the Rust table emits (including the four owned ARTIFACTS rows).
//! Exit codes: 0 ok, 1 drift, 2 usage or I/O error.

use std::path::PathBuf;
use std::process::ExitCode;

use graphlaw::registry::{
    mcp_manifest_blake3, mcp_manifest_pretty, receipt_surface_blake3, registry_blake3,
    registry_json_pretty, registry_turtle, surface_blake3,
};

fn registry_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("registry")
}

const HELP: &str = "\
graphlaw-registry — emit or check the capability registry files.

Usage:
  graphlaw-registry --write | --check | --print-json | --print-ttl | --print-mcp | --print-digests

Modes (exactly one per invocation):
  --write          Write registry/capability-registry.json, .ttl and mcp-manifest.json
                   under the crate root and re-pin the ARTIFACTS.blake3 rows this bin
                   owns (`registry`, `surface`, `receipt-surface`, `mcp-manifest`);
                   the `wasm` row is never touched here — it is re-pinned by the wasm
                   rebuild flow
  --check          Compare the committed files against live output; exit 1 with a
                   first-difference summary on drift
  --print-json     Print the capability registry JSON to stdout
  --print-ttl      Print the capability registry Turtle to stdout
  --print-mcp      Print the MCP tool manifest JSON to stdout
  --print-digests  Print the live surface/registry/mcp-manifest BLAKE3 digests

Exit codes: 0 ok, 1 drift (`--check`), 2 usage or I/O error.";

fn usage(msg: &str) -> ExitCode {
    eprintln!("graphlaw-registry: {msg}");
    eprintln!(
        "usage: graphlaw-registry --write | --check | --print-json | --print-ttl | --print-mcp | --print-digests"
    );
    eprintln!("run `graphlaw-registry --help` for mode details");
    ExitCode::from(2)
}

const ARTIFACTS_FILE: &str = "ARTIFACTS.blake3";

/// The four rows this bin owns, in fixed output order: kind, digest, byte-count
/// placeholder, name. The byte-count field is `-` for document digests (they are
/// digests of canonical documents, not of any file on disk).
fn owned_artifacts_rows() -> [String; 4] {
    [
        format!("registry {} - capability-registry.json", registry_blake3()),
        format!("surface {} - capability-registry.json", surface_blake3()),
        format!(
            "receipt-surface {} - src/duckdb_eval.rs+src/sql_ast.rs",
            receipt_surface_blake3()
        ),
        format!(
            "mcp-manifest {} - registry/mcp-manifest.json",
            mcp_manifest_blake3()
        ),
    ]
}

const ARTIFACTS_TRAILING_COMMENTS: &[&str] = &[
    "# receipt-surface: blake3 of the canonical graphlaw.receipt-surface/1 document",
    "#   (per-file blake3+bytes for src/duckdb_eval.rs and src/sql_ast.rs, the receipt-minting",
    "#   surface); recomputed by graphlaw::registry::receipt_surface_blake3(). Re-pin via",
    "#   `cargo run --bin graphlaw-registry -- --write` whenever either file changes.",
];

const WASM_ROW_NOTE: &str =
    "# wasm row: never re-pinned by graphlaw-registry; re-pinned by the wasm rebuild flow.";

/// One parsed row of ARTIFACTS.blake3 (comment lines are preserved verbatim).
struct Artifacts {
    lines: Vec<String>,
    wasm_row: Option<String>,
}

fn parse_artifacts(text: &str) -> Artifacts {
    let mut lines = Vec::new();
    let mut wasm_row = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            lines.push(line.to_string());
        } else {
            let mut fields = line.split_whitespace();
            match fields.next() {
                Some("wasm") => wasm_row = Some(line.to_string()),
                Some("registry" | "surface" | "receipt-surface" | "mcp-manifest") => {} // owned: regenerated
                Some(other) => {
                    // Unknown row kinds are hand-authored pins; preserve them.
                    lines.push(line.to_string());
                    let _ = other;
                }
                None => {}
            }
        }
    }
    Artifacts { lines, wasm_row }
}

/// The full regenerated ARTIFACTS.blake3 content: every preserved comment/unknown/
/// wasm line in original order, the wasm-row ownership note if absent, the three
/// owned rows in fixed order, then the regenerated trailing comment block.
fn artifacts_repin(existing: Option<&str>) -> Result<String, String> {
    let mut out = String::new();
    let mut have_wasm_note = false;
    if let Some(text) = existing {
        let parsed = parse_artifacts(text);
        for line in &parsed.lines {
            // Trailing comments are regenerated below; the wasm note is kept in place.
            if ARTIFACTS_TRAILING_COMMENTS.contains(&line.as_str()) {
                continue;
            }
            if line == WASM_ROW_NOTE {
                have_wasm_note = true;
            }
            out.push_str(line);
            out.push('\n');
        }
        let wasm = parsed.wasm_row.as_deref().ok_or_else(|| {
            format!(
                "{ARTIFACTS_FILE} has no `wasm` row; the bin does not pin build artifacts — \
                 re-pin it via the wasm rebuild flow, then rerun --write"
            )
        })?;
        if !have_wasm_note {
            out.push_str(WASM_ROW_NOTE);
            out.push('\n');
        }
        out.push_str(wasm);
        out.push('\n');
    }
    for row in owned_artifacts_rows() {
        out.push_str(&row);
        out.push('\n');
    }
    for c in ARTIFACTS_TRAILING_COMMENTS {
        out.push_str(c);
        out.push('\n');
    }
    Ok(out)
}

/// The live digests as ARTIFACTS rows, for `--check`.
fn artifacts_expected_rows() -> [String; 4] {
    owned_artifacts_rows()
}

fn first_difference(expected: &str, found: &str) -> String {
    for (i, (e, f)) in expected.lines().zip(found.lines()).enumerate() {
        if e != f {
            return format!(
                "first difference at line {}: expected `{e}`, found `{f}`",
                i + 1
            );
        }
    }
    format!(
        "line counts differ: expected {}, found {}",
        expected.lines().count(),
        found.lines().count()
    )
}

fn check() -> ExitCode {
    let dir = registry_dir();
    let mut drift = false;
    for (file, expected) in [
        ("capability-registry.json", registry_json_pretty()),
        ("capability-registry.ttl", registry_turtle()),
        ("mcp-manifest.json", mcp_manifest_pretty()),
    ] {
        match std::fs::read_to_string(dir.join(file)) {
            Ok(found) if found == expected => println!("ok      {file}"),
            Ok(found) => {
                drift = true;
                println!("DRIFT   {file}: {}", first_difference(&expected, &found));
            }
            Err(e) => {
                drift = true;
                println!("MISSING {file}: {e}");
            }
        }
    }
    // ARTIFACTS.blake3: the three rows this bin owns must match live digests.
    match std::fs::read_to_string(dir.join(ARTIFACTS_FILE)) {
        Ok(text) => {
            let found: Vec<String> = text
                .lines()
                .filter(|l| !l.trim_start().starts_with('#'))
                .map(str::to_string)
                .collect();
            let row = |kind: &str| {
                found
                    .iter()
                    .find(|l| l.starts_with(kind))
                    .map(|l| l.to_string())
            };
            for expected in artifacts_expected_rows() {
                let kind = expected.split_whitespace().next().unwrap_or_default();
                match row(kind) {
                    Some(f) if f == expected => println!("ok      {ARTIFACTS_FILE} {kind}"),
                    Some(f) => {
                        drift = true;
                        println!(
                            "DRIFT   {ARTIFACTS_FILE} {kind}: expected `{expected}`, found `{f}`"
                        );
                    }
                    None => {
                        drift = true;
                        println!("MISSING {ARTIFACTS_FILE} {kind} row");
                    }
                }
            }
        }
        Err(e) => {
            drift = true;
            println!("MISSING {ARTIFACTS_FILE}: {e}");
        }
    }
    if drift {
        eprintln!(
            "registry drift: run `cargo run --features abi --bin graphlaw-registry -- --write`"
        );
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn write() -> ExitCode {
    let dir = registry_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return usage(&format!("cannot create {}: {e}", dir.display()));
    }
    for (file, body) in [
        ("capability-registry.json", registry_json_pretty()),
        ("capability-registry.ttl", registry_turtle()),
        ("mcp-manifest.json", mcp_manifest_pretty()),
    ] {
        if let Err(e) = std::fs::write(dir.join(file), body) {
            return usage(&format!("cannot write {file}: {e}"));
        }
        println!("wrote {}", dir.join(file).display());
    }
    // Re-pin the owned ARTIFACTS.blake3 rows from live computed digests. The wasm
    // row pins a build artifact and is preserved verbatim (never touched here).
    let artifacts_path = dir.join(ARTIFACTS_FILE);
    let existing = std::fs::read_to_string(&artifacts_path).ok();
    let body = match artifacts_repin(existing.as_deref()) {
        Ok(b) => b,
        Err(e) => return usage(&e),
    };
    if let Err(e) = std::fs::write(&artifacts_path, &body) {
        return usage(&format!("cannot write {ARTIFACTS_FILE}: {e}"));
    }
    println!("repinned {}", artifacts_path.display());
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [a] if a == "--write" => write(),
        [a] if a == "--check" => check(),
        [a] if a == "--print-json" => {
            print!("{}", registry_json_pretty());
            ExitCode::SUCCESS
        }
        [a] if a == "--print-ttl" => {
            print!("{}", registry_turtle());
            ExitCode::SUCCESS
        }
        [a] if a == "--print-mcp" => {
            print!("{}", mcp_manifest_pretty());
            ExitCode::SUCCESS
        }
        [a] if a == "--print-digests" => {
            println!("surface_blake3  {}", surface_blake3());
            println!("registry_blake3 {}", registry_blake3());
            println!("mcp-manifest    {}", mcp_manifest_blake3());
            ExitCode::SUCCESS
        }
        [a] if a == "--help" || a == "-h" => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        _ => usage(
            "expected exactly one of --write, --check, --print-json, --print-ttl, --print-mcp, --print-digests (try --help)",
        ),
    }
}
