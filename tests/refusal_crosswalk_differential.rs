//! Refusal-crosswalk differential court (construction lane: extend the
//! L142 strata differential surface to the refusal-vocabulary crosswalk).
//!
//! The canonical three-runtime crosswalk lives in the sibling marketplace
//! pack /Users/sac/ggen-marketplace/packs/refusal-vocabulary-pack/ (READ-ONLY
//! from this repo's perspective): 51 erlmcp refusal codes (1001-1089) ->
//! 18 ash_a2a S42 refusal classes -> 6 strata-valve classes -> A2A wire
//! (-32000 column). Its closure-style totality gates live in gates/.
//!
//! Fixtures are byte-identical pinned copies of the pack (ontology + 6 gates
//! + queries/crosswalk.rq), sha256-pinned in
//! tests/fixtures/refusal_crosswalk/manifest.sha256. The manifest is the pin
//! of record; a drift meta-test compares pin vs the live sibling file and
//! BLOCKS (skips) when the sibling checkout is absent.
//!
//! Two independent engines — graphlaw's `NativeSparqlEngine` (vendored
//! purrdf-sparql-eval) and python3 rdflib — must agree with the pack's own
//! closure-gate verdicts (CLEAN, zero rows, on the canonical graph), and
//! graphlaw's independent per-valve-class evaluation must reproduce the
//! pack's L101-adjudicated valve mapping table. Sabotage twin included:
//! mutating a mapping must flip the verdict (court fails).

use graphlaw::dialect::Dialect;
use graphlaw::law::LawState;
use graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult};
use graphlaw::sparql::NativeSparqlEngine;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::process::Command;

const FIXTURE_ROOT: &str = "tests/fixtures/refusal_crosswalk";

/// Live sibling source of the pins (READ-ONLY; never edited by this repo).
const SIBLING_ROOT: &str = "/Users/sac/ggen-marketplace/packs/refusal-vocabulary-pack";

/// Pinned files: (relative path under fixture root, relative path under the
/// sibling pack).
const PINNED: &[(&str, &str)] = &[
    ("ontology.ttl", "ontology.ttl"),
    (
        "gates/010_exactly_one_class.rq",
        "gates/010_exactly_one_class.rq",
    ),
    ("gates/020_class_coverage.rq", "gates/020_class_coverage.rq"),
    ("gates/030_jsonrpc_law.rq", "gates/030_jsonrpc_law.rq"),
    ("gates/040_totality.rq", "gates/040_totality.rq"),
    ("gates/050_valve_totality.rq", "gates/050_valve_totality.rq"),
    (
        "gates/060_a2a_wire_totality.rq",
        "gates/060_a2a_wire_totality.rq",
    ),
    ("queries/crosswalk.rq", "queries/crosswalk.rq"),
];

/// The 6 strata-valve classes and their L101-adjudicated ash_a2a classes,
/// derived by reading the pack's own ontology/ValveMapping individuals and
/// its G5 closure gate (gates/050_valve_totality.rq: every declared
/// rv:ValveClass mapped by EXACTLY ONE rv:ValveMapping to a declared class).
/// This table is the expected verdict surface the court checks graphlaw's
/// independent evaluation against.
const VALVE_EXPECTED: &[(&str, &str)] = &[
    ("malformed_frame", "refused_structure"),
    ("fuel_exhausted", "refused_bounds"),
    ("parent_discontinuity", "refused_receipt"),
    ("arena_failure", "refused_consequence"),
    ("window_run_capacity", "refused_bounds"),
    ("window_tau_capacity", "refused_bounds"),
];

fn fixture(rel: &str) -> String {
    let path = format!("{FIXTURE_ROOT}/{rel}");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("pinned fixture missing: {path}: {e}"))
}

