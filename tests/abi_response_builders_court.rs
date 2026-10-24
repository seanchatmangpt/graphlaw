#![cfg(feature = "abi")]
//! SQLRDF-28 court: the ABI response builders and their helpers
//! (`limit_response`, `missing_buffer_response`, `unknown_buffer_response`,
//! and — through the public JSON ABI only, no visibility widening —
//! `spec_of`, `state_of`, `str_field`, `term_json`, `check_limit`).
//! Real dispatch, real parsers, real SPARQL engine — no mocks.
//!
//! Coverage note: `term_json`'s `triple` arm is unreachable through SPARQL
//! results (a solution-row cell is an optional term, never a quoted triple
//! here), so it is left uncovered, same as the rest of the suite.
//!
//! Every refusal test is paired with a positive control (anti-vacuity twins
//! noted per test); messages are exact-equality and `details` are whole-map
//! equality so message-mangle and key-drop mutants die.

use graphlaw::abi::{
    MAX_JSON_DEPTH, call_json, limit_response, missing_buffer_response, unknown_buffer_response,
};
use serde_json::{Value, json};

/// Run one request through the ABI, assert `ok == false`, return the error object.
fn err_of(req: &Value) -> Value {
    let r = call_json(req);
    assert_eq!(r["ok"], false, "{r}");
    r["error"].clone()
}

/// Exactly the four keys of the `ResourceLimit` details map — whole-map
/// equality so key-drop mutants in `limit()` die.
fn assert_details_exact(r: &Value, limit: &str, observed: usize, max: usize) {
    assert_eq!(
        r["error"]["details"],
        json!({"code": "ResourceLimit", "limit": limit, "observed": observed, "max": max}),
        "{r}"
    );
}

/// The depth-twin request builder: `n` nested arrays around a payload value.
/// `json_depth` counts every `[`/`{` (string-aware), so `n` arrays plus the
/// payload object's own braces give a request depth of `n + 1`.
fn wrapped(n: usize, payload: Value) -> Value {
    let mut v = payload;
    for _ in 0..n {
        v = Value::Array(vec![v]);
    }
    v
}

/// Three-term N-Triples fixture: IRI object, blank-node subject with a lang
/// literal, and a typed integer literal.
fn nt() -> &'static str {
    "<urn:s> <urn:p> <urn:o> .\n\
     _:b <urn:l> \"chat\"@en .\n\
     <urn:s> <urn:n> \"1\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n"
}

fn sparql(query: &str) -> Value {
    json!({"op": "sparql", "data": {"text": nt(), "dialect": "ntriples"}, "query": query})
}

/// Run a SELECT returning exactly one row / one variable; return the single cell.
fn one_cell(query: &str) -> Value {
    let r = call_json(&sparql(query));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["kind"], "solutions", "{r}");
    let rows = r["rows"].as_array().expect("rows array");
    assert_eq!(rows.len(), 1, "{r}");
    let row = rows[0].as_array().expect("row array");
    assert_eq!(row.len(), 1, "{r}");
    row[0].clone()
}

// ---------------------------------------------------------------------------
// 1-3. The direct response builders (pub, used by the wasm shell).
// ---------------------------------------------------------------------------

/// Kills the `limit_response -> vec![]` survivor: `vec![]` does not parse as
/// JSON, so `from_slice` fails. Positive control for the serialization path
/// is test 10 (`capabilities`).
#[test]
fn limit_response_emits_typed_resource_limit_json() {
    let bytes = limit_response("request_bytes", 5, 4);
    let r: Value = serde_json::from_slice(&bytes).expect("limit_response emits JSON");
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
    assert_eq!(
        r["error"]["message"],
        "resource limit `request_bytes` exceeded: 5 > 4",
        "{r}"
    );
    assert_details_exact(&r, "request_bytes", 5, 4);
}

/// Kills bad()/message-mangle mutants of `missing_buffer_response`.
#[test]
fn missing_buffer_response_names_gl_alloc() {
    let bytes = missing_buffer_response();
    let r: Value = serde_json::from_slice(&bytes).expect("missing_buffer_response emits JSON");
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["kind"], "Unsupported", "{r}");
    assert_eq!(
        r["error"]["message"],
        "request buffer missing: gl_alloc refused or was never called",
        "{r}"
    );
    assert!(
        r["error"].get("details").is_none(),
        "bad() sets no details: {r}"
    );
}

/// Kills message-mangle mutants of `unknown_buffer_response`.
#[test]
fn unknown_buffer_response_names_live_allocation() {
    let bytes = unknown_buffer_response();
    let r: Value = serde_json::from_slice(&bytes).expect("unknown_buffer_response emits JSON");
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["kind"], "Unsupported", "{r}");
    assert_eq!(
        r["error"]["message"],
        "request buffer is not a live allocation from gl_alloc",
        "{r}"
    );
    assert!(
        r["error"].get("details").is_none(),
        "bad() sets no details: {r}"
    );
}

// ---------------------------------------------------------------------------
// 4-6. spec_of / str_field refusals through the public JSON ABI.
// ---------------------------------------------------------------------------

/// Kills the spec_of fallthrough (`_`) arm-drop mutant.
/// Positive control: the same op with a real data spec succeeds (test 8).
#[test]
fn call_json_refuses_missing_data_spec() {
    let e = err_of(&json!({"op": "sparql", "query": "SELECT * WHERE { ?s ?p ?o }"}));
    assert_eq!(e["kind"], "Unsupported", "{e}");
    assert_eq!(e["message"], "missing data spec `data`", "{e}");
}

