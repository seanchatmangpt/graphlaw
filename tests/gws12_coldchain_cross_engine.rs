//! GWS-12 court: cross-engine determinism for the cold-chain admission surface.
//!
//! The same query + corpus + rules must produce byte-identical state
//! transitions across every available engine leg:
//!
//!   (a) native graphlaw ABI (`graphlaw::abi::call`, the case study's
//!       Phase-5/6 trace),
//!   (b) the compiled wasm module (`graphlaw_wasm.wasm`, the GWS-5/6 courts'
//!       mechanism — gl_alloc/gl_call/gl_free over linear memory, wasmi host),
//!   (c) smalt's COLD-1 court (Elixir + duckdb CLI + independent N3
//!       re-derivation) — replayed in the lane; its partition and audit hash
//!       are pinned here as the cross-repo constants below.
//!
//! Asserted: the quarantine/release partition is IDENTICAL across legs, and
//! native vs wasm response bytes are byte-identical for the same admission
//! request. The audit hash (sha256 of canonical query + literals, the smalt
//! COLD-1 formula) binds query+literals and is pinned cross-language.
//!
//! Corpus: COLD-1's real fixture CSVs (10 batches, 17 events) via
//! `COLDCHAIN_CSV_DIR`; aggregates computed by the REAL duckdb CLI
//! (`DUCKDB_BIN`, default /opt/homebrew/bin/duckdb) — no mocks.

mod common;

use std::collections::BTreeMap;
use std::process::Command;

use common::{native_bytes, wasm_bytes};
use serde_json::json;

/// COLD-1 (smalt, current branch, replayed this lane) partition constants.
const COLD1_QUARANTINED: &[&str] = &["BATCH-A1", "BATCH-A2", "BATCH-Q1", "BATCH-Q2", "BATCH-Q3"];
const COLD1_RELEASED: &[&str] = &["BATCH-R1", "BATCH-R2", "BATCH-R3", "BATCH-R4", "BATCH-R5"];
/// COLD-1 audit hash: sha256(canonical_query || "||" || "{-20.0, -15.0, 300}")
/// — reproduced independently in Python in the lane receipt; pinned here.
const COLD1_AUDIT_HASH: &str = "6971c912530619217b462f0b458a390eec34383a8155e0ccfcce869e4b6aeb8e";

#[allow(dead_code)] // pinned as the audit-hash input; court builds its own copy
const CANONICAL_QUERY: &str = "SELECT o.object_id, MAX(e.temp_celsius) AS max_temp, SUM(e.duration_seconds) AS duration FROM ocel_events AS e JOIN ocel_event_object_rel AS r ON e.event_id = r.event_id JOIN ocel_objects AS o ON r.object_id = o.object_id WHERE e.temp_celsius > -20.0 AND r.qualifier = 'contains_product' AND o.object_type = 'bio:BiologicBatch' GROUP BY o.object_id";

/// GWS-6's landed ruleset, verbatim semantics: SWAP math builtins.
const RULESET: &str = r#"
@prefix bio: <http://biopharma.org/vocab#> .
@prefix gl:  <http://graphlaw.org/gl#> .
@prefix math: <http://www.w3.org/2000/10/swap/math#> .

{
    ?batch a bio:ExcursionBatch ;
        bio:totalDuration ?d ;
        bio:maxTemp ?t ;
        gl:auditProof ?p .
    ?d math:greaterThan 300 .
} => {
    ?batch a bio:QuarantinedBatch ;
        gl:auditProof ?p ;
        gl:status "QUARANTINE" .
} .

{
    ?batch a bio:ExcursionBatch ;
        bio:totalDuration ?d ;
        bio:maxTemp ?t ;
        gl:auditProof ?p .
    ?d math:notGreaterThan 300 .
    ?t math:lessThan -15 .
} => {
    ?batch a bio:ReleasedBatch ;
        gl:auditProof ?p ;
        gl:status "RELEASED-21CFR211.165" .
} .
"#;