fn fixture_path(rel: &str) -> String {
    format!("{FIXTURE_ROOT}/{rel}")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Verify pinned bytes match manifest.sha256 before any evaluation runs.
fn pinned_fixtures_match_manifest() {
    let manifest = std::fs::read_to_string(format!("{FIXTURE_ROOT}/manifest.sha256"))
        .expect("manifest.sha256 present");
    let mut pinned = 0;
    for line in manifest.lines().filter(|l| !l.trim().is_empty()) {
        let (hash, path) = line
            .split_once("  ")
            .expect("sha256sum format `<hash>  <path>`");
        let rel = path.trim_start_matches('*').trim_start_matches("./");
        let bytes = std::fs::read(format!("{FIXTURE_ROOT}/{rel}")).expect("pinned file readable");
        assert_eq!(sha256_hex(&bytes), hash, "pin drift on {rel}");
        pinned += 1;
    }
    assert!(pinned >= 8, "manifest should pin >= 8 files, got {pinned}");
    for (rel, _) in PINNED {
        assert!(
            manifest.contains(&format!("./{rel}")),
            "pinned file {rel} not in manifest"
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

/// rdflib via a real python3 subprocess (Chicago: real collaborator).
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
    let verdict: Value = serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim())
        .map_err(|e| format!("verdict parse: {e}"))?;
    if verdict["ok"].as_bool() != Some(true) {
        return Err(format!("rdflib error: {}", verdict["error"]));
    }
    Ok(verdict["rows"].as_u64().expect("rows u64") as usize)
}

/// graphlaw's independent evaluation of one valve class's placement: 0 rows
/// iff the mapped ash_a2a className equals `expected`.
fn valve_mismatch_rows(ontology: &str, valve: &str, expected: &str) -> Result<usize, String> {
    let query = format!(
        r#"PREFIX rv: <urn:refusal-vocabulary:>
SELECT ?vm WHERE {{
  ?vm a rv:ValveMapping ;
      rv:valveClass ?vc ;
      rv:ashA2aClass ?cls .
  ?vc rv:valveName "{valve}" .
  ?cls rv:className ?cn .
  FILTER (?cn != "{expected}")
}}"#
    );
    engine_rows(ontology, &query)
}

/// graphlaw's independent count of distinct erlmcp codes carrying a
/// jsonRpcCode wire mapping.
fn wire_code_count(ontology: &str) -> Result<usize, String> {
    engine_rows(
        ontology,
        r#"PREFIX rv: <urn:refusal-vocabulary:>
SELECT DISTINCT ?code WHERE {
  ?m a rv:RefusalMapping ; rv:erlmcpCode ?code ; rv:jsonRpcCode ?c .
}"#,
    )
}

/// graphlaw's independent count of distinct erlmcp codes.
fn erlmcp_code_count(ontology: &str) -> Result<usize, String> {
    engine_rows(
        ontology,
        r#"PREFIX rv: <urn:refusal-vocabulary:>
SELECT DISTINCT ?code WHERE {
  ?m a rv:RefusalMapping ; rv:erlmcpCode ?code .
}"#,
    )
}

/// Pin-vs-live-sibling drift: every pinned byte sequence must still equal the
/// marketplace pack's live file. BLOCKED (skip) when the sibling checkout is
/// absent from this machine.
#[test]
fn refusal_crosswalk_pin_matches_live_sibling() {
    pinned_fixtures_match_manifest();
    if !std::path::Path::new(SIBLING_ROOT).is_dir() {
        eprintln!("BLOCKED: sibling pack absent at {SIBLING_ROOT}; drift check skipped");
        return;
    }
    for (rel, sibling_rel) in PINNED {
        let pinned_bytes = std::fs::read(fixture_path(rel)).expect("pinned file readable");
        let live = std::fs::read(format!("{SIBLING_ROOT}/{sibling_rel}"))
            .unwrap_or_else(|e| panic!("sibling file {sibling_rel} unreadable: {e}"));
        assert_eq!(
            pinned_bytes, live,
            "pin drift: tests/fixtures/refusal_crosswalk/{rel} differs from live {SIBLING_ROOT}/{sibling_rel} (re-copy and re-pin)"
        );
    }
}

/// Differential court, part 1 — the pack's own closure gates (G1..G6) over
/// the pinned canonical graph: both engines must agree with the pack's
/// verdict CLEAN (zero violation rows) on every gate.
#[test]
fn refusal_crosswalk_pack_gates_clean_and_engines_agree() {
    pinned_fixtures_match_manifest();
    let ont = fixture("ontology.ttl");
    let ont_path = fixture_path("ontology.ttl");

    let mut results: Vec<(&str, Result<usize, String>, Result<usize, String>)> = Vec::new();
    for (rel, _) in PINNED.iter().skip(1).take(6) {
        let rq = fixture(rel);
        let rq_path = fixture_path(rel);
        let e = engine_rows(&ont, &rq);
        let r = rdflib_rows(&ont_path, &rq_path);
        results.push((rel, e, r));
    }

    println!("\ngate                                      engine  rdflib  verdict");
    for (gate, e, r) in &results {
        println!(
            "{gate:<42} {:>6} {:>6}  {}",
            e.as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|_| "ERR".into()),
            r.as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|_| "ERR".into()),
            if *e == Ok(0) && *r == Ok(0) {
                "CLEAN"
            } else {
                "REFUSE"
            }
        );
    }

    for (gate, e, r) in &results {
        assert_eq!(
            *e,
            Ok(0),
            "graphlaw engine verdict on {gate}: expected CLEAN"
        );
        assert_eq!(*r, Ok(0), "rdflib verdict on {gate}: expected CLEAN");
    }
}

