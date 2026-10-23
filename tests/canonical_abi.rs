//! Pins the `canonical` ABI op contract: RDFC-1.0 canonical form + BLAKE3
//! identity through `graphlaw::abi::call` — the same surface the wasm kernel
//! (`gl_call`) exposes to hosts.
use serde_json::{Value, json};

fn call(v: Value) -> Value {
    let out = graphlaw::abi::call(serde_json::to_vec(&v).unwrap().as_slice());
    serde_json::from_slice(&out).expect("response is JSON")
}

fn ok(v: Value) -> Value {
    let r = call(v);
    assert_eq!(r["ok"], json!(true), "expected ok, got {r}");
    r
}

const DOC_TTL: &str = r#"@prefix ex: <http://example.org/> .
ex:alice ex:knows ex:bob .
ex:bob ex:knows ex:carol .
"#;

// Same dataset, reordered statements and renamed prefixes.
const DOC_TTL_REORDERED: &str = r#"@prefix p: <http://example.org/> .
p:bob p:knows p:carol .
p:alice p:knows p:bob .
"#;

#[test]
fn canonical_returns_rdfc_id_and_canonical_nquads() {
    let r = ok(json!({"op": "canonical", "data": {"text": DOC_TTL, "dialect": "turtle"}}));
    let id = r["id"].as_str().expect("id is a string");
    assert!(id.starts_with("blake3:"), "id is blake3:-prefixed: {id}");
    assert_eq!(id.len(), 7 + 64, "id is a full blake3 hex digest");
    let nq = r["nquads"].as_str().expect("nquads is a string");
    assert!(
        nq.contains("<http://example.org/alice>"),
        "canonical form present: {nq}"
    );
    assert_eq!(r["quads"], json!(2));
}

#[test]
fn canonical_identity_invariant_under_reordering_and_prefix_renaming() {
    let a = ok(json!({"op": "canonical", "data": {"text": DOC_TTL, "dialect": "turtle"}}));
    let b = ok(json!({"op": "canonical",
                      "data": {"text": DOC_TTL_REORDERED, "dialect": "turtle"}}));
    assert_eq!(a["id"], b["id"], "identity must not drift with syntax");
    assert_eq!(a["nquads"], b["nquads"]);
}

#[test]
fn canonical_identity_moves_when_a_statement_changes() {
    let a = ok(json!({"op": "canonical", "data": {"text": DOC_TTL, "dialect": "turtle"}}));
    let changed = DOC_TTL.replace("ex:carol", "ex:dave");
    let b = ok(json!({"op": "canonical", "data": {"text": &changed, "dialect": "turtle"}}));
    assert_ne!(a["id"], b["id"], "identity must move with content");
}

#[test]
fn canonical_refuses_invalid_rdf_with_typed_error() {
    let r = call(
        json!({"op": "canonical", "data": {"text": "not rdf at all <<<", "dialect": "turtle"}}),
    );
    assert_eq!(r["ok"], json!(false));
    let err = &r["error"];
    assert!(
        err["message"].as_str().is_some(),
        "error carries a message: {r}"
    );
}

#[test]
fn canonical_handles_blank_node_isomorphism() {
    let a = ok(json!({"op": "canonical",
                      "data": {"text": "_:x <http://example.org/knows> _:y .\n_:y <http://example.org/knows> _:x .\n", "dialect": "nquads"}}));
    let b = ok(json!({"op": "canonical",
                      "data": {"text": "_:p <http://example.org/knows> _:q .\n_:q <http://example.org/knows> _:p .\n"}}));
    assert_eq!(
        a["id"], b["id"],
        "blank-node relabeling must not move identity"
    );
}