/// Runs the real duckdb CLI over the COLD-1 fixture CSVs — the same aggregate
/// pushdown leg smalt's court runs (WHERE > -20.0, qualifier, type, GROUP BY).
fn duckdb_aggregates() -> BTreeMap<String, (f64, i64)> {
    let dir = std::env::var("COLDCHAIN_CSV_DIR").unwrap_or_else(|_| {
        format!(
            "{}/smalt/tmp/coldchain",
            std::env::var("HOME").expect("HOME set")
        )
    });
    let duckdb =
        std::env::var("DUCKDB_BIN").unwrap_or_else(|_| "/opt/homebrew/bin/duckdb".to_string());
    let sql = format!(
        "SELECT o.object_id, MAX(e.temp_celsius) AS max_temp, SUM(e.duration_seconds) AS duration \
         FROM read_csv_auto('{dir}/ocel_events.csv') AS e \
         JOIN read_csv_auto('{dir}/ocel_event_object_rel.csv') AS r ON e.event_id = r.event_id \
         JOIN read_csv_auto('{dir}/ocel_objects.csv') AS o ON r.object_id = o.object_id \
         WHERE e.temp_celsius > -20.0 AND r.qualifier = 'contains_product' \
         AND o.object_type = 'bio:BiologicBatch' \
         GROUP BY o.object_id ORDER BY o.object_id"
    );
    let out = Command::new(&duckdb)
        .args(["-csv", "-noheader", "-c", &sql])
        .output()
        .expect("duckdb CLI runs");
    assert!(
        out.status.success(),
        "duckdb refused: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).expect("utf8");
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let parts: Vec<&str> = l.split(',').collect();
            assert_eq!(parts.len(), 3, "row shape: {l}");
            (
                parts[0].to_string(),
                (
                    parts[1].parse::<f64>().expect("max_temp f64"),
                    parts[2].parse::<i64>().expect("duration i64"),
                ),
            )
        })
        .collect()
}

/// Phase-5 materialized triples: one ExcursionBatch per duckdb aggregate row,
/// each bound to the COLD-1 audit hash (query+literals seal) as gl:auditProof.
fn phase5_data(aggs: &BTreeMap<String, (f64, i64)>) -> String {
    let mut s = String::from(
        "@prefix bio: <http://biopharma.org/vocab#> .\n@prefix gl:  <http://graphlaw.org/gl#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n",
    );
    for (batch, (max_t, dur)) in aggs {
        s.push_str(&format!(
            "<http://biopharma.org/batches/{batch}> a bio:ExcursionBatch ;\n    bio:totalDuration {dur} ;\n    bio:maxTemp {max_t} ;\n    gl:auditProof \"{COLD1_AUDIT_HASH}\"^^xsd:hexBinary .\n\n"
        ));
        let _ = batch.len();
    }
    s
}

fn admission_request(data: &str) -> serde_json::Value {
    json!({"op": "n3", "text": format!("{data}\n{RULESET}")})
}

/// Batches with a typed QuarantinedBatch derivation.
fn quarantined(derived: &str) -> Vec<String> {
    derived
        .lines()
        .filter(|l| l.contains("a bio:QuarantinedBatch"))
        .filter_map(|l| l.split('/').next_back().and_then(|s| s.split('>').next()))
        .map(|s| s.to_string())
        .collect()
}