/// Differential court, part 2 — for each of the 6 valve classes, graphlaw's
/// independent evaluation over the pinned crosswalk graph must agree with
/// the marketplace pack's own closure-gate verdict (the L101 adjudication
/// table in VALVE_EXPECTED, cross-checked by pinned G5/050_valve_totality.rq
/// being CLEAN).
#[test]
fn valve_class_differential_agreement() {
    pinned_fixtures_match_manifest();
    let ont = fixture("ontology.ttl");

    // The pack's own verdict: G5 valve totality CLEAN (every valve class
    // mapped exactly once to a declared class).
    let g5 = engine_rows(&ont, &fixture("gates/050_valve_totality.rq"));
    assert_eq!(g5, Ok(0), "pack G5 valve-totality gate must be CLEAN");

    println!("\nvalve class            graphlaw  pack(L101)  agree");
    let mut diverged = 0;
    for (valve, expected) in VALVE_EXPECTED {
        let got = valve_mismatch_rows(&ont, valve, expected);
        let agree = got == Ok(0);
        if !agree {
            diverged += 1;
        }
        println!(
            "{valve:<22} {:>8}  {:>10}  {}",
            got.as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|e| format!("ERR:{e}")),
            expected,
            if agree { "YES" } else { "NO" }
        );
        assert_eq!(
            got,
            Ok(0),
            "valve class {valve}: graphlaw disagrees with pack verdict {expected}"
        );
    }
    assert_eq!(diverged, 0, "valve differential divergence");
    assert_eq!(
        VALVE_EXPECTED.len(),
        6,
        "the crosswalk declares exactly 6 valve classes; court must cover all 6"
    );
}

/// Totality court (fail-closed): every one of the 51 erlmcp source codes maps
/// to EXACTLY ONE ash_a2a class; all 18 classes covered; every present wire
/// code stays inside the A2A JSON-RPC law set (pinned G3). Wire-code
/// coverage is partial BY THE PACK'S OWN DESIGN (jsonRpcCode present only
/// where error.ex defines an applicable code) — the court asserts each
/// mapped code carries at most one wire code and records the coverage gap;
/// full wire presence would refuse the canonical pack itself.
#[test]
fn erlmcp_code_totality_fail_closed() {
    pinned_fixtures_match_manifest();
    let ont = fixture("ontology.ttl");

    let codes = erlmcp_code_count(&ont).expect("engine counts codes");
    assert_eq!(codes, 51, "crosswalk must place all 51 erlmcp codes");

    // Exactly one class per code: subselect COUNT + FILTER, fail-closed.
    let multi_or_missing = engine_rows(
        &ont,
        r#"PREFIX rv: <urn:refusal-vocabulary:>
SELECT ?code ?n WHERE {
  { SELECT ?code (COUNT(DISTINCT ?c) AS ?n) WHERE {
      ?m a rv:RefusalMapping ; rv:erlmcpCode ?code ; rv:ashA2aClass ?c .
    } GROUP BY ?code }
  FILTER (?n != 1)
}"#,
    );
    assert_eq!(multi_or_missing, Ok(0), "some code lacks exactly-one class");

    // All 18 declared classes targeted (G2 pinned gate, both directions).
    let g2 = engine_rows(&ont, &fixture("gates/020_class_coverage.rq"));
    assert_eq!(g2, Ok(0), "class-coverage gate must be CLEAN");

    // Wire codes: at most one per code, all inside the law set (G3 pinned).
    let g3 = engine_rows(&ont, &fixture("gates/030_jsonrpc_law.rq"));
    assert_eq!(g3, Ok(0), "jsonrpc law-set gate must be CLEAN");
    let wired = wire_code_count(&ont).expect("engine counts wired codes");
    println!(
        "\nwire-code coverage: {wired}/51 codes carry a jsonRpcCode (partial by pack design: \
         jsonRpcCode present only where the A2A error.ex law defines an applicable code; \
         -32000 column belongs to rv:A2aWireMapping, not the erlmcp column)"
    );
    assert!(
        wired <= codes,
        "wire coverage cannot exceed the code universe"
    );
}

