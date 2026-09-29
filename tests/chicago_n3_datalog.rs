//! Chicago 80/20 canonical N3 + Datalog compliance court.
//!
//! This is intentionally not a full standards suite. It selects a compact set
//! of high-value semantic invariants from the W3C N3 semantics and the canonical
//! Soufflé transitive-closure tutorial, then requires GraphLaw's admitted
//! upstream authorities to agree on the shared closure.
//!
//! Sources:
//! - https://w3c.github.io/N3/spec/
//! - https://w3c.github.io/N3/spec/semantics.html
//! - https://souffle-lang.github.io/tutorial
//!
//! Standing ceiling: passing this court proves representative canonical
//! behavior for the cases below. It does not claim complete W3C N3, Soufflé,
//! OpenRuleBench, or Datalog conformance.

#![cfg(feature = "abi")]

use std::collections::BTreeSet;

use graphlaw::abi::call;
use serde_json::{Value, json};

const NS: &str = "https://chicago.graphlaw.dev/";

fn run(req: Value) -> Value {
    serde_json::from_slice(&call(req.to_string().as_bytes())).expect("GraphLaw ABI returns JSON")
}

fn ok(req: Value) -> Value {
    let out = run(req.clone());
    assert_eq!(out["ok"], true, "request {req} was refused: {out}");
    out
}

fn canonical_n3(program: &str) -> String {
    let out = ok(json!({"op": "n3", "text": program}));
    let derived = out["derived"].as_str().expect("N3 result text");
    let dataset =
        graphlaw::rdf::parse_dataset(derived.as_bytes(), "text/turtle", None).expect("N3 result is RDF");
    graphlaw::rdf::try_canonicalize(dataset.as_ref())
        .expect("RDFC canonicalization")
        .nquads
}

fn n3_pairs(program: &str, predicate: &str) -> BTreeSet<(String, String)> {
    let out = ok(json!({"op": "n3", "text": program}));
    let derived = out["derived"].as_str().expect("N3 result text");
    let dataset =
        graphlaw::rdf::parse_dataset(derived.as_bytes(), "text/turtle", None).expect("N3 result is RDF");
    let bytes = graphlaw::rdf::serialize_dataset(
        dataset.as_ref(),
        "application/n-triples",
        graphlaw::rdf::SerializeGraph::Dataset,
    )
    .expect("serialize N3 closure");

    let target = format!("<{predicate}>");
    String::from_utf8(bytes)
        .expect("N-Triples UTF-8")
        .lines()
        .filter_map(|line| {
            let mut terms = line.split_whitespace();
            let subject = terms.next()?;
            let pred = terms.next()?;
            let object = terms.next()?;
            (pred == target).then(|| {
                (
                    subject.trim_matches(['<', '>']).to_string(),
                    object.trim_matches(['<', '>']).to_string(),
                )
            })
        })
        .collect()
}

fn datalog_request(facts: Vec<Value>, reverse_rules: bool) -> Value {
    let mut rules = vec![
        json!({
            "head": ["?x", format!("{NS}reachable"), "?y"],
            "body": [["?x", format!("{NS}edge"), "?y"]]
        }),
        json!({
            "head": ["?x", format!("{NS}reachable"), "?z"],
            "body": [
                ["?x", format!("{NS}reachable"), "?y"],
                ["?y", format!("{NS}edge"), "?z"]
            ]
        }),
    ];
    if reverse_rules {
        rules.reverse();
    }
    json!({"op": "datalog", "rules": rules, "facts": facts})
}

fn edge_facts() -> Vec<Value> {
    [
        ("a", "b"),
        ("b", "c"),
        ("c", "b"),
        ("c", "d"),
    ]
    .into_iter()
    .map(|(s, o)| {
        json!([
            format!("{NS}{s}"),
            format!("{NS}edge"),
            format!("{NS}{o}")
        ])
    })
    .collect()
}

fn datalog_reachable(out: &Value) -> BTreeSet<(String, String)> {
    out["facts"]
        .as_array()
        .expect("Datalog facts")
        .iter()
        .filter_map(|fact| {
            let f = fact.as_array()?;
            (f.get(1)?.as_str()? == format!("{NS}reachable")).then(|| {
                (
                    f[0].as_str().unwrap().to_string(),
                    f[2].as_str().unwrap().to_string(),
                )
            })
        })
        .collect()
}