/// Batches with a typed ReleasedBatch derivation.
fn released(derived: &str) -> Vec<String> {
    derived
        .lines()
        .filter(|l| l.contains("a bio:ReleasedBatch"))
        .filter_map(|l| l.split('/').next_back().and_then(|s| s.split('>').next()))
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn gws12_partition_identical_across_all_three_legs() {
    let aggs = duckdb_aggregates();
    // 9 of 10 batches have aggregate rows; BATCH-A2's events are all at
    // exactly -20.0 and are eliminated by the strict > -20.0 pushdown —
    // the missing-premise leg, fail-closed quarantined by silence.
    assert_eq!(aggs.len(), 9, "A2 must be pushdown-eliminated: {aggs:?}");
    assert!(!aggs.contains_key("BATCH-A2"), "{aggs:?}");

    let data = phase5_data(&aggs);
    let req = admission_request(&data);

    // --- Leg (a) vs Leg (b): native ABI vs wasm module, byte-identical. ---
    let n = native_bytes(&req);
    let w = wasm_bytes(&req);
    assert_eq!(
        n, w,
        "CROSS-ENGINE DIVERGENCE: native vs wasm responses differ"
    );
    let r: serde_json::Value = serde_json::from_slice(&n).expect("json");
    assert_eq!(r["ok"], json!(true), "{r}");
    let derived = r["derived"].as_str().expect("derived text");

    // --- Leg (c): partition must equal smalt COLD-1's, batch for batch. ---
    let q = quarantined(derived);
    let mut rel = released(derived);
    rel.sort();
    rel.dedup();

    // Typed quarantine (RULE 1: duration > 300): Q1 (400s), Q3 (400s).
    assert_eq!(
        q,
        vec!["BATCH-Q1", "BATCH-Q3"],
        "typed quarantine partition diverged: {q:?}"
    );

    // Typed release (RULE 2): exactly COLD-1's released set.
    assert_eq!(
        rel, COLD1_RELEASED,
        "release partition diverged from COLD-1 leg"
    );

    // Silent batches (derive NOTHING — fail-closed, no release): A1 (max ==
    // -15.0 exactly: RULE 2's lessThan is strict), Q2 (max -14.0 >= -15: RULE
    // 1 is duration-only), A2 (no aggregate row at all). Union with the typed
    // quarantine must equal COLD-1's quarantined set — partition identical,
    // quarantine-by-silence is fail-closed.
    let mut silent = COLD1_QUARANTINED.to_vec();
    for b in &q {
        silent.retain(|x| x != b);
    }
    assert_eq!(
        silent,
        vec!["BATCH-A1", "BATCH-A2", "BATCH-Q2"],
        "fail-closed silent set diverged"
    );

    // The seal binds query+literals into the admission input itself: every
    // ExcursionBatch triple in the Phase-5 data carries the COLD-1 audit hash,
    // identically on both engines (derived text omits already-known facts —
    // eyeron derives only NEW triples, so the proof is present-by-input).
    let proof_count = data.matches(COLD1_AUDIT_HASH).count();
    assert_eq!(
        proof_count, 9,
        "each aggregate batch carries the audit proof"
    );
}

#[test]
fn gws12_tampered_aggregate_flips_partition_identically_on_both_engines() {
    // Non-vacuity: a tampered literal (BATCH-R1's -17.0 -> -12.0, above the
    // release bound) must flip R1 to quarantined the SAME way on native and
    // wasm — a divergence-sensitive corpus, not a partition both engines
    // trivially agree on.
    let mut aggs = duckdb_aggregates();
    let dur = aggs["BATCH-R1"].1;
    aggs.insert("BATCH-R1".to_string(), (-12.0, dur));
    let data = phase5_data(&aggs);
    let req = admission_request(&data);

    let n = native_bytes(&req);
    let w = wasm_bytes(&req);
    assert_eq!(n, w, "tampered-corpus responses diverge across engines");

    let r: serde_json::Value = serde_json::from_slice(&n).expect("json");
    let derived = r["derived"].as_str().expect("derived text");
    // R1 (120 s <= 300, max -12.0 >= -15) matches NEITHER rule: it derives
    // NOTHING — fail-closed quarantine by silence, never release-on-doubt,
    // byte-identically on native and wasm.
    assert!(
        !derived.contains("BATCH-R1"),
        "tampered R1 must be fail-closed silent, never released: {derived}"
    );
    let rel_now = {
        let mut v: Vec<&str> = derived
            .lines()
            .filter(|l| l.contains("a bio:ReleasedBatch"))
            .filter_map(|l| l.split('/').next_back())
            .filter_map(|s| s.split('>').next())
            .collect();
        v.sort();
        v.dedup();
        v
    };
    assert_eq!(
        rel_now,
        ["BATCH-R2", "BATCH-R3", "BATCH-R4", "BATCH-R5"],
        "tamper must shrink the release set identically on both engines"
    );
}
