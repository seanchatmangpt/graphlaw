//! W08 differential court: N >= 30 canonical, keyless/clockless inputs (every
//! law request carries an explicit `now_unix`) run natively via
//! `graphlaw::abi::call` and through the compiled wasm module via the shared
//! Host harness in `tests/common`. Raw-byte `assert_eq` per case — never a
//! parsed-JSON compare.
//!
//! Reuse disclosure: the Host/build helpers are imported from
//! `tests/common/mod.rs` (extracted from `tests/wasm_abi.rs`), not duplicated.
//! The only locally duplicated logic is `wasm_artifact()` below, which is the
//! same cargo invocation as `common::wasm_path` but *fallible* so the suite can
//! skip loud instead of panicking when the wasm32-wasip1 target is absent.
//!
//! Skip policy: loud and counted. If the artifact is unavailable (env-set but
//! missing, or unset and the one-time build fails), every case is named and
//! counted in a `[SKIP]` line and the court runs zero comparisons — it never
//! reports a silent pass.
#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde_json::{Value, json};

mod common;

// ---------------------------------------------------------------------------
// wasm artifact availability (fallible variant of common::wasm_path)
// ---------------------------------------------------------------------------

fn wasm_artifact() -> Result<PathBuf, String> {
    static OK: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    OK.get_or_init(|| {
        if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
            let p = PathBuf::from(p);
            return if p.exists() {
                Ok(p)
            } else {
                Err(format!(
                    "GRAPHLAW_WASM set but artifact missing: {}",
                    p.display()
                ))
            };
        }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let target = root.join("target/wasm-abi");
        let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
            .current_dir(&root)
            .args([
                "build",
                "-p",
                "graphlaw-wasm",
                "--target",
                "wasm32-wasip1",
                "--profile",
                "wasm",
                "--target-dir",
            ])
            .arg(&target)
            .status()
            .map_err(|e| format!("cargo failed to launch: {e}"))?;
        if !status.success() {
            return Err("wasm build failed (is the wasm32-wasip1 target installed?)".into());
        }
        let p = target.join("wasm32-wasip1/wasm/graphlaw_wasm.wasm");
        if p.exists() {
            Ok(p)
        } else {
            Err(format!("wasm artifact not found at {}", p.display()))
        }
    })
    .clone()
}

/// Loud SKIP, unless a require-env is set, in which case the missing artifact
/// is a hard failure (a skip must never be the CI outcome).
fn skip(reason: &str, cases: &[&str]) {
    if std::env::var("GRAPHLAW_REQUIRE_WASM").as_deref() == Ok("1")
        || std::env::var("GRAPHLAW_REQUIRE_ENGINE").as_deref() == Ok("1")
    {
        panic!(
            "wasm_native_differential: {reason} — GRAPHLAW_REQUIRE_WASM/GRAPHLAW_REQUIRE_ENGINE=1 forbids skipping ({} cases: {})",
            cases.len(),
            cases.join(", ")
        );
    }
    eprintln!(
        "[SKIP] wasm_native_differential: {reason} — {} cases skipped: {}",
        cases.len(),
        cases.join(", ")
    );
}

// ---------------------------------------------------------------------------
// raw-byte differential with a typed Divergence panic
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Divergence {
    op: String,
    index: usize,
    native_len: usize,
    wasm_len: usize,
    /// byte offset of the first differing byte, or None if one is a prefix
    first_diff: Option<usize>,
}

impl Divergence {
    fn panic_for(&self, name: &str, req: &Value, native: &[u8], wasm: &[u8]) -> ! {
        panic!(
            "DIVERGENCE {name}\nop: {}\nindex: {}\nnative_len: {}\nwasm_len: {}\nfirst_diff: {:?}\nrequest: {req}\nnative: {}\nwasm:   {}",
            self.op,
            self.index,
            self.native_len,
            self.wasm_len,
            self.first_diff,
            String::from_utf8_lossy(native),
            String::from_utf8_lossy(wasm),
        )
    }
}

