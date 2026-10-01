#![cfg(feature = "pack-tools")]
//! The Rust tools that replaced the Python scripts, checked against the
//! outputs the Python scripts produced (committed fixtures) and end to end
//! against `hook.ttl`.

use graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult, TermValue};
use graphlaw::smon::{broaden, build_dataset, read_turtle, to_turtle, turns_from_jsonl};
use graphlaw::sparql::NativeSparqlEngine;
use std::path::PathBuf;

fn pack(p: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("packs/self-monitoring-pack")
        .join(p)
}

fn canonical(ds: &graphlaw::rdf::RdfDataset) -> String {
    graphlaw::rdf::try_canonicalize(ds)
        .expect("canonicalizes")
        .nquads
}

fn hook_rows(ds: &std::sync::Arc<graphlaw::rdf::RdfDataset>) -> usize {
    let hook = read_turtle(&pack("hook.ttl")).unwrap();
    let q = match NativeSparqlEngine::new()
        .query(
            &hook,
            SparqlRequest {
                query: "SELECT ?q WHERE { <http://seanchatmangpt.github.io/packs/self-monitoring#derive_escalation_obligation> <http://seanchatmangpt.github.io/praxis/kh#query> ?q }",
                base_iri: None,
                substitutions: &[],
            },
        )
        .unwrap()
    {
        SparqlResult::Solutions { rows, .. } => match rows[0][0].clone().unwrap() {
            TermValue::Literal { lexical_form, .. } => lexical_form,
            o => panic!("{o:?}"),
        },
        o => panic!("{o:?}"),
    };
    match NativeSparqlEngine::new()
        .query(
            ds,
            SparqlRequest {
                query: &q,
                base_iri: None,
                substitutions: &[],
            },
        )
        .unwrap()
    {
        SparqlResult::Solutions { rows, .. } => rows.len(),
        o => panic!("{o:?}"),
    }
}

#[test]
fn broaden_reproduces_the_committed_python_output() {
    let input = read_turtle(&pack("fixtures/session-real.ttl")).unwrap();
    let (out, report) = broaden(&input, "e2e-capability-grounding").unwrap();
    let expected = read_turtle(&pack("fixtures/session-real-broad-topic.ttl")).unwrap();
    assert!(report.rewrite2 > 0, "experiment must change something");
    assert_eq!(
        canonical(&out),
        canonical(&expected),
        "Rust broadening differs from the Python fixture"
    );
    // Round-trips through Turtle unchanged.
    let again =
        graphlaw::rdf::parse_dataset(&to_turtle(&out).unwrap(), "text/turtle", None).unwrap();
    assert_eq!(canonical(&again), canonical(&out));
}

fn line(v: serde_json::Value) -> String {
    format!("{v}\n")
}

#[test]
fn transcript_capture_feeds_the_hook() {
    let mut raw = String::new();
    let human = |t: &str| {
        line(serde_json::json!({"type":"user","origin":{"kind":"human"},"message":{"content":t}}))
    };
    let assistant = |id: &str, t: &str| {
        line(
            serde_json::json!({"type":"assistant","message":{"id":id,"content":[
            {"type":"thinking","thinking":"ignored"},{"type":"text","text":t},{"type":"tool_use","name":"x"}]}}),
        )
    };
    raw += &human("does the CLI reach the swarm end-to-end?");
    raw += &assistant(
        "m1",
        "Architecture overview:\n| a | b |\n|---|---|\n| 1 | 2 |",
    );
    raw += "not json\n";
    raw += &line(
        serde_json::json!({"type":"user","origin":{"kind":"task-notification"},"message":{"content":"ignored"}}),
    );
    raw += &line(serde_json::json!({"type":"user","message":{"content":[{"type":"tool_result"}]}}));
    raw += &human("does the CLI reach the swarm end-to-end, really?");
    raw += &assistant("m2", "test result: ok. 3 passed; 0 failed");

    let turns = turns_from_jsonl(&raw, false);
    let kinds: Vec<_> = turns.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        [
            "GroundingQuestion",
            "SurveyResponse",
            "GroundingQuestion",
            "RunResponse"
        ]
    );
    assert_eq!(turns[0].topic, "cli-e2e-swarm");
    assert!(!turns.iter().any(|t| t.text.contains("ignored")));
    assert_eq!(
        turns_from_jsonl(&raw, true).len(),
        2,
        "assistant-only drops human turns"
    );

    let ds = build_dataset(&turns, "sess", "https://example.org/s").unwrap();
    assert_eq!(
        hook_rows(&ds),
        1,
        "one repeated grounding topic after a survey response"
    );
}

#[test]
fn classification_priority_and_topic_fallback() {
    use graphlaw::smon::{classify_turn, extract_topic};
    // A run wins over a survey shape in the same text.
    assert_eq!(
        classify_turn("test result: ok\n```mermaid").0,
        "RunResponse"
    );
    assert_eq!(classify_turn("blocked on approval").0, "BlockerResponse");
    assert_eq!(classify_turn("hello").0, "Other");
    assert_eq!(
        extract_topic("zebra quokka narwhal walrus"),
        "kw-zebra-quokka-narwhal"
    );
    assert_eq!(extract_topic("!!"), "unclassified-topic");
    assert_eq!(extract_topic("does the arrazzo swarm work"), "arazzo-swarm");
}
