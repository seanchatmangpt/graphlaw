//! Differential gate-evaluation court (lane L142): the same pinned strata-pack
//! SPARQL gate run through two independent engines — graphlaw's
//! `NativeSparqlEngine` (vendored purrdf-sparql-eval) and python3 rdflib —
//! must agree on row-ness.
//!
//! Motivated by the UNION-binding-leak divergence: engine differences are a
//! proven defect class, so silence-agreement over the clean graph is not
//! enough; a negative court also proves agreement on FIRING (both engines
//! produce a violation row over a mutated ontology).
//!
//! Fixtures are byte-identical pinned copies of
//! /Users/sac/ggen-marketplace/packs/strata-*-pack/{ontology.ttl,gates/*.rq},
//! blake3-pinned in `manifest.blake3` (regenerate pins by re-copying from the
//! marketplace pack source; the manifest is the pin of record).
//!
//! rdflib is exercised through a real python3 subprocess (Chicago: real
//! collaborator, no mocks); the repo already runs subprocesses in tests
//! (readme_first_ten_minutes.rs, corpus_conformance.rs). If a sandbox blocks
//! python3, regenerate pre-recorded row-JSON via the command documented in
//! tests/fixtures/strata_gate_differential/README.md and swap `rdflib_rows`
//! for a fixture reader.

use graphlaw::dialect::Dialect;
use graphlaw::law::LawState;
use graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult};
use graphlaw::sparql::NativeSparqlEngine;
use serde_json::Value;
use std::process::Command;

const FIXTURE_ROOT: &str = "tests/fixtures/strata_gate_differential";

/// (pack, gate file) pairs pinned for the court.
const GATES: &[(&str, &str)] = &[
    ("hash", "010_digest_length.rq"),         // FILTER-heavy violation rows
    ("hash", "020_algorithm_closure.rq"),     // UNION + subselect COUNT
    ("valve", "010_refusal_atom_closure.rq"), // UNION + VALUES + FILTER NOT EXISTS
    ("valve", "020_wire_class_present.rq"),   // OPTIONAL + BOUND
    ("ledger", "010_ledger_layout.rq"),       // V7-style datatype arms
    ("tx", "020_tx_closure.rq"),              // closure / NOT EXISTS arms
    ("governance", "020_commitment_closure.rq"), // V7-family fail-closed presence
];

fn fixture(pack: &str, rel: &str) -> String {
    let path = format!("{FIXTURE_ROOT}/{pack}/{rel}");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("pinned fixture missing: {path}: {e}"))
}

fn fixture_path(pack: &str, rel: &str) -> String {
    format!("{FIXTURE_ROOT}/{pack}/{rel}")
}