fn compare(name: &str, index: usize, req: &Value) {
    let native = common::native_bytes(req);
    let wasm = common::wasm_bytes(req);
    if name == "capabilities" {
        compare_capabilities(&native, &wasm);
        return;
    }
    if native == wasm {
        return;
    }
    let first_diff = native.iter().zip(wasm.iter()).position(|(a, b)| a != b);
    Divergence {
        op: req["op"].as_str().unwrap_or("?").to_string(),
        index,
        native_len: native.len(),
        wasm_len: wasm.len(),
        first_diff,
    }
    .panic_for(name, req, &native, &wasm);
}

/// Since the EH closure the wasm module links frozen-duckdb as part of the
/// ONE unit (repo law: DuckDB is not a feature), so host and wasm advertise
/// the SAME authority list — compared for full equality now. Every field,
/// including both digests, stays byte-for-field identical.
fn compare_capabilities(native: &[u8], wasm: &[u8]) {
    let n: Value = serde_json::from_slice(native).expect("native capabilities JSON");
    let w: Value = serde_json::from_slice(wasm).expect("wasm capabilities JSON");
    assert_eq!(
        n["authorities"], w["authorities"],
        "native and wasm authority lists diverged: {{native: {n}, wasm: {w}}}"
    );
    assert!(
        n["authorities"]
            .as_array()
            .expect("native authorities")
            .iter()
            .any(|a| a["authority"] == "frozen-duckdb"),
        "capabilities dropped frozen-duckdb — the ONE-unit law broke: {n}"
    );
}

/// Differential ok: byte-identical and admitted on both engines.
fn diff_ok(name: &str, index: usize, req: &Value) -> Value {
    compare(name, index, req);
    let v: Value = serde_json::from_slice(&common::native_bytes(req)).expect("native JSON");
    assert_eq!(v["ok"], json!(true), "{name}: request {req} failed: {v}");
    v
}

/// Differential refusal: byte-identical and refused on both engines.
fn diff_refused(name: &str, index: usize, req: &Value) -> Value {
    compare(name, index, req);
    let v: Value = serde_json::from_slice(&common::native_bytes(req)).expect("native JSON");
    assert_eq!(
        v["ok"],
        json!(false),
        "{name}: request {req} was admitted: {v}"
    );
    v
}

// ---------------------------------------------------------------------------
// canonical, keyless/clockless inputs
// ---------------------------------------------------------------------------

const DOC_TTL: &str = r#"@prefix ex: <http://example.org/> .
ex:alice ex:knows ex:bob .
ex:bob ex:knows ex:carol .
"#;

const DOC_TTL_REORDERED: &str = r#"@prefix p: <http://example.org/> .
p:bob p:knows p:carol .
p:alice p:knows p:bob .
"#;

const BNQ_AB: &str = "_:x <http://example.org/knows> _:y .\n_:y <http://example.org/knows> _:x .\n";
const BNQ_PQ: &str = "_:p <http://example.org/knows> _:q .\n_:q <http://example.org/knows> _:p .\n";

fn at(o: &str) -> String {
    format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n")
}

fn law(data: Value, steps: Value) -> Value {
    json!({"op": "law", "data": data, "now_unix": 1_700_000_000u64, "steps": steps})
}

fn ttl(text: &str) -> Value {
    json!({"text": text, "dialect": "turtle"})
}

const HOOK_PACK: &str = "packs/self-monitoring-pack/hook.ttl";
const HOOK_SESSION: &str = "packs/self-monitoring-pack/fixtures/session-real-broad-topic.ttl";

/// Declared expectation per case; never inferred from the case name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expect {
    Admitted,
    Refused,
}

type Case = (String, Expect, Value);