/// Kills the string-arm swap/drop mutant of spec_of.
#[test]
fn call_json_refuses_string_data_spec() {
    let e = err_of(&json!({
        "op": "sparql",
        "data": "<urn:s> <urn:p> <urn:o> .",
        "query": "SELECT * WHERE { ?s ?p ?o }"
    }));
    assert_eq!(e["kind"], "Unsupported", "{e}");
    assert_eq!(
        e["message"],
        "`data` must be a data spec object {\"text\": ...}",
        "{e}"
    );
}

/// Kills the str_field family mutants (missing / non-string `query`).
#[test]
fn call_json_refuses_missing_string_field() {
    let e = err_of(&json!({"op": "sparql", "data": {"text": "", "dialect": "ntriples"}}));
    assert_eq!(e["kind"], "Unsupported", "{e}");
    assert_eq!(e["message"], "missing string field `query`", "{e}");
}

// ---------------------------------------------------------------------------
// 7. state_of refusal + positive control (real parser, real refusal route).
// ---------------------------------------------------------------------------

#[test]
fn state_of_refuses_bad_text_and_accepts_good_text() {
    // Refusal leg: real parser refuses; routed through state_of -> LawState::parse.
    let e = err_of(&json!({
        "op": "law",
        "data": {"text": "not turtle", "dialect": "turtle"},
        "steps": []
    }));
    assert_ne!(e["message"], "", "refusal carries a message: {e}");
    // Positive twin: valid (empty) N-Triples state with no steps is admitted.
    let r = call_json(&json!({
        "op": "law",
        "data": {"text": "", "dialect": "ntriples"},
        "steps": []
    }));
    assert_eq!(r["ok"], true, "{r}");
}

// ---------------------------------------------------------------------------
// 8. term_json arms through real SPARQL results.
// ---------------------------------------------------------------------------

/// Positive control for tests 4-6 and driver for the term_json arms.
#[test]
fn call_json_emits_term_json_shapes_via_sparql() {
    let r = call_json(&sparql("SELECT * WHERE { ?s ?p ?o }"));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["kind"], "solutions", "{r}");
    assert_eq!(r["variables"], json!(["s", "p", "o"]), "{r}");
    assert!(!r["rows"].as_array().expect("rows").is_empty(), "{r}");

    // IRI arm: exact object equality.
    assert_eq!(
        one_cell("SELECT ?o WHERE { <urn:s> <urn:p> ?o }"),
        json!({"type": "uri", "value": "urn:o"}),
        "iri arm"
    );

    // Typed-literal arm: exact object equality; `datatype` is a JSON STRING
    // (not nested), the full datatype IRI.
    assert_eq!(
        one_cell("SELECT ?n WHERE { <urn:s> <urn:n> ?n }"),
        json!({
            "type": "literal",
            "value": "1",
            "datatype": "http://www.w3.org/2001/XMLSchema#integer"
        }),
        "typed-literal arm"
    );

    // Language-literal arm: `xml:lang` present, datatype not drifted away.
    let cell = one_cell("SELECT ?l WHERE { _:b <urn:l> ?l }");
    assert_eq!(cell["type"], "literal", "{cell}");
    assert_eq!(cell["value"], "chat", "{cell}");
    assert_eq!(cell["xml:lang"], "en", "{cell}");
    assert_eq!(
        cell.as_object().expect("cell object").len(),
        4,
        "literal+lang has exactly type/value/datatype/xml:lang: {cell}"
    );

    // Blank-node arm: `type == "bnode"` and a nonempty label (label naming is
    // parser-owned, so only its presence is pinned).
    let cell = one_cell("SELECT ?s WHERE { ?s <urn:l> \"chat\"@en }");
    assert_eq!(cell["type"], "bnode", "{cell}");
    assert!(
        cell["value"].as_str().expect("bnode label string").len() > 0,
        "{cell}"
    );
}

// ---------------------------------------------------------------------------
// 9. check_limit via the json_depth gate — the cap pair.
// ---------------------------------------------------------------------------

/// Depth `MAX_JSON_DEPTH + 1` (64 arrays + the payload object = 65) is refused
/// before dispatch ever sees it.
#[test]
fn json_depth_over_cap_refused() {
    let over = MAX_JSON_DEPTH; // 64 arrays -> request depth 65
    let e = err_of(&wrapped(over, json!({"op": "capabilities"})));
    assert_eq!(e["kind"], "ResourceLimit", "{e}");
    assert_eq!(
        e["message"],
        "resource limit `json_depth` exceeded: 65 > 64",
        "{e}"
    );
    let d = &e["details"];
    assert_eq!(d["observed"], 65, "{e}");
    assert_eq!(d["max"], MAX_JSON_DEPTH, "{e}");
    assert_eq!(
        d,
        &json!({"code": "ResourceLimit", "limit": "json_depth", "observed": 65, "max": 64}),
        "{e}"
    );
}

/// TWIN (kills the `check_limit: replace > with >=` mutant): the same nesting
/// at exactly the cap (63 arrays + payload object = 64) is admitted and the
/// wrapped request reaches dispatch intact.
#[test]
fn json_depth_at_exactly_the_cap_admitted() {
    let at_cap = MAX_JSON_DEPTH - 1; // 63 arrays -> request depth 64
    let r = call_json(&wrapped(at_cap, json!({"op": "capabilities"})));
    assert_eq!(r["ok"], true, "{r}");
}
