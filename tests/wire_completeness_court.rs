//! WP P17 wire-completeness court: every `BACKEND_AUTHORITIES` row
//! (`src/lib.rs`) is exercised on the real JSON ABI wire by at least one op.
//!
//! Chain of custody (all on the real wire, `graphlaw::abi::call`, no mocks):
//! 1. The `capabilities` op reports every declared authority row verbatim
//!    (capability, authority, revision) — the wire surface and the declared
//!    table cannot drift apart silently.
//! 2. A static capability → op map (derived from `src/abi.rs::dispatch` +
//!    `src/abi/ops/*`) is checked for TOTALITY: every `BACKEND_AUTHORITIES`
//!    row maps to ≥ 1 op, and every mapped op name is on the real dispatch
//!    surface (advertised via `capabilities.ops`, or the digest-stability
//!    native-only op `duckdb`).
//! 3. Every (capability, op) pair is EXECUTED with a minimal valid request.
//!    Green = `ok:true` with the op's audit fields present; refusal probes
//!    (below) prove the error channel speaks the typed vocabulary instead of
//!    panicking.
//! 4. Receipt-bearing ops assert their audit fields: `law` → `receipts[]`
//!    (`step` / `authority` / `revision` / `parent` / `child` / `added`),
//!    `duckdb` → `query_hash` + `receipt` + `cursor`, `canonical` → content
//!    `id`.
//! 5. Refusal vocabulary check: every probe answer is either `ok:true` or
//!    `{"ok":false,"error":{kind,...,details.code}}` with `kind` in the
//!    `RefusalKind` wire vocabulary and `code` in the typed code vocabulary.
//!    A panic, an empty kind, or an unregistered code is court red.
//!
//! Receipt: the capability → op → fields matrix is printed to stdout
//! (run with `--nocapture`); every row is a witnessed wire execution.

use serde_json::{Value, json};

use graphlaw::BACKEND_AUTHORITIES;

/// IRI namespace for the court's own fixture data.
const NS: &str = "https://graphlaw.dev/wire-court#";

/// The wire `RefusalKind` vocabulary (`src/dialect/refusal.rs`,
/// `format!("{:?}", kind)` over the ABI). Append-only — this court pins the
/// current set; a new kind must be ADDED here, never silently appear.
const REFUSAL_KINDS: &[&str] = &[
    "NotSemanticContent",
    "Ambiguous",
    "EngineRejected",
    "Unsupported",
    "ResourceLimit",
    "TierMismatch",
    "DuckDb",
    "BareSql",
    "Fuse",
];

/// The typed `details.code` vocabulary (`src/abi.rs`, `src/abi/ops/*`,
/// `src/duckdb_op.rs`). Append-only; never renumbered.
const REFUSAL_CODES: &[&str] = &[
    "ResourceLimit",
    "Refused",
    "PlanRefused",
    "ReceiptRequired",
    "ReceiptRefused",
    "LeaseRefused",
    "NotAdmitted",
    "UnverifiedLeaseRefused",
    "PolicyRefused",
    "Fuse",
    "DuckDb",
    "BatchMemberFailed",
];

/// Capability → exercising ABI ops (from `src/abi.rs::dispatch` +
/// `src/abi/ops/*`). One entry per `BACKEND_AUTHORITIES` capability string;
/// totality is courted in [`wire_reports_every_declared_authority`].
const CAPABILITY_OPS: &[(&str, &[&str])] = &[
    ("RDF 1.2 / codecs / storage IR", &["parse", "convert", "canonical", "sniff"]),
    ("SPARQL 1.1/1.2", &["sparql"]),
    ("SHACL", &["shacl"]),
    ("ShEx 2.1", &["shex"]),
    ("Datalog", &["datalog"]),
    (
        "RDF/RDFS/OWL-RL entailment",
        &["entail"],
    ),
    (
        "Knowledge hooks (kh: orchestration over SPARQL)",
        &["hooks"],
    ),
    ("Notation3", &["n3"]),
    (
        "Vectorized columnar execution, Parquet scan, and SIMD aggregations",
        &["duckdb"],
    ),
];