fn canonical_cases() -> Vec<Case> {
    let mut v = vec![
        (
            "canonical_doc".to_string(),
            Expect::Admitted,
            json!({"op": "canonical", "data": ttl(DOC_TTL)}),
        ),
        (
            "canonical_reordered_prefix_renamed".to_string(),
            Expect::Admitted,
            json!({"op": "canonical", "data": ttl(DOC_TTL_REORDERED)}),
        ),
        (
            "canonical_changed_statement".to_string(),
            Expect::Admitted,
            json!({"op": "canonical",
                   "data": ttl(&DOC_TTL.replace("ex:carol", "ex:dave"))}),
        ),
        (
            "canonical_bnode_iso_a".to_string(),
            Expect::Admitted,
            json!({"op": "canonical", "data": {"text": BNQ_AB, "dialect": "nquads"}}),
        ),
        (
            "canonical_bnode_iso_b".to_string(),
            Expect::Admitted,
            json!({"op": "canonical", "data": {"text": BNQ_PQ, "dialect": "nquads"}}),
        ),
        (
            "canonical_invalid_rdf_refused".to_string(),
            Expect::Refused,
            json!({"op": "canonical", "data": ttl("not rdf at all <<<")}),
        ),
    ];
    // six run12 portability vectors through the canonical op
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/sa2a/portable/run12");
    let mut files: Vec<_> = std::fs::read_dir(&root)
        .expect("run12 corpus present")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
        .collect();
    files.sort();
    for (i, p) in files.into_iter().take(6).enumerate() {
        let text = std::fs::read_to_string(&p).unwrap();
        v.push((
            format!("canonical_run12_{i:02}"),
            Expect::Admitted,
            json!({"op": "canonical", "data": ttl(&text)}),
        ));
    }
    v
}

