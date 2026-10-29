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
    ("mock", "all"),     // mockall crate / macro
    ("auto", "mock!"),   // automock! attribute macro
    ("jest", ".mock"),   // jest.mock( class-of patterns
    ("vi", ".mock"),     // vitest vi.mock(
    ("sinon", ".stub"),  // sinon.stub(
    ("struct Mock", ""), // cfg(test) Mock* double structs
];

/// Documented non-violations. Key = path relative to the manifest root,
/// value = justification (Chicago doctrine: hand-written real-interface
/// implementation of the real `RdfAstGraphReader` trait over real data).
const ALLOWLIST: &[(&str, &str)] = &[(
    "tests/no_mock_markers.rs",
    "this gate: marker literals are split so the scanner cannot match itself.",
)];

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

// ---------------------------------------------------------------------------
// X1 gates (wave W defect class): (a) test-local oracle laundering, (b)
// `unwrap_or`-class default laundering inside tests/.
//
// (a) Defect: a test file defines its own decision closure/oracle
//     (`fn <name>(...) -> ...`) and then asserts *on that fn* nearby — the
//     test verifies its own model instead of real collaborator state
//     (`[[testing-chicago-style]]`, London rejected). Heuristic: an assert
//     macro referencing the fn name within ORACLE_WINDOW_LINES of the def.
//     Existing sanctioned helpers (real readbacks over real state,
//     dual-anchored golden digests) are allowlisted below with justification.
// (b) Defect: `.unwrap_or(...)` / `.unwrap_or_default()` inside tests/
//     launders a failed read into a default the test then "verifies"
//     (`[[no-overclaiming-rust]]`: `unwrap_or_default()` on a meaningful
//     Result is failure laundered into "ran fine"). Sanctioned uses today
//     are serde_json field probes on already-parsed envelopes and
//     environment defaults — allowlisted literally below.
// ---------------------------------------------------------------------------

/// How many lines after `fn <name>(...) ->` an assert on `<name>(` still
/// counts as asserting on a test-local oracle.
const ORACLE_WINDOW_LINES: usize = 40;

/// Extract the fn name from a line that declares `fn <name>(...) -> ...`.
/// Returns None for `fn name()` test fns (no written return type) and for
/// non-fn lines.
fn oracle_fn_decl(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let rest = trimmed
        .strip_prefix("pub ")
        .and_then(|r| r.strip_prefix("fn "))
        .or_else(|| trimmed.strip_prefix("fn "))?;
    let open = rest.find('(')?;
    if !trimmed.contains("->") {
        return None; // unit-test fns and helpers without a written return type
    }
    let name = rest[..open].trim();
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    Some(name.to_string())
}

/// True when the line is an assert-macro invocation mentioning `name(`.
fn asserts_on(line: &str, name: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("assert") && trimmed.contains(&format!("{name}("))
}