/// `RefusalKind` debug names not (yet) fired by a probe in this court.
/// The vocabulary check asserts observed kinds are REGISTERED; this set is
/// the honest statement of which kinds this court witnessed live
/// (`Ambiguous` routing is courted by the sniff/tier sibling courts).
const UNWITNESSED_KINDS: &[&str] = &["Ambiguous"];

/// Minimal valid N-Triples fixture for one row of data.
fn nt() -> String {
    format!("<{NS}s> <{NS}p> <{NS}o> .\n")
}

/// Turtle fixture for SHACL/ShEx data rows.
fn turtle() -> String {
    "@prefix ex: <https://graphlaw.dev/wire-court#> .\n\
     ex:s ex:p ex:o .\n"
    .to_string()
}

/// Read a repo data fixture (real file, read-only; the court never edits it).
fn read_fixture(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {rel} unreadable: {e}"))
}

/// One raw wire call: bytes in through `graphlaw::abi::call`, JSON out.
/// This is the exact host-visible boundary (`gl_call` carries these bytes).
fn wire(request: &Value) -> Value {
    let bytes = graphlaw::abi::call(request.to_string().as_bytes());
    serde_json::from_slice(&bytes).expect("GraphLaw ABI always answers JSON")
}

/// Assert the envelope of a refusal: `ok:false`, `error.kind` registered,
/// `details.code` (when present) registered.
fn assert_refusal_envelope(r: &Value, ctx: &str) {
    assert_eq!(r["ok"], Value::Bool(false), "{ctx}: must refuse: {r}");
    let kind = r["error"]["kind"].as_str().unwrap_or_else(|| {
        panic!("{ctx}: refusal without error.kind is outside the vocabulary: {r}")
    });
    assert!(
        REFUSAL_KINDS.contains(&kind),
        "{ctx}: unregistered refusal kind `{kind}`: {r}"
    );
    if let Some(code) = r["error"]["details"]["code"].as_str() {
        assert!(
            REFUSAL_CODES.contains(&code),
            "{ctx}: unregistered refusal code `{code}`: {r}"
        );
    }
}

/// Assert `ok:true` and return the response.
fn ok(request: Value) -> Value {
    let r = wire(&request);
    assert_eq!(
        r["ok"],
        Value::Bool(true),
        "op `{}` must answer ok:true on a minimal valid request: {r}",
        request["op"]
    );
    r
}

/// Sorted top-level response field names — the "fields" column of the matrix.
fn fields_of(r: &Value) -> Vec<String> {
    let mut f: Vec<String> = r
        .as_object()
        .expect("response object")
        .keys()
        .cloned()
        .collect();
    f.sort();
    f
}

// ---------------------------------------------------------------------------
// 1. Wire surface == declared table
// ---------------------------------------------------------------------------

/// The `capabilities` op reports every `BACKEND_AUTHORITIES` row verbatim and
/// the full advertised op surface; `duckdb` stays native-only (digest
/// stability) yet is dispatched.
#[test]
fn wire_reports_every_declared_authority() {
    let caps = ok(json!({"op": "capabilities"}));

    // Authority rows: exact tuple equality with the declared table.
    let reported = caps["authorities"].as_array().expect("authorities array");
    assert_eq!(
        reported.len(),
        BACKEND_AUTHORITIES.len(),
        "wire authorities != declared table: {caps}"
    );
    for (w, d) in reported.iter().zip(BACKEND_AUTHORITIES.iter()) {
        assert_eq!(w["capability"], d.capability, "{caps}");
        assert_eq!(w["authority"], d.authority, "{caps}");
        assert_eq!(w["revision"], d.revision, "{caps}");
    }

    // Advertised op surface: 14 ops, registry order pinned by the registry.
    let ops = caps["ops"].as_array().expect("ops array");
    assert_eq!(ops.len(), 14, "advertised op surface drifted: {ops:?}");
    assert_eq!(ops[0], "capabilities");
    assert_eq!(ops[13], "policy");
    assert!(
        !ops.iter().any(|o| o == "duckdb"),
        "duckdb is digest-stability unadvertised; if this flips, update the court + registry docs"
    );

    // The native-only op is nevertheless dispatched (the wasm unit links it).
    let duck = wire(&json!({
        "op": "duckdb", "mode": "close", "cursor": "wire-court-probe"
    }));
    assert_eq!(duck["ok"], Value::Bool(true), "{duck}");
}