fn cases() -> Vec<Case> {
    let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> . @prefix ex: <https://e/> . @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
                  ex:S a sh:NodeShape ; sh:targetClass ex:T ; sh:property [ sh:path ex:p ; sh:datatype xsd:integer ] .";
    let min_count_shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> . @prefix ex: <https://e/> .\n\
                            ex:S a sh:NodeShape ; sh:targetClass ex:T ; sh:property [ sh:path ex:name ; sh:minCount 1 ] .";
    let sample = "@prefix ex: <https://e/> . ex:s ex:p ex:o, \"caf\\u00e9\"@fr, 42 ; a ex:T .";
    let two_bad = "<https://e/a> a <https://e/T> .\n<https://e/b> a <https://e/T> .\n";
    let rdfs_data = json!({"text": "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n<urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n", "dialect": "ntriples"});
    let entail_data = json!({"text": "@prefix ex: <https://e/> . @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> . ex:Cat rdfs:subClassOf ex:Animal . ex:tom a ex:Cat .", "dialect": "turtle"});
    let policy_problem = json!({
        "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"], "goal": {"facts": ["done"]},
        "transitions": [
            {"action": "flip", "from": "s0", "to": "g", "probability_ppm": 500000},
            {"action": "flip", "from": "s0", "to": "s0", "probability_ppm": 500000}]});
    let skewed = json!({"policy": [{"state": "s0", "action": "flip", "outcomes": [
        {"state": "g", "probability_ppm": 400000}, {"state": "s0", "probability_ppm": 500000}]}]});
    let good_policy = json!({"policy": [{"state": "s0", "action": "flip", "outcomes": [
        {"state": "g", "probability_ppm": 500000}, {"state": "s0", "probability_ppm": 500000}]}]});

    let mut v: Vec<Case> = vec![
        ("capabilities".into(),
            Expect::Admitted, json!({"op": "capabilities"})),
        // law steps: n3, rdfs, one entailment regime, datalog, hooks
        (
            "law_rdfs".into(),
            Expect::Admitted,
            law(rdfs_data.clone(), json!([{"step": "rdfs"}])),
        ),
        (
            "law_n3".into(),
            Expect::Admitted,
            law(
                ttl("@prefix : <https://e/> . :s a :Human ."),
                json!([{"step": "n3", "rules": "@prefix : <https://e/> . { ?x a :Human } => { ?x a :Mortal } ."}]),
            ),
        ),
        (
            "entail_regime_rdfs".into(),
            Expect::Admitted,
            json!({"op": "entail", "data": entail_data, "regime": "rdfs"}),
        ),
        (
            "entail_regime_owl_rl".into(),
            Expect::Admitted,
            json!({"op": "entail", "data": entail_data, "regime": "owl-rl"}),
        ),
        (
            "datalog_least_fixpoint".into(),
            Expect::Admitted,
            json!({"op": "datalog",
                "rules": [
                    {"head": ["?x", "https://e/anc", "?y"], "body": [["?x", "https://e/par", "?y"]]},
                    {"head": ["?x", "https://e/anc", "?z"], "body": [["?x", "https://e/anc", "?y"], ["?y", "https://e/par", "?z"]]}],
                "facts": [["https://e/a", "https://e/par", "https://e/b"], ["https://e/b", "https://e/par", "https://e/c"]]}),
        ),
        (
            "hooks_firings".into(),
            Expect::Admitted,
            json!({"op": "hooks",
                "pack": {"text": common::read(HOOK_PACK), "dialect": "turtle"},
                "data": {"text": common::read(HOOK_SESSION), "dialect": "turtle"}}),
        ),
        // SHACL conforming + 2-violation (byte-identical violation arrays)
        (
            "shacl_conforming".into(),
            Expect::Admitted,
            json!({"op": "shacl", "data": ttl("@prefix ex: <https://e/> . ex:s a ex:T ; ex:p 1 ."), "shapes": shapes}),
        ),
        (
            "shacl_two_violations".into(),
            Expect::Admitted,
            json!({"op": "shacl", "data": ttl(sample), "shapes": shapes}),
        ),
        // receipt chaining
        (
            "receipt_chain_record_require".into(),
            Expect::Admitted,
            law(
                json!({"text": at("a"), "dialect": "ntriples"}),
                json!([
                    {"step": "plan", "plan": {
                        "actions": [{"name": "a-b", "pre": at("a"), "add": at("b"), "del": at("a")}],
                        "goal": at("b")}},
                    {"step": "record-receipts"},
                    {"step": "require-receipt", "step_name": "plan-action"}]),
            ),
        ),
        // plan admission: happy, pre_not refusal, goal_not refusal
        (
            "plan_admission_happy".into(),
            Expect::Admitted,
            law(
                json!({"text": at("a"), "dialect": "ntriples"}),
                json!([{"step": "plan", "plan": {
                    "actions": [{"name": "a-b", "pre": at("a"), "add": at("b"), "del": at("a")},
                                {"name": "b-c", "pre": at("b"), "add": at("c"), "del": at("b")}],
                    "goal": at("c")}}]),
            ),
        ),
        (
            "plan_pre_not_blocked".into(),
            Expect::Refused,
            law(
                json!({"text": format!("{}<urn:c:b> <urn:p:occupied> <urn:v:yes> .\n", at("a")), "dialect": "ntriples"}),
                json!([{"step": "plan", "plan": {
                    "actions": [{"name": "a-b", "pre": at("a"), "pre_not": "<urn:c:b> <urn:p:occupied> <urn:v:yes> .\n",
                                 "add": at("b"), "del": at("a")}],
                    "goal": at("b")}}]),
            ),
        ),
        (
            "plan_pre_not_free".into(),
            Expect::Admitted,
            law(
                json!({"text": at("a"), "dialect": "ntriples"}),
                json!([{"step": "plan", "plan": {
                    "actions": [{"name": "a-b", "pre": at("a"), "pre_not": "<urn:c:b> <urn:p:occupied> <urn:v:yes> .\n",
                                 "add": at("b"), "del": at("a")}],
                    "goal": at("b"), "goal_not": at("a")}}]),
            ),
        ),
        (
            "plan_goal_not_refusal".into(),
            Expect::Refused,
            law(
                json!({"text": at("a"), "dialect": "ntriples"}),
                json!([{"step": "plan", "plan": {
                    "actions": [{"name": "a-b", "pre": at("a"), "add": at("b"), "del": at("a")}],
                    "goal": at("b"), "goal_not": at("b")}}]),
            ),
        ),
        // all five typed refusals
        (
            "refusal_NotAdmitted".into(),
            Expect::Refused,
            law(
                ttl(two_bad),
                json!([{"step": "shacl", "shapes": min_count_shapes}]),
            ),
        ),
        (
            "refusal_PlanRefused".into(),
            Expect::Refused,
            law(
                json!({"text": at("a"), "dialect": "ntriples"}),
                json!([{"step": "plan", "plan": {"actions": [
                    {"name": "a-b", "pre": at("z"), "add": at("b"), "del": at("a")}], "goal": at("b")}}]),
            ),
        ),
        (
            "refusal_PolicyRefused".into(),
            Expect::Refused,
            json!({"op": "policy", "problem": policy_problem, "policy": skewed}),
        ),
        (
            "refusal_ReceiptRequired".into(),
            Expect::Refused,
            law(
                json!({"text": at("a"), "dialect": "ntriples"}),
                json!([{"step": "require-receipt", "step_name": "derive:rdfs"}]),
            ),
        ),
        // signed-lease refusals (LeaseRefused / UnverifiedLeaseRefused), deterministic from_seed keys
        (
            "lease_signed_happy".into(),
            Expect::Admitted,
            signed_req(&issuer_key(), 4_000_000_000, 50),
        ),
        (
            "refusal_LeaseRefused_untrusted".into(),
            Expect::Refused,
            signed_req(&other_key(), 4_000_000_000, 50),
        ),
        (
            "refusal_LeaseRefused_expired".into(),
            Expect::Refused,
            signed_req(&issuer_key(), 100, 100),
        ),
        (
            "refusal_UnverifiedLeaseRefused".into(),
            Expect::Refused,
            unsigned_lease_req(),
        ),
        (
            "policy_happy".into(),
            Expect::Admitted,
            json!({"op": "policy", "problem": policy_problem, "policy": good_policy}),
        ),
    ];

    // the six ops that had no differential coverage: admitted + refused each,
    // minimal requests from registry/op-examples.json
    let ex_ttl = "@prefix ex: <https://e/> . ex:s ex:p ex:o, \"caf\\u00e9\"@fr, 42 ; a ex:T .";
    let ex_data = json!({"dialect": "turtle", "text": ex_ttl});
    let shex_data = json!({"text": "@prefix ex: <https://e/> . ex:cat ex:says \"meow\" . ex:rock ex:weight 5 ."});
    let shex_map = "<https://e/cat>@<https://e/Cat>, <https://e/rock>@<https://e/Cat>";
    v.extend([
        ("sniff_turtle".to_string(), Expect::Admitted,
         json!({"op": "sniff", "text": "@prefix e: <https://e/> . e:s e:p e:o ."})),
        ("sniff_html_refused".to_string(), Expect::Refused,
         json!({"op": "sniff", "text": "<!DOCTYPE html><html></html>"})),
        ("parse_turtle".to_string(), Expect::Admitted,
         json!({"op": "parse", "dialect": "turtle", "text": ex_ttl})),
        ("parse_turtle_invalid_refused".to_string(), Expect::Refused,
         json!({"op": "parse", "dialect": "turtle", "text": "<a> <b> ."})),
        ("convert_turtle_to_ntriples".to_string(), Expect::Admitted,
         json!({"op": "convert", "dialect": "turtle", "text": ex_ttl, "to": "ntriples"})),
        ("convert_non_rdf_target_refused".to_string(), Expect::Refused,
         json!({"op": "convert", "dialect": "turtle", "text": ex_ttl, "to": "n3"})),
        ("sparql_select".to_string(), Expect::Admitted,
         json!({"op": "sparql", "data": ex_data,
                "query": "SELECT ?o WHERE { <https://e/s> <https://e/p> ?o } ORDER BY ?o"})),
        ("sparql_syntax_refused".to_string(), Expect::Refused,
         json!({"op": "sparql", "data": ex_data, "query": "SELEKT nope"})),
        ("shex_shexc".to_string(), Expect::Admitted,
         json!({"op": "shex", "data": shex_data, "map": shex_map,
                "schema": "PREFIX ex: <https://e/>\nex:Cat { ex:says . }"})),
        ("shex_bad_schema_refused".to_string(), Expect::Refused,
         json!({"op": "shex", "data": shex_data, "map": shex_map, "schema": "ex:Cat {{{"})),
        ("n3_reason".to_string(), Expect::Admitted,
         json!({"op": "n3", "text": "@prefix : <https://e/> . :s a :Human . { ?x a :Human } => { ?x a :Mortal } ."})),
        ("n3_syntax_refused".to_string(), Expect::Refused,
         json!({"op": "n3", "text": "{ ?x a } =>"})),
    ]);
    v.extend(canonical_cases());
    v
}

