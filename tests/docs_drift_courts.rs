//! Docs-drift courts: prose claims vs. the real on-disk state they describe.
//!
//! Real-state (Chicago): every court reads the claim file AND its source of
//! truth from disk — no mocks, no cached constants — and fails naming the
//! file+phrase that drifted. Deleting a claim line panics (missing anchor);
//! rewording a value fails the equality assert.
//!
//! Anchor matching is whitespace-tolerant: each doc is scanned through a
//! byte-for-byte 1:1 copy with `\n` mapped to ` ` (markdown line wraps cannot
//! hide a claim), and every offset found there is valid in the raw text.
//!
//! Status 2026-10-24 (execution lane finC):
//! - `runner_doc_fuel_matches_fuel_per_request_const` — GREEN.
//! - `doc_pin_claims_match_on_disk_pins` — RED until docs/WASM.md's pin
//!   sections are re-pinned to the live mint; README.md already matches.
//! - `eyeron_version_prose_matches_manifest` — RED until CHANGELOG.md's
//!   upstream-vendored note says 0.7.9 (README/UPSTREAM already match).

use std::fs;
use std::path::Path;

const RUNNER_SRC: &str = "src/bin/graphlaw_run_eh.rs";
const RUNNER_DOC: &str = "docs/reference/runner-exit-codes.md";
const PRIV_PIN: &str = "priv/graphlaw.wasm.blake3";
const REGISTRY: &str = "registry/ARTIFACTS.blake3";
const README: &str = "README.md";
const WASM_DOC: &str = "docs/WASM.md";
const EYERON_MANIFEST: &str = "crates/graphlaw-eyeron/Cargo.toml";
const CHANGELOG: &str = "CHANGELOG.md";
const EYERON_UPSTREAM: &str = "crates/graphlaw-eyeron/UPSTREAM.md";

fn read(rel: &str) -> String {
    fs::read_to_string(Path::new(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// Byte-for-byte 1:1 copy with `\n` mapped to ` `: line wraps cannot hide an
/// anchor, and every offset in the copy is valid in the original.
fn norm_of(text: &str) -> String {
    text.replace('\n', " ")
}

/// Maximal digit/underscore runs immediately following every occurrence of
/// `anchor` in the (newline-normalized) text, parsed as u64 (Rust-style `_`
/// separators stripped). Panics name the anchor and the offending lexical.
fn numbers_after(src: &str, anchor: &str) -> Vec<u64> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    let mut from = 0;
    while let Some(rel) = src[from..].find(anchor) {
        let s = from + rel + anchor.len();
        let mut e = s;
        while e < bytes.len() && (bytes[e].is_ascii_digit() || bytes[e] == b'_') {
            e += 1;
        }
        let lex: String = src[s..e].chars().filter(|c| *c != '_').collect();
        let n: u64 = lex.parse().unwrap_or_else(|err| {
            panic!("no parseable number right after {anchor:?} (lexical {lex:?}): {err}")
        });
        out.push(n);
        from = e;
    }
    out
}

/// Blank-line-delimited paragraph in `raw` containing byte offset `pos`.
fn paragraph_at<'a>(raw: &'a str, pos: usize) -> &'a str {
    let start = raw[..pos].rfind("\n\n").map(|i| i + 1).unwrap_or(0);
    let end = raw[pos..]
        .find("\n\n")
        .map(|i| pos + i)
        .unwrap_or(raw.len());
    &raw[start..end]
}

/// Locate `anchor` (whitespace-tolerant) or panic naming file and phrase.
fn locate(norm: &str, anchor: &str, file: &str) -> usize {
    norm.find(anchor).unwrap_or_else(|| {
        panic!("{file}: missing required anchor {anchor:?} — claim line deleted or reworded")
    })
}

/// First maximal hex run (>= 8 chars) after `anchor`, searched from `from`,
/// stopping at the first non-hex char — elision-tolerant, so `fa155830…`
/// reads as `fa155830`.
fn hex_token_after(norm: &str, from: usize, anchor: &str, file: &str) -> String {
    let pos = norm[from..]
        .find(anchor)
        .map(|rel| from + rel + anchor.len())
        .unwrap_or_else(|| {
            panic!("{file}: missing required anchor {anchor:?} — claim line deleted or reworded")
        });
    let bytes = norm.as_bytes();
    let mut e = pos;
    while e < bytes.len() && bytes[e].is_ascii_hexdigit() {
        e += 1;
    }
    let token = &norm[pos..e];
    assert!(
        token.len() >= 8,
        "{file}: hex token right after {anchor:?} is {token:?} — shorter than 8 hex chars"
    );
    token.to_ascii_lowercase()
}