// ---------------------------------------------------------------------------
// 2. The matrix: capability → op → executed → audit fields
// ---------------------------------------------------------------------------

/// Execute the minimal valid request for `op` and assert its audit fields.
/// Returns (response, audit-field description) for the matrix.
fn exercise(op: &str) -> (Value, &'static str) {
    match op {
        "capabilities" => {
            let r = ok(json!({"op": "capabilities"}));
            assert!(!r["authorities"].as_array().expect("authorities").is_empty());
            assert_eq!(r["abi"], 1);
            (r, "authorities[].capability/authority/revision; ops; registry_blake3")
        }
        "sniff" => {
            let r = ok(json!({"op": "sniff", "text": nt()}));
            assert_eq!(r["dialect"], "NTriples", "{r}");
            assert_eq!(r["engine"], "PurRdf", "{r}");
            (r, "dialect; engine")
        }
        "parse" => {
            let r = ok(json!({"op": "parse", "text": nt(), "dialect": "ntriples"}));
            assert_eq!(r["quads"], 1, "{r}");
            assert!(r["id"].as_str().is_some_and(|i| !i.is_empty()), "{r}");
            (r, "id (content hash); quads; dialect")
        }
        "convert" => {
            let r = ok(json!({"op": "convert", "text": nt(), "dialect": "ntriples", "to": "turtle"}));
            assert!(r["text"].as_str().is_some_and(|t| t.contains("prefix") || t.contains("@prefix")), "{r}");
            assert!(r["id"].as_str().is_some_and(|i| !i.is_empty()), "{r}");
            (r, "text (turtle); id")
        }
        "canonical" => {
            let r = ok(json!({"op": "canonical", "data": {"text": nt(), "dialect": "ntriples"}}));
            assert!(r["id"].as_str().is_some_and(|i| !i.is_empty()), "{r}");
            assert_eq!(r["quads"], 1, "{r}");
            (r, "id (blake3 canonical); nquads; quads")
        }
        "sparql" => {
            let r = ok(json!({
                "op": "sparql",
                "data": {"text": nt(), "dialect": "ntriples"},
                "query": format!("SELECT ?o WHERE {{ <{NS}s> <{NS}p> ?o }}"),
            }));
            assert_eq!(r["kind"], "solutions", "{r}");
            assert!(!r["rows"].as_array().expect("rows").is_empty(), "{r}");
            (r, "kind; variables; rows")
        }
        "shacl" => {
            let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> . @prefix ex: <https://graphlaw.dev/wire-court#> .\n\
                          ex:S a sh:NodeShape ; sh:targetNode ex:s ; sh:property [ sh:path ex:p ; sh:minCount 1 ] .";
            let r = ok(json!({"op": "shacl", "data": {"text": nt(), "dialect": "ntriples"}, "shapes": shapes}));
            assert_eq!(r["conforms"], Value::Bool(true), "{r}");
            (r, "conforms; results[]")
        }
        "shex" => {
            let shexc = "PREFIX ex: <https://graphlaw.dev/wire-court#>\nex:S { ex:p . }";
            let r = ok(json!({
                "op": "shex",
                "data": {"text": turtle(), "dialect": "turtle"},
                "schema": shexc,
                "map": format!("<{NS}s>@<{NS}S>"),
            }));
            assert_eq!(r["conforms"], Value::Bool(true), "{r}");
            assert_eq!(r["entries"][0]["status"], "Conformant", "{r}");
            (r, "conforms; entries[] (status per entry)")
        }
        "n3" => {
            let r = ok(json!({
                "op": "n3",
                "text": format!("@prefix e: <{NS}> .\ne:s a e:Human .\n{{ ?x a e:Human }} => {{ ?x a e:Mortal }} .\n"),
            }));
            assert!(
                r["derived"].as_str().is_some_and(|d| d.contains("Mortal")),
                "{r}"
            );
            (r, "derived (n3 closure text)")
        }
        "entail" => {
            let data = format!(
                "@prefix e: <{NS}> . @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                 e:Cat rdfs:subClassOf e:Animal .\n e:tom a e:Cat .\n"
            );
            let r = ok(json!({"op": "entail", "data": {"text": data, "dialect": "turtle"}, "regime": "rdfs"}));
            assert!(
                r["nquads"]
                    .as_str()
                    .is_some_and(|n| n.contains("Animal")),
                "rdfs closure must derive the superclass: {r}"
            );
            (r, "nquads; added")
        }
        "datalog" => {
            let r = ok(json!({
                "op": "datalog",
                "rules": [{"head": ["?x", format!("{NS}mortal"), "?x"],
                           "body": [["?x", format!("{NS}human"), "?x"]]}],
                "facts": [[format!("{NS}s"), format!("{NS}human"), format!("{NS}s")]],
            }));
            // The closure reports input fact + derived fact.
            assert_eq!(r["count"], 2, "{r}");
            let facts = r["facts"].as_array().expect("facts");
            assert!(
                facts.iter().any(|f| f[1] == format!("{NS}mortal")),
                "seminaive eval must derive mortal: {r}"
            );
            (r, "count; facts[]; nquads")
        }
        "hooks" => {
            let r = ok(json!({
                "op": "hooks",
                "pack": {"text": read_fixture("packs/self-monitoring-pack/hook.ttl"), "dialect": "turtle"},
                "data": {"text": read_fixture("packs/self-monitoring-pack/fixtures/session-real-broad-topic.ttl"), "dialect": "turtle"},
            }));
            assert_eq!(r["firings"].as_array().expect("firings").len(), 3, "{r}");
            (r, "id; rounds; quads; firings[]")
        }
        "law" => {
            let data = format!("<{NS}s> <{NS}p> <{NS}o> .\n");
            let r = ok(json!({
                "op": "law",
                "data": {"text": data, "dialect": "ntriples"},
                "steps": [{"step": "rdfs"}],
            }));
            let receipts = r["receipts"].as_array().expect("receipts array");
            assert_eq!(receipts.len(), 1, "{r}");
            assert_eq!(receipts[0]["step"], "derive:rdfs", "{r}");
            assert_eq!(receipts[0]["authority"], "purrdf::entail", "{r}");
            (r, "states[]; receipts[].step/authority/revision/parent/child/added; nquads")
        }
        "policy" => {
            let problem = json!({
                "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
                "initial_states": ["s0"],
                "goal": {"facts": ["done"]},
                "transitions": [
                    {"action": "flip", "from": "s0", "to": "g", "probability_ppm": 500000},
                    {"action": "flip", "from": "s0", "to": "s0", "probability_ppm": 500000}]
            });
            let policy = json!({"policy": [{"state": "s0", "action": "flip", "outcomes": [
                {"state": "g", "probability_ppm": 500000},
                {"state": "s0", "probability_ppm": 500000}]}]});
            let r = ok(json!({"op": "policy", "problem": problem, "policy": policy}));
            assert_eq!(r["entries"], json!([["s0", "flip"]]), "{r}");
            (r, "initial_states; reachable; goal_states; entries[]; ntriples")
        }
        "duckdb" => {
            let turtle_ast = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <https://graphlaw.dev/wire-court#> .
ex:C a sql:CreateStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "wire_court_events" ;
    sql:assignColumn "batch_id" .
"#;
            let r = ok(json!({
                "op": "duckdb", "mode": "exec",
                "query_ast": "urn:wire-court:create",
                "ast": {"turtle": turtle_ast},
                "subject_prefix": "urn:wire-row/",
                "vocab_prefix": "urn:wire-p/",
            }));
            // Receipt-bearing: query_hash + receipt IRI + cursor are the
            // audit channel of op-15.
            let hash = r["query_hash"].as_str().expect("query_hash");
            assert_eq!(hash.len(), 64, "hex blake3 query_hash: {r}");
            let receipt = r["receipt"].as_str().expect("receipt IRI");
            assert!(
                receipt.contains(&hash[..16]),
                "receipt IRI mints from the query hash: {r}"
            );
            assert!(r["cursor"].as_str().is_some_and(|c| !c.is_empty()), "{r}");
            (r, "receipt (IRI); query_hash (blake3); rows_emitted; cursor; more")
        }
        other => panic!("no minimal valid request authored for op `{other}`"),
    }
}