/// Sabotage twin: mutate the pinned graph (add a second ValveMapping onto
/// vc-fuel-exhausted pointing at a different class) and the court MUST fail —
/// the pack's G5 gate fires, and graphlaw's differential evaluation diverges
/// from the expected verdict. A court that cannot fail is not a court.
#[test]
fn sabotage_twin_mutated_verdict_fails_court() {
    pinned_fixtures_match_manifest();
    let ont = fixture("ontology.ttl");

    let sabotage = "\nrv:vm-sabotage a rv:ValveMapping ;\n    rv:valveClass rv:vc-fuel-exhausted ;\n    rv:ashA2aClass rv:cls-refused-rule .\n";
    let mutated = format!("{ont}{sabotage}");
    assert_ne!(mutated, ont, "sabotage mutation applied");

    // 1. Pack's own gate verdict flips: G5 fires (double-mapped valve class).
    let g5 = engine_rows(&mutated.as_str(), &fixture("gates/050_valve_totality.rq"));
    assert!(
        matches!(&g5, Ok(n) if *n >= 1),
        "G5 must FIRE on the sabotaged graph, got {g5:?}"
    );

    // 2. graphlaw's differential evaluation flips: fuel_exhausted no longer
    //    maps solely to refused_bounds (two rows of disagreement remain even
    //    after correcting, because one mapping still contradicts).
    let still_agrees = valve_mismatch_rows(&mutated.as_str(), "fuel_exhausted", "refused_bounds");
    // With two mappings (one refused_bounds, one refused_rule), the mismatch
    // query returns >= 1 row: the court detects the tampering.
    assert!(
        matches!(&still_agrees, Ok(n) if *n >= 1),
        "differential evaluation must detect the sabotaged mapping, got {still_agrees:?}"
    );

    // 3. Exactly-one-class per CODE totality is untouched (sabotage hit the
    //    valve column): negative control proving the court's selectivity.
    let codes = erlmcp_code_count(&mutated.as_str()).expect("engine counts codes");
    assert_eq!(codes, 51, "erlmcp column untouched by valve sabotage");
}

/// Crosswalk projection (queries/crosswalk.rq) also evaluates cleanly in both
/// engines — the fourth differential surface beyond the 6 gates.
#[test]
fn crosswalk_projection_query_agrees() {
    pinned_fixtures_match_manifest();
    let ont = fixture("ontology.ttl");
    let ont_path = fixture_path("ontology.ttl");
    let rq = fixture("queries/crosswalk.rq");
    let rq_path = fixture_path("queries/crosswalk.rq");

    let e = engine_rows(&ont, &rq);
    let r = rdflib_rows(&ont_path, &rq_path);

    // Expected row shape: 51 code rows + 6 valve rows + 18 class rows +
    // 2 a2a-wire rows = 77 (derived from the pack's own closure counts).
    let expected_rows = 51 + 6 + 18 + 2;
    assert_eq!(e, Ok(expected_rows), "graphlaw projection row count");
    assert_eq!(r, Ok(expected_rows), "rdflib projection row count");
}