/// Digit run (comma separators stripped) immediately following `anchor`,
/// searched from `from` — e.g. `is **57,781,384 bytes**` under `is **`.
fn digits_after(norm: &str, from: usize, anchor: &str, file: &str) -> u64 {
    let pos = norm[from..]
        .find(anchor)
        .map(|rel| from + rel + anchor.len())
        .unwrap_or_else(|| {
            panic!("{file}: missing required anchor {anchor:?} — claim line deleted or reworded")
        });
    let digits: String = norm[pos..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .filter(|c| *c != ',')
        .collect();
    digits.parse().unwrap_or_else(|err| {
        panic!("{file}: no parseable byte-size after {anchor:?} ({digits:?}): {err}")
    })
}

/// `(<digits[,digits…]> B)` byte-size following `anchor`, searched from
/// `from`, comma separators stripped.
fn paren_size_after(norm: &str, from: usize, anchor: &str, file: &str) -> u64 {
    let pos = norm[from..]
        .find(anchor)
        .map(|rel| from + rel + anchor.len())
        .unwrap_or_else(|| {
            panic!("{file}: missing required anchor {anchor:?} — claim line deleted or reworded")
        });
    let rest = &norm[pos..];
    let open = rest.find('(').unwrap_or_else(|| {
        panic!("{file}: no '(' byte-size after {anchor:?}")
    });
    let digits: String = rest[open + 1..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .filter(|c| *c != ',')
        .collect();
    digits.parse().unwrap_or_else(|err| {
        panic!("{file}: no parseable byte-size in parens after {anchor:?} ({digits:?}): {err}")
    })
}

/// Section starting at `start_anchor`, bounded by the next `\n##` heading.
/// Returns (section byte range in raw, start offset in the normalized copy).
fn section_between<'a>(raw: &'a str, norm: &str, start_anchor: &str, file: &str) -> &'a str {
    let pos = locate(norm, start_anchor, file);
    let from = pos + start_anchor.len();
    let end = raw[from.min(raw.len())..]
        .find("\n##")
        .map(|i| from + i + 1)
        .unwrap_or(raw.len());
    &raw[pos..end]
}

/// Maximal hex runs (>= 8 chars) in raw text with their byte offsets.
fn hex_runs(text: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_hexdigit() {
            let s = i;
            while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                i += 1;
            }
            if i - s >= 8 {
                out.push((text[s..i].to_ascii_lowercase(), s));
            }
        } else {
            i += 1;
        }
    }
    out
}

fn has_historical_marker(para: &str) -> bool {
    let low = para.to_ascii_lowercase();
    [
        "historical",
        "prior",
        "superseded",
        "legacy",
        "pre-closure",
        "previous",
    ]
    .iter()
    .any(|kw| low.contains(kw))
}

/// Sweep a scoped claim region: every hex run >= 8 that is NOT a prefix of
/// the live digest must sit in a paragraph marking it historical.
fn sweep_stale_hex(region_raw: &str, region_name: &str, live_digest: &str, failures: &mut Vec<String>) {
    for (run, pos) in hex_runs(region_raw) {
        if live_digest.starts_with(run.as_str()) {
            continue;
        }
        let para = paragraph_at(region_raw, pos);
        if !has_historical_marker(para) {
            failures.push(format!(
                "{region_name}: stale hex token {run} is presented without a historical/\
                 prior/superseded/legacy/pre-closure/previous marker in its paragraph"
            ));
        }
    }
}