fn expected_reachable() -> BTreeSet<(String, String)> {
    [
        ("a", "b"),
        ("a", "c"),
        ("a", "d"),
        ("b", "b"),
        ("b", "c"),
        ("b", "d"),
        ("c", "b"),
        ("c", "c"),
        ("c", "d"),
    ]
    .into_iter()
    .map(|(s, o)| (format!("{NS}{s}"), format!("{NS}{o}")))
    .collect()
}

fn n3_transitive_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         :a :edge :b .\n\
         :b :edge :c .\n\
         :c :edge :b .\n\
         :c :edge :d .\n\
         {{ ?x :edge ?y }} => {{ ?x :reachable ?y }} .\n\
         {{ ?x :reachable ?y . ?y :edge ?z }} => {{ ?x :reachable ?z }} .\n"
    )
}

#[test]
fn w3c_n3_implication_and_chaining_fire() {
    // Distilled from the W3C N3 rule semantics: when a premise matches, its
    // conclusion is entailed. A second rule proves chaining reaches fixpoint.
    let program = format!(
        "@prefix : <{NS}> .\n\
         :weather a :Raining .\n\
         {{ ?x a :Raining }} => {{ ?x a :Cloudy }} .\n\
         {{ ?x a :Cloudy }} => {{ ?x a :ObservableWeather }} .\n"
    );
    let closure = canonical_n3(&program);
    assert!(
        closure.contains(&format!("<{NS}weather>")) &&
        closure.contains(&format!("<{NS}ObservableWeather>")),
        "{closure}"
    );
}

#[test]
fn n3_semantics_are_invariant_to_statement_order() {
    let left = format!(
        "@prefix : <{NS}> .\n\
         :s a :Human .\n\
         {{ ?x a :Human }} => {{ ?x a :Mortal }} .\n\
         {{ ?x a :Mortal }} => {{ ?x a :Finite }} .\n"
    );
    let right = format!(
        "@prefix : <{NS}> .\n\
         {{ ?x a :Mortal }} => {{ ?x a :Finite }} .\n\
         {{ ?x a :Human }} => {{ ?x a :Mortal }} .\n\
         :s a :Human .\n"
    );
    assert_eq!(canonical_n3(&left), canonical_n3(&right));
}

#[test]
fn souffle_transitive_closure_example_has_exact_least_fixpoint() {
    // Canonical Soufflé tutorial shape: base edge rule + recursive reachable
    // rule, including a cycle. RDF triples encode the binary relations.
    let out = ok(datalog_request(edge_facts(), false));
    assert_eq!(datalog_reachable(&out), expected_reachable(), "{out}");
}

#[test]
fn datalog_fixpoint_is_invariant_to_fact_and_rule_order() {
    let forward = ok(datalog_request(edge_facts(), false));

    let mut reversed_facts = edge_facts();
    reversed_facts.reverse();
    let reversed = ok(datalog_request(reversed_facts, true));

    assert_eq!(forward["facts"], reversed["facts"]);
    assert_eq!(datalog_reachable(&forward), expected_reachable());
}

#[test]
fn n3_and_datalog_agree_on_the_shared_recursive_closure() {
    let n3 = n3_pairs(&n3_transitive_program(), &format!("{NS}reachable"));
    let dl = datalog_reachable(&ok(datalog_request(edge_facts(), false)));
    assert_eq!(n3, expected_reachable());
    assert_eq!(dl, expected_reachable());
    assert_eq!(n3, dl);
}

#[test]
fn malformed_n3_and_malformed_datalog_fail_closed() {
    let n3 = run(json!({"op": "n3", "text": "{ ?x a } =>"}));
    assert_eq!(n3["ok"], false, "{n3}");
    assert_eq!(n3["error"]["engine"], "Eyeron", "{n3}");

    let datalog = run(json!({
        "op": "datalog",
        "rules": [{
            "head": ["?x", format!("{NS}reachable"), "?y"],
            "body": "not-an-array"
        }],
        "facts": []
    }));
    assert_eq!(datalog["ok"], false, "{datalog}");
}