fn blake3_hex(bytes: &[u8]) -> String {
    let mut h = blake3::Hasher::new();
    h.update(bytes);
    hex(h.finalize().as_bytes())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Verify pinned bytes match manifest.blake3 before any evaluation runs.
#[test]
fn pinned_fixtures_match_manifest() {
    let manifest = std::fs::read_to_string(format!("{FIXTURE_ROOT}/manifest.blake3"))
        .expect("manifest.blake3 present");
    let mut pinned = 0;
    for line in manifest.lines().filter(|l| !l.trim().is_empty()) {
        let (hash, path) = line
            .split_once("  ")
            .expect("b3sum format `<hash>  <path>`");
        let rel = path.trim_start_matches('*').trim_start_matches("./");
        let bytes = std::fs::read(format!("{FIXTURE_ROOT}/{rel}")).expect("pinned file readable");
        assert_eq!(blake3_hex(&bytes), hash, "pin drift on {rel}");
        pinned += 1;
    }
    assert!(pinned >= 12, "manifest should pin >= 12 files, got {pinned}");
    for (pack, gate) in GATES {
        let marker = format!("./{pack}/gates/{gate}");
        assert!(
            manifest.contains(&marker),
            "gate {pack}/{gate} not pinned in manifest"
        );
    }
}

fn engine_rows(ontology_ttl: &str, query: &str) -> Result<usize, String> {
    let state = LawState::parse(ontology_ttl.as_bytes(), Dialect::Turtle, None)
        .map_err(|e| format!("engine parse: {e}"))?;
    match NativeSparqlEngine::new()
        .query(
            state.dataset(),
            SparqlRequest {
                query,
                base_iri: None,
                substitutions: &[],
            },
        )
        .map_err(|e| format!("engine query: {e}"))?
    {
        SparqlResult::Solutions { rows, .. } => Ok(rows.len()),
        other => Err(format!("expected solutions, got {other:?}")),
    }
}

/// rdflib via a real python3 subprocess: loads the ontology, runs the gate
/// SELECT, prints a one-line JSON verdict on stdout.
fn rdflib_rows(ontology_path: &str, query_path: &str) -> Result<usize, String> {
    let script = r#"
import json, sys
from rdflib import Graph
ont_path, query_path = sys.argv[1], sys.argv[2]
try:
    g = Graph()
    g.parse(ont_path, "turtle")
    res = g.query(open(query_path).read())
    print(json.dumps({"ok": True, "rows": len(list(res))}))
except Exception as err:
    print(json.dumps({"ok": False, "error": str(err)}))
"#;
    let out = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(ontology_path)
        .arg(query_path)
        .output()
        .map_err(|e| format!("python3 spawn: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "python3 exit {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let verdict: Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).map_err(|e| {
            format!(
                "verdict parse: {e}; stdout={:?}",
                String::from_utf8_lossy(&out.stdout)
            )
        })?;
    if verdict["ok"].as_bool() != Some(true) {
        return Err(format!("rdflib error: {}", verdict["error"]));
    }
    Ok(verdict["rows"].as_u64().expect("rows u64") as usize)
}

#[test]
fn differential_gate_agreement() {
    pinned_fixtures_match_manifest();

    type GateVerdict = (&'static str, &'static str, Result<usize, String>, Result<usize, String>);
    let mut results: Vec<GateVerdict> = Vec::new();
    for (pack, gate) in GATES {
        let ont = fixture(pack, "ontology.ttl");
        let rq = fixture(pack, &format!("gates/{gate}"));
        let ont_path = fixture_path(pack, "ontology.ttl");
        let rq_path = fixture_path(pack, &format!("gates/{gate}"));

        let e = engine_rows(&ont, &rq);
        let r = rdflib_rows(&ont_path, &rq_path);
        results.push((pack, gate, e, r));
    }

    println!("\ngate                                   engine  rdflib  agree");
    let mut diverged = 0;
    for (pack, gate, e, r) in &results {
        let en = e.as_ref().ok();
        let rn = r.as_ref().ok();
        let agree = en == rn;
        if !agree {
            diverged += 1;
        }
        println!(
            "{pack}/{gate:<32} {:>6} {:>6}  {}",
            en.map(|n| n.to_string()).unwrap_or_else(|| "ERR".into()),
            rn.map(|n| n.to_string()).unwrap_or_else(|| "ERR".into()),
            if agree { "YES" } else { "NO" }
        );
    }

    // Clean pinned ontologies are violation-free: both engines must agree on
    // zero rows (silence agreement), and neither engine may error.
    assert_eq!(diverged, 0, "engine/rdflib row-count divergence detected");
    for (pack, gate, e, r) in &results {
        assert_eq!(*e, Ok(0), "engine rows on clean {pack}/{gate}");
        assert_eq!(*r, Ok(0), "rdflib rows on clean {pack}/{gate}");
    }
}

/// Negative court: agreement on FIRING, not just on silence. Mutate a scratch
/// copy of the ledger ontology (drop the lg:maxAccounts value triple) and both
/// engines must produce the same >= 1 violation rows for the V7 ledger layout
/// gate.
#[test]
fn negative_firing_agreement_on_mutated_ontology() {
    let ont = fixture("ledger", "ontology.ttl");
    let rq = fixture("ledger", "gates/010_ledger_layout.rq");
    let marker = "lg:maxAccounts \"65536\"^^xsd:integer";
    assert!(ont.contains(marker), "pinned ontology lost the mutation target");
    let mutated = ont.replacen(marker, "", 1);

    // Scratch copy lives only for the duration of the court and is deleted on
    // exit (fan-out cleanup law: no orphaned scratch).
    let dir = std::env::temp_dir().join("graphlaw-l142-negative");
    std::fs::create_dir_all(&dir).unwrap();
    let ont_path = dir.join("ontology_mutated.ttl");
    let rq_path = dir.join("010_ledger_layout.rq");
    std::fs::write(&ont_path, &mutated).unwrap();
    std::fs::write(&rq_path, &rq).unwrap();

    let e = engine_rows(&mutated, &rq);
    let r = rdflib_rows(ont_path.to_str().unwrap(), rq_path.to_str().unwrap());
    let _ = std::fs::remove_dir_all(&dir);

    let en = e.expect("engine evaluates mutated ontology");
    let rn = r.expect("rdflib evaluates mutated ontology");
    assert!(en >= 1, "engine must FIRE on the mutated ontology, got {en}");
    assert!(rn >= 1, "rdflib must FIRE on the mutated ontology, got {rn}");
    assert_eq!(en, rn, "firing agreement: engine={en} rdflib={rn}");
}