// ---- deterministic Ed25519 keys and signed leases (attest::from_seed) ------

fn issuer_key() -> graphlaw::attest::SigningKey {
    graphlaw::attest::SigningKey::from_seed([3; 32])
}

fn other_key() -> graphlaw::attest::SigningKey {
    graphlaw::attest::SigningKey::from_seed([4; 32])
}

fn signed_lease_json(key: &graphlaw::attest::SigningKey, expires: u64) -> Value {
    use graphlaw::law::{Ceiling, Lease};
    let signed = graphlaw::attest::sign_lease(
        key,
        Lease {
            id: "L1".into(),
            holder: "h".into(),
            ceiling: Ceiling::Construct,
            scope: vec!["derive:rdfs".into()],
            expires_unix: expires,
            issued_unix: 0,
        },
    );
    json!({
        "lease": {"id": "L1", "holder": "h", "ceiling": "construct", "scope": ["derive:rdfs"],
                  "expires_unix": expires, "issued_unix": 0},
        "attestation": {"key_id": signed.attestation.key_id,
                        "payload_blake3": signed.attestation.payload_blake3,
                        "signature": signed.attestation.signature},
    })
}

fn signed_req(key: &graphlaw::attest::SigningKey, expires: u64, now: u64) -> Value {
    json!({"op": "law",
        "data": {"text": "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n<urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n", "dialect": "ntriples"},
        "signed_lease": signed_lease_json(key, expires),
        "trusted_keys": [issuer_key().verifying_key().to_hex()],
        "now_unix": now,
        "steps": [{"step": "rdfs"}]})
}