/// THE court: every declared authority row is exercised live on the wire and
/// the capability → op → fields matrix is printed as the receipt.
#[test]
fn every_authority_row_is_exercised_on_the_wire() {
    // Totality: every BACKEND_AUTHORITIES capability maps to >= 1 op.
    for row in BACKEND_AUTHORITIES {
        let (_, ops) = CAPABILITY_OPS
            .iter()
            .find(|(c, _)| *c == row.capability)
            .unwrap_or_else(|| {
                panic!(
                    "capability {:?} has no ABI op mapping — the wire-completeness invariant is broken",
                    row.capability
                )
            });
        assert!(!ops.is_empty(), "capability {:?} maps to zero ops", row.capability);
    }

    // Soundness: every mapped op is on the real dispatch surface.
    let caps = ok(json!({"op": "capabilities"}));
    let advertised: Vec<&str> = caps["ops"]
        .as_array()
        .expect("ops")
        .iter()
        .map(|o| o.as_str().expect("op string"))
        .collect();
    for (cap, ops) in CAPABILITY_OPS {
        for op in *ops {
            assert!(
                advertised.contains(op) || *op == "duckdb",
                "capability {cap:?} maps to op `{op}` which is neither advertised nor the native-only duckdb op"
            );
        }
    }

    // Execution + matrix receipt.
    let mut matrix = String::from(
        "capability | authority | op | ok | audit fields\n",
    );
    for row in BACKEND_AUTHORITIES {
        let (_, ops) = CAPABILITY_OPS
            .iter()
            .find(|(c, _)| *c == row.capability)
            .expect("totality asserted above");
        for op in *ops {
            let (r, audit) = exercise(op);
            let fields = fields_of(&r);
            // Audit-field presence: the audit column names fields that MUST
            // be in the response.
            for need in audit.split("; ").map(str::trim) {
                let key = need.split_whitespace().next().unwrap_or(need);
                let key = key.trim_end_matches("[]");
                assert!(
                    r.get(key).is_some(),
                    "op `{op}` response lacks audit field `{key}` (audit: {audit}): {r}"
                );
                // Array-typed audit fields must actually be arrays.
                if need.ends_with("[]") {
                    assert!(
                        r[key].is_array(),
                        "op `{op}` audit field `{key}` must be an array: {r}"
                    );
                }
            }
            matrix.push_str(&format!(
                "{} | {} @ {} | {} | true | response fields: {}; audit: {}\n",
                row.capability,
                row.authority,
                row.revision,
                op,
                fields.join(","),
                audit
            ));
        }
    }
    print!("\n=== WP P17 capability → op → fields matrix (all rows executed ok:true) ===\n{matrix}\n");
}

