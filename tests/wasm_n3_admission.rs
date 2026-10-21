//! GWS-6 court: Phase-6 N3 admission through the wasm ABI (`op: n3`).
//!
//! The case study's Phase-6 classification (release on duration<=300 AND
//! maxTemp<-15, quarantine on duration>300) runs as an N3 ruleset over the
//! Phase-5 materialized triples, inside the compiled wasm module, exactly as a
//! wasmex host would drive it.
//!
//! Ruleset adaptation (typed): the case-study prose names `gl:greaterThan`;
//! eyeron's native builtin vocabulary is the SWAP math family
//! (`http://www.w3.org/2000/10/swap/math#greaterThan`, `#notGreaterThan`,
//! `#lessThan` — see `crates/graphlaw-eyeron/src/ast.rs`). The ruleset below
//! uses the native SWAP IRIs; no other adaptation was needed.
//!
//! Fail-closed leg: a batch whose premises are incomplete (typed
//! ExcursionBatch but no bio:recordedProof on record) matches neither rule and
//! derives NOTHING — no release by default.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod common;

use serde_json::json;

const RULESET: &str = r#"
@prefix bio: <http://biopharma.org/vocab#> .
@prefix gl:  <http://graphlaw.org/gl#> .
@prefix math: <http://www.w3.org/2000/10/swap/math#> .

# RULE 1 — quarantine on cumulative excursion duration > 300 s
{
    ?batch a bio:ExcursionBatch ;
        bio:totalDuration ?d ;
        bio:maxTemp ?t ;
        bio:recordedProof ?p .
    ?d math:greaterThan 300 .
} => {
    ?batch a bio:QuarantinedBatch ;
        gl:auditProof ?p ;
        gl:status "QUARANTINE" .
} .

# RULE 2 — release on duration <= 300 AND maxTemp < -15 C
{
    ?batch a bio:ExcursionBatch ;
        bio:totalDuration ?d ;
        bio:maxTemp ?t ;
        bio:recordedProof ?p .
    ?d math:notGreaterThan 300 .
    ?t math:lessThan -15 .
} => {
    ?batch a bio:ReleasedBatch ;
        gl:auditProof ?p ;
        gl:status "RELEASED-21CFR211.165" .
} .
"#;

/// Phase-5 materialized triples: COLD-1's corpus values (BATCH-MAB-9042:
/// 140 s / -18.2 C compliant; BATCH-MAB-9043: 450 s / -12.4 C breach) plus
/// BATCH-MAB-9044, typed but missing its recorded audit proof (bio:recordedProof).
const DATA: &str = r#"
@prefix bio: <http://biopharma.org/vocab#> .
@prefix gl:  <http://graphlaw.org/gl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<http://biopharma.org/batches/BATCH-MAB-9042> a bio:ExcursionBatch ;
    bio:totalDuration 140 ;
    bio:maxTemp -18.2 ;
    bio:recordedProof "aa"^^xsd:hexBinary .

<http://biopharma.org/batches/BATCH-MAB-9043> a bio:ExcursionBatch ;
    bio:totalDuration 450 ;
    bio:maxTemp -12.4 ;
    bio:recordedProof "bb"^^xsd:hexBinary .

<http://biopharma.org/batches/BATCH-MAB-9044> a bio:ExcursionBatch ;
    bio:totalDuration 120 ;
    bio:maxTemp -19.0 .
"#;

fn admission_request() -> serde_json::Value {
    json!({"op": "n3", "text": format!("{DATA}\n{RULESET}")})
}

fn derived_text() -> String {
    let r = common::ok(&admission_request());
    r["derived"].as_str().unwrap().to_string()
}

#[test]
fn n3_admission_partitions_released_vs_quarantined() {
    let derived = derived_text();

    // 9042: 140 <= 300 and -18.2 < -15 -> released, with proof + status
    assert!(
        derived.contains("<http://biopharma.org/batches/BATCH-MAB-9042> a bio:ReleasedBatch"),
        "9042 must derive ReleasedBatch, derived was:\n{derived}"
    );
    assert!(
        derived.contains("gl:status \"RELEASED-21CFR211.165\""),
        "{derived}"
    );

    // 9043: 450 > 300 -> quarantined (fail-closed), never released
    assert!(
        derived.contains("<http://biopharma.org/batches/BATCH-MAB-9043> a bio:QuarantinedBatch"),
        "9043 must derive QuarantinedBatch, derived was:\n{derived}"
    );
    assert!(derived.contains("gl:status \"QUARANTINE\""), "{derived}");
    assert!(
        !derived.contains("9043") || !derived.contains("ReleasedBatch\n"),
        "9043 must never be released"
    );

    // Exactly one of each — the rules must not cross-fire.
    assert_eq!(derived.matches("ReleasedBatch").count(), 1, "{derived}");
    assert_eq!(derived.matches("QuarantinedBatch").count(), 1, "{derived}");
}

#[test]
fn n3_admission_is_fail_closed_for_incomplete_premises() {
    let derived = derived_text();

    // BATCH-MAB-9044 is typed but has no bio:recordedProof on record, so neither
    // rule's premise completes: it derives NOTHING. Absence of evidence is
    // quarantine, not release.
    assert!(
        !derived.contains("BATCH-MAB-9044"),
        "fail-closed: incomplete batch must derive nothing, derived was:\n{derived}"
    );
    // The document itself is not refused: the n3 op succeeds, the partition
    // for that subject is simply empty.
    assert_eq!(common::ok(&admission_request())["ok"], json!(true));
}

#[test]
fn n3_admission_matches_native_differential() {
    // wasm and native abi agree byte-identically on the same admission request.
    let req = admission_request();
    let w = common::wasm(&req);
    let n = common::native(&req);
    assert_eq!(w, n);
    assert_eq!(w["ok"], json!(true));
}