fn unsigned_lease_req() -> Value {
    json!({"op": "law",
        "data": {"text": "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n<urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n", "dialect": "ntriples"},
        "lease": {"id": "L1", "holder": "h", "ceiling": "construct", "scope": ["derive:rdfs"], "expires_unix": 100, "issued_unix": 0},
        "now_unix": 50,
        "steps": [{"step": "rdfs"}]})
}


// ---------------------------------------------------------------------------
// enumeration + refusal-code coverage courts (no artifact needed)
// ---------------------------------------------------------------------------

/// Refusal codes the differential does not witness, each with the reason.
/// Adding a code to the registry without a case or an entry here fails.
const UNWITNESSED: &[(&str, &str)] = &[
    ("Refused", "generic engine-refusal wrapper; kind-specific codes are the witnessed forms"),
    ("ResourceLimit", "needs multi-MiB payloads; covered by wasm_abi resource-limit courts"),
    ("ReceiptRefused", "needs a signed-receipt chain mutation; covered by wasm_abi f_wasm_require_signed_receipt_step"),
    ("DuckDb", "op `duckdb` is unadvertised from capabilities.ops; covered by the SQL-in-RDF courts"),
];

fn op_of(req: &Value) -> String {
    req["op"].as_str().unwrap_or("?").to_string()
}

#[test]
fn case_ops_equal_registry_op_names() {
    let used: std::collections::BTreeSet<String> =
        cases().iter().map(|(_, _, r)| op_of(r)).collect();
    let all: std::collections::BTreeSet<String> = graphlaw::registry::op_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let missing: Vec<_> = all.difference(&used).collect();
    let extra: Vec<_> = used.difference(&all).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "differential cases must cover exactly the registry ops; missing: {missing:?}; unknown: {extra:?}"
    );
}

#[test]
fn every_case_has_an_admitted_and_refused_witness_per_refusable_op() {
    // every op that has a refused case also has an admitted one
    for (_, e, r) in cases().iter().filter(|(_, e, _)| *e == Expect::Refused) {
        let op = op_of(r);
        assert!(
            cases().iter().any(|(_, e2, r2)| *e2 == Expect::Admitted && op_of(r2) == op),
            "op {op} ({e:?}) has a refused case but no admitted case"
        );
    }
}