/// Sanctioned test-local helper fns: (file, fn name, justification).
/// These are real readbacks over real collaborator state (real DuckDB rows,
/// real on-disk ledgers, real response envelopes) or dual-anchored goldens
/// that ALSO pin against an independent constant — not London-style
/// assert-on-your-own-model oracles.
const ORACLE_ALLOWLIST: &[(&str, &str, &str)] = &[
    (
        "tests/set_operations_court.rs",
        "column_values",
        "read-back helper over the REAL ExecutionReceipt.result_triples (frozen-duckdb          rows), sorted; the expectation values live in the courts, not in this helper —          proximity heuristic misfire, dual-anchored asserts at each use site.",
    ),
    (
        "tests/abi_manifest_court.rs",
        "expected_hash_bytes",
        "dual-anchored: assert_eq against expected_hash_bytes AND against the pinned \
         GOLDEN_FRAME_HASH_HEX constant, so the golden does not verify only itself.",
    ),
    (
        "tests/evidentiary_ring_court.rs",
        "desc",
        "constructor for real ring entries; asserts target the real EvidenceRing and its \
         on-disk ledger, not the constructor itself.",
    ),
    (
        "tests/k5_hostile_sqlrdf.rs",
        "hostile_t_survives",
        "real DuckDB readback (query_row over the real connection); failure maps to false \
         which fails the assert loudly, never laundered.",
    ),
    (
        "tests/law_state.rs",
        "select_count",
        "real SPARQL engine readback over real LawState dataset; asserts pin real derived \
         counts, not the helper's own arithmetic.",
    ),
    (
        "tests/op_differential.rs",
        "s",
        "real response-envelope field extractor; loud unwrap_or_else panic on absence.",
    ),
    (
        "tests/op_forward_compat.rs",
        "envelope_ok",
        "real response-envelope probe (serde_json over real wasm bytes) driving declared-\
         outcome controls on both runtimes.",
    ),
    (
        "tests/plan_negation.rs",
        "req",
        "request builder, not a decision oracle; asserts target the real kernel response.",
    ),
    (
        "tests/receipt_feedback.rs",
        "plan",
        "fixture builder; digest asserts are dual-anchored (state vs builder digest vs \
         mutated variants, must differ).",
    ),
    (
        "tests/refusal_crosswalk_differential.rs",
        "blake3_hex",
        "hash helper over real pinned fixture bytes; asserts compare against the pinned \
         manifest.blake3, an external constant.",
    ),
    (
        "tests/registry_dispatch.rs",
        "message",
        "real response-envelope field extractor; asserts the real kernel error text.",
    ),
    (
        "tests/receipt_lease.rs",
        "base",
        "fixture builder (real LawState parse, loud unwrap); asserts target the real kernel transitions.",
    ),
    (
        "tests/strata_gate_differential.rs",
        "blake3_hex",
        "hash helper over real pinned fixture bytes; asserts compare against the pinned \
         manifest.blake3, an external constant.",
    ),
    (
        "tests/strata_gate_differential.rs",
        "hex",
        "encoding helper; asserts compare against pinned manifest.blake3 constants.",
    ),
    (
        "tests/wasm_abi.rs",
        "refusal_of",
        "real response-envelope extractor; asserts pinned (code, reason) pairs from the \
         real kernel refusal surface.",
    ),
    (
        "tests/law_state.rs",
        "read",
        "fixture fs readback with loud unwrap; asserted content comes from the real kernel.",
    ),
    (
        "tests/verify_cli.rs",
        "stdout",
        "real subprocess Output extractor; asserts pin real CLI stdout bytes.",
    ),
    (
        "tests/smon_cli_court.rs",
        "stderr",
        "real subprocess Output extractor; asserts pin real CLI stderr text.",
    ),
    (
        "tests/knowledge_hooks.rs",
        "obligations",
        "real SPARQL engine readback over real LawState dataset; asserts pin real derived counts.",
    ),
    (
        "tests/chicago_boundary_constraints.rs",
        "violation_component",
        "real response-envelope extractor with loud unwrap_or_else panic; asserts pinned component names.",
    ),
];