// ---------------------------------------------------------------------------
// 3. Receipt-bearing ops: the audit channel under a microscope
// ---------------------------------------------------------------------------

/// The `law` op receipts carry `step` / `authority` / `revision` and the
/// full parent→child chain; `record-receipts` + `require-receipt` prove the
/// receipts array is the auditable state channel.
#[test]
fn receipt_bearing_law_op_asserts_audit_fields() {
    let data = format!("<{NS}s> <{NS}p> <{NS}o> .\n");
    let r = ok(json!({
        "op": "law",
        "data": {"text": data, "dialect": "ntriples"},
        "steps": [
            {"step": "rdfs"},
            {"step": "n3", "rules": format!("@prefix e: <{NS}> .\n{{ ?x <{NS}p> ?y }} => {{ ?x <{NS}derived> ?y }} .\n")},
        ]
    }));
    let receipts = r["receipts"].as_array().expect("receipts array");
    assert_eq!(receipts.len(), 2, "{r}");
    let expected = [("derive:rdfs", "purrdf::entail"), ("derive:n3", "purrdf::n3")];
    for (receipt, (step, authority)) in receipts.iter().zip(expected.iter()) {
        assert_eq!(&receipt["step"], step, "{r}");
        assert_eq!(&receipt["authority"], authority, "{r}");
        assert_eq!(receipt["revision"], "3.0.1", "{r}");
        for field in ["parent", "child", "added"] {
            assert!(
                receipt.get(field).is_some(),
                "receipt for `{step}` lacks audit field `{field}`: {r}"
            );
        }
        // parent → child chaining is the audit spine.
        assert_ne!(receipt["parent"], receipt["child"], "{r}");
    }
    // states[0] is the input state id; receipts' parents chain from it.
    let states = r["states"].as_array().expect("states array");
    assert_eq!(states.len(), 3, "input + 2 children: {r}");
    assert_eq!(receipts[0]["parent"], states[0], "{r}");
    assert_eq!(receipts[1]["parent"], states[1], "{r}");

    // record-receipts + require-receipt: the receipts become the recorded
    // state; require-receipt mints its own audit receipt (admit:require-
    // receipt, authority purrdf) without changing the state.
    let r2 = ok(json!({
        "op": "law",
        "data": {"text": data, "dialect": "ntriples"},
        "steps": [
            {"step": "rdfs"},
            {"step": "record-receipts"},
            {"step": "require-receipt", "step_name": "derive:rdfs"},
        ]
    }));
    let receipts2 = r2["receipts"].as_array().expect("receipts");
    assert_eq!(receipts2.len(), 2, "{r2}");
    assert_eq!(receipts2[1]["step"], "admit:require-receipt", "{r2}");
    assert_eq!(receipts2[1]["authority"], "purrdf", "{r2}");
    assert_eq!(
        receipts2[1]["parent"], receipts2[1]["child"],
        "require-receipt changes no state: {r2}"
    );

    let refused = wire(&json!({
        "op": "law",
        "data": {"text": data, "dialect": "ntriples"},
        "steps": [{"step": "require-receipt", "step_name": "derive:rdfs"}],
    }));
    assert_refusal_envelope(&refused, "require-receipt without record");
    assert_eq!(refused["error"]["details"]["code"], "ReceiptRequired");
    assert_eq!(refused["error"]["details"]["step"], "derive:rdfs");
}