/// Court (a): the runner doc's fuel claims must equal `FUEL_PER_REQUEST`.
/// Follows the source: the const is parsed out of `graphlaw_run_eh.rs`, not
/// restated here, so a source-side flip fails this court too.
#[test]
fn runner_doc_fuel_matches_fuel_per_request_const() {
    let src = read(RUNNER_SRC);
    const CONST_ANCHOR: &str = "const FUEL_PER_REQUEST: u64 = ";
    let sites = numbers_after(&src, CONST_ANCHOR);
    assert_eq!(
        sites.len(),
        1,
        "{RUNNER_SRC}: expected exactly 1 {CONST_ANCHOR:?} site, found {}",
        sites.len()
    );
    let fuel = sites[0];

    let doc = read(RUNNER_DOC);
    let mut failures: Vec<String> = Vec::new();
    for (anchor, min) in [
        ("fuel budget of `", 1usize),
        ("`FUEL_PER_REQUEST` = ", 1),
        ("guest exhausted fuel budget ", 2),
    ] {
        let n = doc.matches(anchor).count();
        if n < min {
            failures.push(format!(
                "{RUNNER_DOC}: anchor {anchor:?} appears {n}x, court requires >= {min}"
            ));
        }
    }

    for (anchor, vals) in [
        ("fuel budget of `", numbers_after(&doc, "fuel budget of `")),
        (
            "`FUEL_PER_REQUEST` = ",
            numbers_after(&doc, "`FUEL_PER_REQUEST` = "),
        ),
        (
            "guest exhausted fuel budget ",
            numbers_after(&doc, "guest exhausted fuel budget "),
        ),
    ] {
        if vals.is_empty() {
            continue; // already recorded as a missing-anchor failure above
        }
        for v in vals {
            if v != fuel {
                failures.push(format!(
                    "{RUNNER_DOC}: fuel value {v} after {anchor:?} != FUEL_PER_REQUEST {fuel}"
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "runner-exit-codes.md drifted from FUEL_PER_REQUEST ({fuel}):\n  {}",
        failures.join("\n  ")
    );
}

/// Court (b): README/WASM.md pin prose must match the two on-disk pins
/// (priv sidecar + registry wasm row), and stale hex inside the scoped claim
/// regions must be explicitly marked historical.
#[test]
fn doc_pin_claims_match_on_disk_pins() {
    let mut failures: Vec<String> = Vec::new();

    let live_digest = read(PRIV_PIN);
    let live_digest = live_digest
        .split_whitespace()
        .next()
        .unwrap_or_else(|| panic!("{PRIV_PIN}: no digest token"))
        .to_ascii_lowercase();

    let registry = read(REGISTRY);
    let row = registry
        .lines()
        .find(|l| l.split_whitespace().next() == Some("wasm"))
        .unwrap_or_else(|| panic!("{REGISTRY}: no `wasm` row"));
    let fields: Vec<&str> = row.split_whitespace().collect();
    let reg_digest = fields
        .get(1)
        .unwrap_or_else(|| panic!("{REGISTRY}: wasm row has no digest field"))
        .to_ascii_lowercase();
    let reg_bytes: u64 = fields
        .get(2)
        .unwrap_or_else(|| panic!("{REGISTRY}: wasm row has no byte-size field"))
        .parse()
        .unwrap_or_else(|err| panic!("{REGISTRY}: wasm row byte-size unparsable: {err}"));
    assert_eq!(
        reg_digest, live_digest,
        "pin files disagree: {PRIV_PIN} says {live_digest}, {REGISTRY} wasm row says {reg_digest}"
    );

    let readme_raw = read(README);
    let readme = norm_of(&readme_raw);
    let readme_anchor = "The pinned artifact in `registry/ARTIFACTS.blake3`";
    let pos = locate(&readme, readme_anchor, README);
    let region = paragraph_at(&readme_raw, pos);
    let readme_token = hex_token_after(&readme, pos, "blake3 `", README);
    let readme_size = digits_after(&readme, pos, "is **", README);
    if !live_digest.starts_with(readme_token.as_str()) {
        failures.push(format!(
            "{README}: pinned-digest token {readme_token} is not a prefix of the live pin {live_digest}"
        ));
    }
    if readme_size != reg_bytes {
        failures.push(format!(
            "{README}: pinned size {readme_size} != registry wasm row {reg_bytes}"
        ));
    }
    sweep_stale_hex(region, README, &live_digest, &mut failures);

    let wasm_raw = read(WASM_DOC);
    let wasm = norm_of(&wasm_raw);
    let cur_pos = locate(&wasm, "The current pin is `", WASM_DOC);
    let cur_token = hex_token_after(&wasm, cur_pos, "The current pin is `", WASM_DOC);
    let cur_size = paren_size_after(&wasm, cur_pos, "The current pin is `", WASM_DOC);
    if !live_digest.starts_with(cur_token.as_str()) {
        failures.push(format!(
            "{WASM_DOC}: current-pin token {cur_token} is not a prefix of the live pin {live_digest}"
        ));
    }
    if cur_size != reg_bytes {
        failures.push(format!(
            "{WASM_DOC}: current-pin size {cur_size} != registry wasm row {reg_bytes}"
        ));
    }
    let cur_region = paragraph_at(&wasm_raw, cur_pos);
    sweep_stale_hex(cur_region, WASM_DOC, &live_digest, &mut failures);

    let section = section_between(&wasm_raw, &wasm, "### Pin currency", WASM_DOC);
    let sec_pos = locate(&wasm, "### Pin currency", WASM_DOC);
    let disk_token = hex_token_after(&wasm, sec_pos, "The on-disk pins read `", WASM_DOC);
    let disk_size = paren_size_after(&wasm, sec_pos, "The on-disk pins read `", WASM_DOC);
    if !live_digest.starts_with(disk_token.as_str()) {
        failures.push(format!(
            "{WASM_DOC}: on-disk-pins token {disk_token} is not a prefix of the live pin {live_digest}"
        ));
    }
    if disk_size != reg_bytes {
        failures.push(format!(
            "{WASM_DOC}: on-disk-pins size {disk_size} != registry wasm row {reg_bytes}"
        ));
    }
    sweep_stale_hex(section, WASM_DOC, &live_digest, &mut failures);

    assert!(
        failures.is_empty(),
        "doc pin claims drifted from the on-disk pins (live {live_digest}, {reg_bytes} B):\n  {}",
        failures.join("\n  ")
    );
}

/// Court (c): eyeron version prose (README vendored line, CHANGELOG upstream
/// note, UPSTREAM.md self-version) must equal the vendored crate's manifest
/// version, parsed table-scoped from `[package]`.
#[test]
fn eyeron_version_prose_matches_manifest() {
    let manifest = read(EYERON_MANIFEST);
    let pkg_pos = manifest.find("[package]").unwrap_or_else(|| {
        panic!("{EYERON_MANIFEST}: missing [package] table")
    });
    let mut version: Option<String> = None;
    for line in manifest[pkg_pos..].lines().skip(1) {
        let t = line.trim();
        if t.starts_with('[') {
            break; // left the [package] table
        }
        if let Some(rest) = t.strip_prefix("version") {
            let rest = rest.trim_start();
            if let Some(val) = rest.strip_prefix('=') {
                version = Some(val.trim().trim_matches('"').to_string());
                break;
            }
        }
    }
    let version = version.unwrap_or_else(|| {
        panic!("{EYERON_MANIFEST}: no `version` key inside [package]")
    });

    let mut failures: Vec<String> = Vec::new();

    let readme = read(README);
    let lines: Vec<&str> = readme
        .lines()
        .filter(|l| l.contains("vendored `crates/graphlaw-eyeron`"))
        .collect();
    if lines.len() != 1 {
        failures.push(format!(
            "{README}: expected exactly 1 line naming the vendored `crates/graphlaw-eyeron`, \
             found {}\n  {lines:#?}",
            lines.len()
        ));
    } else if !lines[0].contains(&format!("Eyeron {version}")) {
        failures.push(format!(
            "{README}: vendored-eyeron line does not say \"Eyeron {version}\":\n  {}",
            lines[0]
        ));
    }

    let changelog = read(CHANGELOG);
    let anchor = "upstream-vendored version (currently ";
    let claimed = match changelog.find(anchor) {
        Some(pos) => {
            let rest = &changelog[pos + anchor.len()..];
            match rest.find(')') {
                Some(end) => rest[..end].trim().to_string(),
                None => {
                    failures.push(format!("{CHANGELOG}: no ')' after {anchor:?}"));
                    String::new()
                }
            }
        }
        None => {
            failures.push(format!("{CHANGELOG}: missing anchor {anchor:?}"));
            String::new()
        }
    };
    if !claimed.is_empty() && claimed != version {
        failures.push(format!(
            "{CHANGELOG}: upstream-vendored version says {claimed}, manifest says {version}"
        ));
    }

    let upstream = read(EYERON_UPSTREAM);
    let anchor2 = "This crate's own version is ";
    let self_v = match upstream.find(anchor2) {
        Some(pos) => {
            let rest = &upstream[pos + anchor2.len()..];
            let token: String = rest
                .chars()
                .take_while(|c| !c.is_whitespace())
                .collect();
            token
                .trim_end_matches(|c: char| ".,;)".contains(c))
                .to_string()
        }
        None => {
            failures.push(format!("{EYERON_UPSTREAM}: missing anchor {anchor2:?}"));
            String::new()
        }
    };
    if !self_v.is_empty() && self_v != version {
        failures.push(format!(
            "{EYERON_UPSTREAM}: self-version says {self_v}, manifest says {version}"
        ));
    }

    assert!(
        failures.is_empty(),
        "eyeron version prose drifted from manifest version {version}:\n  {}",
        failures.join("\n  ")
    );
}