/// Sanctioned `.unwrap_or` / `.unwrap_or_default` uses: (file, line snippet,
/// justification). A hit is sanctioned only when the same file's line contains
/// the snippet literally — widening a snippet is an allowlist edit, reviewed.
const UNWRAP_ALLOWLIST: &[(&str, &str, &str)] = &[
    (
        "tests/wasm_env_isolation_court.rs",
        ".copied().unwrap_or(0),",
        "chunk-slot probe on received wasm response bytes: absent slot -> 0 is the          compared value; already-parsed envelope, data-shape not a laundered failure.",
    ),
    (
        "tests/attach_bypass_court.rs",
        ".as_str().unwrap_or(\"\");",
        "serde_json field probe on an already-parsed refusal envelope; feeds          contains-assert directly (sanctioned class per header).",
    ),
    (
        "tests/receipt_integrity_court.rs",
        ".filter(|p| p.extension().map(|x| x == \"md\").unwrap_or(false))",
        "receipt-extension probe: no extension -> false is the CORRECT classification;          builds the scanned-receipt set.",
    ),
    // serde_json Value field probes on already-parsed envelopes: the document
    // parsed successfully; a missing optional string field is data-shape, not
    // a laundered failure. In every case the extracted value feeds a
    // subsequent assert or is asserted directly.
    (
        "tests/sql_rdf_dialect.rs",
        ".unwrap_or(\"\")",
        "serde_json optional-field probe on an already-parsed envelope; extracted values feed assert/starts_with.",
    ),
    (
        "tests/sql_rdf_dialect.rs",
        "as_u64().unwrap_or(0)",
        "serde_json optional-field probe; `> 0` assert fails loud on absence.",
    ),
    (
        "tests/op_examples.rs",
        ".unwrap_or(Value::Null)",
        "serde_json comparison default inside example-table diff.",
    ),
    (
        "tests/wasm_memory_wall.rs",
        "as_str().unwrap_or(",
        "serde_json error-envelope message probe; asserted downstream.",
    ),
    (
        "tests/wasm_abi.rs",
        "as_str().unwrap_or(",
        "serde_json error-envelope code/reason probes; fed into assert_eq against pinned refusals.",
    ),
    (
        "tests/wasm_abi.rs",
        "as_u64().unwrap_or(0)",
        "serde_json optional-field probe; `> 0` assert fails loud on absence.",
    ),
    (
        "tests/wasm_duckdb_op.rs",
        "unwrap_or_default()",
        "serde_json error-envelope message probe feeding a contains() assert.",
    ),
    (
        "tests/wasm_duckdb_op_wired.rs",
        "unwrap_or_default()",
        "serde_json error-envelope message probe feeding a contains() assert.",
    ),
    (
        "tests/wasm_governance_preflight.rs",
        "unwrap_or_default()",
        "serde_json optional report-field extraction; fields asserted downstream.",
    ),
    (
        "tests/k6_hostile_lease.rs",
        "unwrap_or(",
        "serde_json error-envelope code/reason probes; asserted against pinned refusal pairs.",
    ),
    (
        "tests/wasm_boundary_fuzz.rs",
        "as_str().unwrap_or(",
        "serde_json error-envelope probes over fuzz responses; kind/message asserted.",
    ),
    (
        "tests/registry_dispatch.rs",
        "unwrap_or(",
        "serde_json envelope probes; summary/message asserted downstream.",
    ),
    (
        "tests/wasm_native_differential.rs",
        "unwrap_or(\"?\")",
        "diagnostic label for a diff report line, not a decision input.",
    ),
    (
        "tests/corpus_conformance.rs",
        "unwrap_or(",
        "file-extension probe for a diagnostic path label.",
    ),
    (
        "tests/duckdb_statement_kinds.rs",
        "unwrap_or(",
        "statement-name parse from an already-matched line; name used in the test's message.",
    ),
    (
        "tests/pn64_abi_conformance.rs",
        "unwrap_or(",
        "comment-stripped line probe for a diagnostic label.",
    ),
    // Genuine environment/numeric defaults where None is a lawful value.
    (
        "tests/wasm_abi.rs",
        "tamper_expires.unwrap_or(expires)",
        "test parameter default: no tamper means the honest expiry; the tampered variant is separately asserted to be refused.",
    ),
    (
        "tests/p28_hot_path_bench.rs",
        "unwrap_or(0)",
        "available_parallelism default for a bench metadata label, not a pass/fail input.",
    ),
    (
        "tests/evidentiary_ring_court.rs",
        "unwrap_or(u32::MAX)",
        "saturating ns->u32 clamp on a real measured sample; sentinel keeps the outlier visible in the ring.",
    ),
    (
        "tests/wasm_boundary_fuzz.rs",
        "unwrap_or(false)",
        "metadata().modified() comparison default on a fuzz temp dir; drives a bounded poll, not an assertion of success.",
    ),
    (
        "tests/registry_artifacts.rs",
        "unwrap_or_default()",
        "read_to_string probe of an optional artifact path; empty string fails the subsequent equality assert loud.",
    ),
    // The scanner file itself: allowlist snippets below would otherwise match
    // the literal allowlist text. Skipped structurally in scan_unwrap_or.
    // k5 readback: query_row failure maps to false, which FAILS the
    // survival assert — loud, the opposite of laundering (also in
    // ORACLE_ALLOWLIST under hostile_t_survives).
    (
        "tests/k5_hostile_sqlrdf.rs",
        ".unwrap_or(false)",
        "DuckDB count readback; false fails the survival assert loud.",
    ),
    (
        "tests/evidentiary_ring_court.rs",
        ".unwrap_or(0)",
        "ledger-line-count readback for a bounded poll; 0 keeps the poll waiting and times out loud rather than passing.",
    ),
];