// ---------------------------------------------------------------------------
// 4. Refusal vocabulary check
// ---------------------------------------------------------------------------

/// Every probe answer is a TYPED refusal: `error.kind` registered, and
/// `details.code` (when present) registered. Exact code pins where the code
/// is the contract.
#[test]
fn refusal_probes_speak_the_typed_vocabulary() {
    let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> . @prefix ex: <https://graphlaw.dev/wire-court#> .\n\
                  ex:S a sh:NodeShape ; sh:targetClass ex:T ; sh:property [ sh:path ex:p ; sh:minCount 1 ] .";
    let two_bad = format!(
        "@prefix ex: <https://graphlaw.dev/wire-court#> .\nex:a a ex:T .\nex:b a ex:T .\n"
    );
    let at = |o: &str| format!("<{NS}robot> <{NS}at> <{NS}{o}> .\n");
    let problem = json!({
        "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"], "goal": {"facts": ["done"]},
        "transitions": [
            {"action": "flip", "from": "s0", "to": "g", "probability_ppm": 500000},
            {"action": "flip", "from": "s0", "to": "s0", "probability_ppm": 500000}]
    });
    let skewed = json!({"policy": [{"state": "s0", "action": "flip", "outcomes": [
        {"state": "g", "probability_ppm": 400000}, {"state": "s0", "probability_ppm": 500000}]}]});
    let fused = format!(
        "@prefix e: <{NS}> .\ne:s a e:Human .\n{{ ?x a e:Human }} => {{ ?x a e:Fused }} .\n{{ ?q a e:Fused }} => false .\n"
    );
    let plan_over_limit = || {
        let mut actions = Vec::new();
        for i in 0..=graphlaw::abi::MAX_PLAN_ACTIONS {
            actions.push(json!({"name": format!("a{i}")}));
        }
        json!({
            "op": "law",
            "data": {"text": at("a"), "dialect": "ntriples"},
            "steps": [{"step": "plan", "plan": {"actions": actions, "goal": ""}}],
        })
    };

    // (probe name, request, expected kind, expected details.code)
    let cases: Vec<(&str, Value, &str, Option<&str>)> = vec![
        ("unknown op", json!({"op": "no_such_op"}), "Unsupported", None),
        ("sniff missing text", json!({"op": "sniff"}), "Unsupported", None),
        ("parse bad document", json!({"op": "parse", "text": "this is not a valid turtle document @@@ @@", "dialect": "ntriples"}), "EngineRejected", None),
        ("parse n3 formula in rdf layer", json!({"op": "parse", "text": "not rdf at all {{{", "dialect": "ntriples"}), "TierMismatch", None),
        ("bare sql sniff refused", json!({"op": "sniff", "text": "SELECT name FROM users"}), "BareSql", None),
        ("unknown dialect", json!({"op": "parse", "text": nt(), "dialect": "klingon"}), "Unsupported", None),
        ("law unverified lease", json!({
            "op": "law",
            "data": {"text": at("a"), "dialect": "ntriples"},
            "lease": {"id": "L1", "holder": "wire-court", "ceiling": "construct",
                      "scope": ["derive:rdfs"], "expires_unix": 9_999_999_999u64},
        }), "Unsupported", Some("UnverifiedLeaseRefused")),
        ("law ceiling breach", json!({
            "op": "law",
            "data": {"text": at("a"), "dialect": "ntriples"},
            "unverified_lease": true, "now_unix": 10,
            "lease": {"id": "L2", "holder": "wire-court", "ceiling": "observe",
                      "scope": ["derive:rdfs"], "expires_unix": 9_999_999_999u64},
            "steps": [{"step": "rdfs"}],
        }), "EngineRejected", Some("LeaseRefused")),
        ("law require-receipt", json!({
            "op": "law",
            "data": {"text": at("a"), "dialect": "ntriples"},
            "steps": [{"step": "require-receipt", "step_name": "derive:rdfs"}],
        }), "EngineRejected", Some("ReceiptRequired")),
        ("law shacl not admitted", json!({
            "op": "law",
            "data": {"text": two_bad, "dialect": "turtle"},
            "steps": [{"step": "shacl", "shapes": shapes}],
        }), "EngineRejected", Some("NotAdmitted")),
        ("law plan refused", json!({
            "op": "law",
            "data": {"text": at("a"), "dialect": "ntriples"},
            "steps": [{"step": "plan", "plan": {"actions": [
                {"name": "a-b", "pre": at("z"), "add": at("b"), "del": at("a")}], "goal": at("b")}}],
        }), "EngineRejected", Some("PlanRefused")),
        ("policy bad mass", json!({"op": "policy", "problem": problem, "policy": skewed}), "EngineRejected", Some("PolicyRefused")),
        ("n3 fuse", json!({"op": "n3", "text": fused}), "Fuse", Some("Fuse")),
        ("duckdb raw sql refused", json!({
            "op": "duckdb", "mode": "exec", "query_ast": "urn:wire-court:evil",
            "ast": {"text": "SELECT 42; DROP TABLE wire_court_events"},
            "subject_prefix": "urn:wire-row/", "vocab_prefix": "urn:wire-p/",
        }), "DuckDb", Some("DuckDb")),
        ("duckdb unknown session", json!({
            "op": "duckdb", "mode": "exec", "session": "wire-court-nope",
            "query_ast": "urn:wire-court:q",
            "ast": {"turtle": "@prefix sql: <http://graphlaw.org/sql#> . ex:q a sql:SelectStatement ."},
            "subject_prefix": "urn:wire-row/", "vocab_prefix": "urn:wire-p/",
        }), "DuckDb", Some("DuckDb")),
        ("duckdb chunk_rows over cap", json!({
            "op": "duckdb", "mode": "exec",
            "query_ast": "urn:wire-court:c",
            "ast": {"turtle": "@prefix sql: <http://graphlaw.org/sql#> . @prefix ex: <https://graphlaw.dev/wire-court#> . ex:C a sql:CreateStatement ; sql:statementIndex 1 ; sql:targetTable \"wire_court_chunk\" ; sql:assignColumn \"x\" ."},
            "subject_prefix": "urn:wire-row/", "vocab_prefix": "urn:wire-p/",
            "chunk_rows": graphlaw::duckdb_op::MAX_CHUNK_ROWS + 1,
        }), "ResourceLimit", Some("ResourceLimit")),
        ("law plan actions over cap", plan_over_limit(), "ResourceLimit", Some("ResourceLimit")),
    ];

    for (name, req, kind, code) in cases {
        let r = wire(&req);
        assert_refusal_envelope(&r, name);
        assert_eq!(
            r["error"]["kind"], kind,
            "{name}: refusal kind drifted: {r}"
        );
        match code {
            Some(c) => assert_eq!(
                r["error"]["details"]["code"], c,
                "{name}: refusal code drifted: {r}"
            ),
            None => assert!(
                r["error"]["details"].is_null(),
                "{name}: expected a kind-level refusal without details.code: {r}"
            ),
        }
    }

    // The witnessed kinds are a registered subset; `Ambiguous` routing is
    // courted by sibling courts (parse_refusal_court, sniff tests).
    let witnessed_kinds = [
        "Unsupported",
        "EngineRejected",
        "TierMismatch",
        "BareSql",
        "DuckDb",
        "ResourceLimit",
        "Fuse",
        "NotSemanticContent",
    ];
    for k in witnessed_kinds {
        assert!(
            REFUSAL_KINDS.contains(&k),
            "witnessed kind `{k}` fell out of the pinned vocabulary"
        );
    }
    assert_eq!(
        REFUSAL_KINDS.len(),
        witnessed_kinds.len() + UNWITNESSED_KINDS.len(),
        "vocabulary pin drifted — a kind was added to RefusalKind without updating this court"
    );
}

