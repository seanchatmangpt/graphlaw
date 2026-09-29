//! Typed builders: same plan and digest as hand-written N-Triples, and no
//! injection through literals or IRIs.
use graphlaw::dialect::Dialect;
use graphlaw::law::LawState;
use graphlaw::plan::{Action, Plan, Triple, TripleError};

const DOOR: &str = "<urn:d:door> <urn:p:is>";

fn t(o: &str) -> Triple {
    Triple::iri("urn:d:door", "urn:p:is", o).unwrap()
}

fn start() -> LawState {
    LawState::parse(
        format!("{DOOR} <urn:v:closed> .\n").as_bytes(),
        Dialect::NTriples,
        None,
    )
    .unwrap()
}

#[test]
fn builder_matches_hand_written_plan_and_digest() {
    let built = Plan::builder()
        .action("open")
        .requires(t("urn:v:closed"))
        .adds(t("urn:v:open"))
        .deletes(t("urn:v:closed"))
        .action("lock")
        .requires(t("urn:v:open"))
        .adds(t("urn:v:locked"))
        .deletes(t("urn:v:open"))
        .goal(t("urn:v:locked"))
        .build()
        .unwrap();
    let by_hand = Plan {
        goal: format!("{DOOR} <urn:v:locked> ."),
        actions: vec![
            Action {
                name: "open".into(),
                pre: format!("{DOOR} <urn:v:closed> ."),
                add: format!("{DOOR} <urn:v:open> ."),
                del: format!("{DOOR} <urn:v:closed> ."),
                ..Default::default()
            },
            Action {
                name: "lock".into(),
                pre: format!("{DOOR} <urn:v:open> ."),
                add: format!("{DOOR} <urn:v:locked> ."),
                del: format!("{DOOR} <urn:v:open> ."),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert_eq!(built.digest(), by_hand.digest());
    assert_eq!(built.canonical_json(), by_hand.canonical_json());
    let a = built.admit(&start()).unwrap();
    let b = by_hand.admit(&start()).unwrap();
    assert_eq!(a.state.id(), b.state.id());
    assert_eq!(a.plan_digest, b.plan_digest);
}

#[test]
fn multiple_triples_join_like_hand_written_lines() {
    let built = Plan::builder()
        .action("a")
        .requires(t("urn:v:1"))
        .requires(t("urn:v:2"))
        .build()
        .unwrap();
    assert_eq!(
        built.actions[0].pre,
        format!("{DOOR} <urn:v:1> .\n{DOOR} <urn:v:2> .")
    );
}

#[test]
fn literal_with_quote_and_newline_round_trips() {
    let text = "say \"hi\"\nback\\slash\ttab";
    let lit = Triple::literal("urn:d:door", "urn:p:note", text).unwrap();
    let plan = Plan::builder()
        .action("note")
        .adds(lit.clone())
        .goal(lit)
        .build()
        .unwrap();
    let admitted = plan.admit(&start()).unwrap();
    // exactly the original triple plus the added one
    assert_eq!(admitted.state.quad_count(), 2);
}

#[test]
fn injection_attempt_stays_one_literal() {
    let evil = "x\" . <urn:evil> <urn:p> <urn:o>";
    let lit = Triple::literal("urn:d:door", "urn:p:note", evil).unwrap();
    let plan = Plan::builder()
        .action("note")
        .adds(lit.clone())
        .goal(lit)
        .build()
        .unwrap();
    let admitted = plan.admit(&start()).unwrap();
    // one original triple + ONE literal triple; no extra `urn:evil` triple
    assert_eq!(admitted.state.quad_count(), 2);
    let nq = purrdf::serialize_dataset(
        admitted.state.dataset(),
        "application/n-quads",
        purrdf::SerializeGraph::Dataset,
    )
    .unwrap();
    let nq = String::from_utf8(nq).unwrap();
    assert!(
        !nq.lines().any(|l| l.starts_with("<urn:evil>")),
        "injected triple appeared: {nq}"
    );
}

#[test]
fn hostile_iris_are_refused_with_a_typed_error() {
    for bad in [
        "urn:a>b",
        "urn:a<b",
        "urn:a b",
        "urn:a\nb",
        "urn:a\"b",
        "urn:a\\b",
        "",
        "urn:a\u{7f}",
    ] {
        match Triple::iri(bad, "urn:p", "urn:o") {
            Err(TripleError::InvalidIri { role, iri }) => {
                assert_eq!(role, "subject");
                assert_eq!(iri, bad);
            }
            other => panic!("{bad:?} not refused: {other:?}"),
        }
        assert!(Triple::iri("urn:s", bad, "urn:o").is_err());
        assert!(Triple::iri("urn:s", "urn:p", bad).is_err());
        assert!(Triple::literal(bad, "urn:p", "x").is_err());
    }
}

#[test]
fn builder_misuse_is_refused() {
    let e = Plan::builder().requires(t("urn:v:x")).build().unwrap_err();
    assert!(matches!(e, TripleError::Builder(_)));
    let e = Plan::builder().action("  ").build().unwrap_err();
    assert!(matches!(e, TripleError::Builder(_)));
}