/// Scan one file's text for the X1 oracle heuristic. `rel` is the
/// manifest-relative path used for allowlist lookup.
fn scan_oracle(rel: &str, text: &str) -> Vec<String> {
    let mut hits = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let Some(name) = oracle_fn_decl(line) else {
            continue;
        };
        let allowed = ORACLE_ALLOWLIST
            .iter()
            .any(|(f, n, _)| *f == rel && *n == name);
        if allowed {
            continue;
        }
        for nxt in &lines[i + 1..(i + 1 + ORACLE_WINDOW_LINES).min(lines.len())] {
            if asserts_on(nxt, &name) {
                hits.push(format!(
                    "{rel}:{}: test-local oracle `{name}` is asserted on within \
                     {ORACLE_WINDOW_LINES} lines of its def — the test verifies its own \
                     model (Chicago doctrine). Assert on real collaborator state, or \
                     allowlist the helper with justification in ORACLE_ALLOWLIST.",
                    i + 1
                ));
                break;
            }
        }
    }
    hits
}

/// Scan one file's text for `.unwrap_or(` / `.unwrap_or_default()` laundering.
fn scan_unwrap_or(rel: &str, text: &str) -> Vec<String> {
    // This gate file carries the snippet literals of the allowlist itself;
    // it cannot scan its own text without self-matching.
    if rel == "tests/no_mock_markers.rs" {
        return Vec::new();
    }
    let mut hits = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let violation = line.contains(".unwrap_or(") || line.contains(".unwrap_or_default()");
        if !violation {
            continue;
        }
        let sanctioned = UNWRAP_ALLOWLIST
            .iter()
            .any(|(f, snippet, _)| *f == rel && line.contains(snippet));
        if sanctioned {
            continue;
        }
        hits.push(format!(
            "{rel}:{}: unwrap_or-class default inside tests/ launders a failed read into \
             \"ran fine\" (Chicago doctrine, [[no-overclaiming-rust]]). Fail loud, assert \
             on the real readback, or allowlist the exact line snippet with justification \
             in UNWRAP_ALLOWLIST.\n    line: {}",
            n + 1,
            line.trim()
        ));
    }
    hits
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

#[test]
fn no_test_local_oracles_or_unwrap_or_laundering_in_tests() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut hits = Vec::new();
    let dir = root.join("tests");
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs") {
            let text = fs::read_to_string(&path).unwrap();
            let rel = format!("tests/{}", path.file_name().unwrap().to_string_lossy());
            hits.extend(scan_oracle(&rel, &text));
            hits.extend(scan_unwrap_or(&rel, &text));
        }
    }
    assert!(
        hits.is_empty(),
        "X1 laundering in tests/ (test-local oracle asserted on, or unwrap_or-class \
         default on a real read):\n{}",
        hits.join("\n")
    );
}

#[test]
fn gate_flags_planted_violations_nonvacuity() {
    let planted = format!(
        "planted_x1_violation_{}.rs",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = std::env::temp_dir().join(&planted);
    fs::write(
        &path,
        // Within the 40-line window: fn decl with return type, then an assert
        // naming the fn, plus an unsanctioned unwrap_or on a real read.
        "fn my_oracle(x: u64) -> u64 {\n\
         \x20   x * 2\n\
         }\n\
         #[test]\n\
         fn t() {\n\
         \x20   let read: Result<u64, String> = Err(\"disk gone\".into());\n\
         \x20   let v = read.clone().map_err(|_| 0u64).unwrap_or(0);\n\
         \x20   assert_eq!(my_oracle(v), 0);\n\
         }\n",
    )
    .expect("planted /tmp violation file must be writable");
    let text = fs::read_to_string(&path).unwrap();
    let rel = format!("tests/{planted}");
    let oracle_hits = scan_oracle(&rel, &text);
    let unwrap_hits = scan_unwrap_or(&rel, &text);
    let cleanup = || fs::remove_file(&path).unwrap();
    assert!(
        !oracle_hits.is_empty() && oracle_hits[0].contains("my_oracle"),
        "anti-vacuity: planted test-local oracle must be flagged, got {oracle_hits:?}"
    );
    assert!(
        !unwrap_hits.is_empty() && unwrap_hits[0].contains("unwrap_or"),
        "anti-vacuity: planted unwrap_or laundering must be flagged, got {unwrap_hits:?}"
    );
    // A clean file (same oracle, no nearby assert, no unwrap_or) is not flagged.
    let clean = "fn helper(x: u64) -> u64 {\n    x + 1\n}\n";
    assert!(
        scan_oracle(&rel, clean).is_empty(),
        "clean helper outside the assert window must not be flagged"
    );
    cleanup();
}