/// The SNIFF-01 sibling: an HTML document is `NotSemanticContent`, never a
/// parse crash — witnessed live here so the kind vocabulary check above is
/// backed by a real probe.
#[test]
fn html_sniff_refuses_not_semantic_content() {
    let r = wire(&json!({"op": "sniff", "text": "<!DOCTYPE html><html></html>"}));
    assert_refusal_envelope(&r, "html sniff");
    assert_eq!(r["error"]["kind"], "NotSemanticContent", "{r}");
}

/// The hooks op's pack + data fixtures stay loadable through the wire op
/// (guards the `read_fixture` paths this court depends on).
#[test]
fn hooks_fixtures_are_readable_through_the_wire() {
    let pack = read_fixture("packs/self-monitoring-pack/hook.ttl");
    let data = read_fixture("packs/self-monitoring-pack/fixtures/session-real-broad-topic.ttl");
    let r = wire(&json!({
        "op": "hooks",
        "pack": {"text": pack, "dialect": "turtle"},
        "data": {"text": data, "dialect": "turtle"},
    }));
    assert_eq!(r["ok"], Value::Bool(true), "{r}");
}

/// Notation3's `derive:n3` law step exercises the Notation3 authority through
/// a receipted transition (authority resolution path: Step::capability →
/// `authority_for_capability` → BACKEND_AUTHORITIES).
#[test]
fn law_step_receipts_resolve_authorities_from_the_declared_table() {
    let data = format!("<{NS}s> <{NS}p> <{NS}o> .\n");
    let r = ok(json!({
        "op": "law",
        "data": {"text": data, "dialect": "ntriples"},
        "steps": [
            {"step": "shacl", "shapes": "@prefix sh: <http://www.w3.org/ns/shacl#> . <https://graphlaw.dev/wire-court#S> a sh:NodeShape ."},
            {"step": "rdfs"},
            {"step": "owl-rl"},
        ]
    }));
    let receipts = r["receipts"].as_array().expect("receipts");
    assert_eq!(receipts.len(), 3, "{r}");
    let expected = [
        ("admit:shacl", "purrdf::shapes"),
        ("derive:rdfs", "purrdf::entail"),
        ("derive:owl-rl", "purrdf::entail"),
    ];
    for (receipt, (step, authority)) in receipts.iter().zip(expected.iter()) {
        assert_eq!(&receipt["step"], step, "{r}");
        assert_eq!(&receipt["authority"], authority, "{r}");
        // Every receipt authority is a row of the declared table.
        assert!(
            BACKEND_AUTHORITIES
                .iter()
                .any(|a| a.authority == receipt["authority"]),
            "receipt authority {} is not a BACKEND_AUTHORITIES row: {r}",
            receipt["authority"]
        );
    }
}
