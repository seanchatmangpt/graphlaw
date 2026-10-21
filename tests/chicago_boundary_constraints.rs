#![cfg(feature = "abi")]
//! Court `chicago-boundary-constraints`: boundary-fixture coverage for the SHACL
//! constraint components smalt's POWL law compiler leans on with zero
//! graphlaw-local coverage (GLR-3 gap close): `sh:maxInclusive` /
//! `sh:minInclusive` (admit-at-boundary, refuse-past), `sh:disjoint`
//! (admit-disjoint / refuse-same-value), and `sh:closed`.
//!
//! Conventions follow tests/chicago_fibo.rs + tests/refusal_details.rs:
//! real purrdf SHACL engine through graphlaw's typed admission path
//! (`op: law` with an `admit:shacl` step — never raw purrdf), assertions on
//! the final response state, no mocks.
//!
//! Standing: finite corpus below passes on exact subject; authority NONE;
//! synthetic subject; proves only these boundary fixtures, not general SHACL
//! conformance.

use graphlaw::abi::call_json;
use serde_json::{Value, json};

const PREFIX: &str = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n@prefix ex: <https://e/> .\n";

fn law_req(data: &str, shapes: &str) -> Value {
    json!({
        "op": "law",
        "data": {"text": data, "dialect": "turtle"},
        "steps": [{"step": "shacl", "shapes": shapes}]
    })
}

fn run(req: &Value) -> Value {
    call_json(req)
}

fn admit(req: &Value) -> Value {
    let r = run(req);
    assert_eq!(r["ok"], true, "{r}");
    r
}

fn err_of(req: &Value) -> Value {
    let r = run(req);
    assert_eq!(r["ok"], false, "{r}");
    r["error"].clone()
}

fn violation_component(e: &Value) -> &str {
    e["details"]["violations"][0]["component"]
        .as_str()
        .unwrap_or_else(|| panic!("expected one typed violation, got {e}"))
}

// ---------------------------------------------------------------- maxInclusive

#[test]
fn max_inclusive_admits_at_boundary_and_refuses_past_it() {
    let shapes = format!(
        "{PREFIX}ex:Cap a sh:NodeShape ; sh:targetClass ex:Order ;\n  sh:property [ sh:path ex:amount ; sh:datatype xsd:integer ; sh:maxInclusive 100 ] .\n"
    );
    let at = "<https://e/o1> a <https://e/Order> ; <https://e/amount> 100 .\n";
    admit(&law_req(at, &shapes));

    let past = "<https://e/o1> a <https://e/Order> ; <https://e/amount> 101 .\n";
    let e = err_of(&law_req(past, &shapes));
    assert_eq!(e["kind"], "EngineRejected", "{e}");
    assert_eq!(e["details"]["code"], "NotAdmitted", "{e}");
    assert_eq!(e["message"], "SHACL admission refused: 1 violation(s)");
    let v = &e["details"]["violations"][0];
    assert!(v["component"].as_str().unwrap().contains("MaxInclusive"), "{v}");
    assert!(v["path"].as_str().unwrap().contains("https://e/amount"), "{v}");
    assert!(v["focus"].as_str().unwrap().contains("https://e/o1"), "{v}");
}

// ---------------------------------------------------------------- minInclusive

#[test]
fn min_inclusive_admits_at_boundary_and_refuses_below_it() {
    let shapes = format!(
        "{PREFIX}ex:Floor a sh:NodeShape ; sh:targetClass ex:Order ;\n  sh:property [ sh:path ex:amount ; sh:datatype xsd:integer ; sh:minInclusive 100 ] .\n"
    );
    let at = "<https://e/o1> a <https://e/Order> ; <https://e/amount> 100 .\n";
    admit(&law_req(at, &shapes));

    let below = "<https://e/o1> a <https://e/Order> ; <https://e/amount> 99 .\n";
    let e = err_of(&law_req(below, &shapes));
    assert_eq!(e["details"]["code"], "NotAdmitted", "{e}");
    assert!(violation_component(&e).contains("MinInclusive"), "{e}");
    let v = &e["details"]["violations"][0];
    assert!(v["path"].as_str().unwrap().contains("https://e/amount"), "{v}");
}

// -------------------------------------------------------------------- disjoint

/// `sh:disjoint` between two properties of one focus node: `ex:parent` values
/// must never also appear as `ex:child` values (and vice versa).
fn disjoint_shapes() -> String {
    format!(
        "{PREFIX}ex:Family a sh:NodeShape ; sh:targetClass ex:Person ;\n  sh:property [ sh:path ex:parent ; sh:disjoint ex:child ] .\n"
    )
}

#[test]
fn disjoint_admits_disjoint_values_and_refuses_shared_value() {
    let shapes = disjoint_shapes();
    let ok = "<https://e/p> a <https://e/Person> ; <https://e/parent> <https://e/g1> ; <https://e/child> <https://e/c1> .\n";
    admit(&law_req(ok, &shapes));

    let bad = "<https://e/p> a <https://e/Person> ; <https://e/parent> <https://e/x> ; <https://e/child> <https://e/x> .\n";
    let e = err_of(&law_req(bad, &shapes));
    assert_eq!(e["details"]["code"], "NotAdmitted", "{e}");
    assert!(violation_component(&e).contains("Disjoint"), "{e}");
    let v = &e["details"]["violations"][0];
    assert!(v["focus"].as_str().unwrap().contains("https://e/p"), "{v}");
}

// ---------------------------------------------------------------------- closed

/// smalt's agent-bounding court consumes `sh:closed`: undeclared predicates
/// refuse, declared-only data admits.
#[test]
fn closed_shape_admits_declared_properties_and_refuses_undeclared() {
    let shapes = format!(
        "{PREFIX}ex:Agent a sh:NodeShape ; sh:targetClass ex:Agent ; sh:closed true ; sh:ignoredProperties ( rdf:type ) ;\n  sh:property [ sh:path ex:name ] ;\n  sh:property [ sh:path ex:role ] .\n"
    );
    let ok = "<https://e/a> a <https://e/Agent> ; <https://e/name> \"r\" ; <https://e/role> \"w\" .\n";
    admit(&law_req(ok, &shapes));

    let bad = "<https://e/a> a <https://e/Agent> ; <https://e/name> \"r\" ; <https://e/side_effect> <https://e/x> .\n";
    let e = err_of(&law_req(bad, &shapes));
    assert_eq!(e["details"]["code"], "NotAdmitted", "{e}");
    assert!(violation_component(&e).contains("Closed"), "{e}");
    let v = &e["details"]["violations"][0];
    assert!(v["focus"].as_str().unwrap().contains("https://e/a"), "{v}");
}