#[test]
fn refusal_codes_are_witnessed_or_explicitly_unwitnessed() {
    // witnesses come from real native responses, not from case names
    let mut witnessed = std::collections::BTreeSet::new();
    for (name, e, req) in cases() {
        if e != Expect::Refused {
            continue;
        }
        let v: Value = serde_json::from_slice(&common::native_bytes(&req)).expect("native JSON");
        assert_eq!(v["ok"], json!(false), "{name}: declared Refused but admitted: {v}");
        if let Some(c) = v["error"]["details"]["code"].as_str() {
            witnessed.insert(c.to_string());
        }
    }
    let mut uncovered = Vec::new();
    for row in graphlaw::registry::REFUSAL_CODES {
        let listed = UNWITNESSED.iter().any(|(c, _)| *c == row.code);
        let seen = witnessed.contains(row.code);
        assert!(
            !(listed && seen),
            "{} is witnessed; remove it from UNWITNESSED",
            row.code
        );
        if !listed && !seen {
            uncovered.push(row.code);
        }
    }
    assert!(
        uncovered.is_empty(),
        "refusal codes with no case and no UNWITNESSED entry: {uncovered:?} (witnessed: {witnessed:?})"
    );
    for (c, reason) in UNWITNESSED {
        assert!(!reason.is_empty(), "{c}: empty reason");
        assert!(
            graphlaw::registry::REFUSAL_CODES.iter().any(|r| r.code == *c),
            "UNWITNESSED lists unknown code {c}"
        );
    }
}

// ---------------------------------------------------------------------------
// the court
// ---------------------------------------------------------------------------

#[test]
fn all_canonical_inputs_are_byte_identical_native_vs_wasm() {
    let cs = cases();
    let names: Vec<&str> = cs.iter().map(|(n, _, _)| n.as_str()).collect();
    if let Err(r) = wasm_artifact() {
        skip(&r, &names);
        return;
    }
    let mut refused_count = 0usize;
    for (index, (name, expect, req)) in cs.iter().enumerate() {
        match expect {
            Expect::Refused => {
                diff_refused(name, index, req);
                refused_count += 1;
            }
            Expect::Admitted => {
                diff_ok(name, index, req);
            }
        }
    }
    println!(
        "wasm_native_differential: {} cases, byte-identical ({} admitted, {} refused)",
        cs.len(),
        cs.len() - refused_count,
        refused_count
    );
    assert!(
        cs.len() >= 30,
        "court must run >= 30 canonical inputs, ran {}",
        cs.len()
    );
}

/// Non-vacuity: the byte-identity gate must be able to fail. Corrupt one byte
/// of a wasm response's receipt digest and assert the corrupt bytes are NOT
/// equal to the native response.
#[test]
fn corrupting_one_receipt_digest_byte_breaks_byte_identity() {
    if let Err(r) = wasm_artifact() {
        skip(
            &r,
            &["corrupting_one_receipt_digest_byte_breaks_byte_identity"],
        );
        return;
    }
    let req = law(
        json!({"text": "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n<urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n", "dialect": "ntriples"}),
        json!([{"step": "rdfs"}]),
    );
    // sanity: the uncorrupted pair is identical (this is the gate under test)
    let native = common::native_bytes(&req);
    let wasm = common::wasm_bytes(&req);
    assert_eq!(native, wasm, "uncorrupted pair must be identical");

    // corrupt one hex char of a blake3 digest inside the wasm response
    let hay = String::from_utf8_lossy(&wasm).into_owned();
    let marker = "blake3:";
    let pos = hay.find(marker).expect("response carries a blake3 digest");
    let mut corrupt = wasm.clone();
    let corrupt_at = pos + marker.len();
    let flip = |b: &mut u8| {
        *b = if *b == b'0' { b'1' } else { b'0' };
    };
    flip(&mut corrupt[corrupt_at]);
    assert_ne!(
        native, corrupt,
        "corrupting a digest byte must break byte identity (gate is non-vacuous)"
    );
}
