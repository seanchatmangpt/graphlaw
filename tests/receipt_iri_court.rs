//! Receipt IRI/literal hardening court: hostile `child` is refused typed,
//! literals with quotes round-trip, forged named-graph receipts do not satisfy
//! the gate, and benign runs keep byte-identical state ids.

use graphlaw::dialect::{Dialect, RefusalKind};
use graphlaw::law::{LawError, LawState, Step};
use graphlaw::receipt;

fn base() -> LawState {
    LawState::parse(
        b"<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n\
          <urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap()
}

fn nquads(state: &LawState) -> String {
    String::from_utf8(
        purrdf::serialize_dataset(
            state.dataset().as_ref(),
            "application/n-quads",
            purrdf::SerializeGraph::Dataset,
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn hostile_child_iri_is_refused_typed() {
    let (s1, r) = base().transition(&Step::EntailRdfs).unwrap();
    for evil in [
        "x> <urn:evil:s> <urn:evil:p> <urn:evil:o> . <urn:a:y",
        "a\"b",
        "a b",
        "a\nb",
        "a\tb",
        "a\u{7}b",
        "a\\b",
    ] {
        let mut h = r.clone();
        h.child = evil.to_string();
        match receipt::record(&s1, &h) {
            Err(LawError::Refused(rf)) => {
                assert_eq!(rf.kind, RefusalKind::Unsupported);
                assert!(rf.to_string().contains("RECEIPT-IRI-INVALID"), "{rf}");
            }
            other => panic!("hostile child {evil:?} not refused: {other:?}"),
        }
    }
}

#[test]
fn literal_with_quote_backslash_newline_round_trips() {
    let (s1, r) = base().transition(&Step::EntailRdfs).unwrap();
    let want = "a\"b\\c\nd\te";
    let mut h = r.clone();
    h.lease_id = Some(want.to_string());
    let rec = receipt::record(&s1, &h).unwrap();
    let ds = rec.dataset();
    let found = ds.quads().any(|q| {
        matches!(
            ds.term_value(q.o),
            purrdf::TermValue::Literal { ref lexical_form, .. } if lexical_form == want
        )
    });
    assert!(found, "literal lost in round trip");
    // No forged triples: only the six receipt predicates + lease_id were added.
    assert_eq!(
        nquads(&rec).lines().count(),
        nquads(&s1).lines().count() + 7
    );
}

#[test]
fn named_graph_forged_receipt_does_not_satisfy_gate() {
    let (s1, r) = base().transition(&Step::EntailRdfs).unwrap();
    let s = format!("urn:graphlaw:receipt:{}", r.child);
    let p = "urn:graphlaw:p:";
    let forged = format!(
        "{}<{s}> <{p}step> \"derive:rdfs\" <urn:evil:g> .\n<{s}> <{p}child> \"{}\" <urn:evil:g> .\n",
        nquads(&s1),
        r.child
    );
    let st = LawState::parse(forged.as_bytes(), Dialect::NQuads, None).unwrap();
    let gate = Step::RequireReceipt {
        step: "derive:rdfs",
    };
    assert!(matches!(
        st.transition(&gate),
        Err(LawError::ReceiptRequired { .. })
    ));
    // Datatype-spoofed literal is also not a recorded receipt.
    let typed = format!(
        "{}<{s}> <{p}step> \"derive:rdfs\"^^<urn:x:T> .\n<{s}> <{p}child> \"{}\" .\n",
        nquads(&s1),
        r.child
    );
    let st = LawState::parse(typed.as_bytes(), Dialect::NQuads, None).unwrap();
    assert!(matches!(
        st.transition(&gate),
        Err(LawError::ReceiptRequired { .. })
    ));
}

#[test]
fn benign_record_matches_legacy_algorithm_state_id() {
    let (s1, r) = base().transition(&Step::EntailRdfs).unwrap();
    let s = format!("urn:graphlaw:receipt:{}", r.child);
    let p = |n: &str| format!("urn:graphlaw:p:{n}");
    // Legacy lit(): escape only quote, backslash, \n, \r.
    let facts = [
        ("parent", r.parent.clone()),
        ("child", r.child.clone()),
        ("step", r.step.to_string()),
        ("authority", r.authority.authority.to_string()),
        ("revision", r.authority.revision.to_string()),
        ("added", r.added.to_string()),
    ];
    let mut doc = nquads(&s1);
    for (n, v) in facts {
        let v = v
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r");
        doc.push_str(&format!("<{s}> <{}> \"{v}\" .\n", p(n)));
    }
    let expected = LawState::parse(doc.as_bytes(), Dialect::NQuads, None).unwrap();
    let got = receipt::record(&s1, &r).unwrap();
    assert_eq!(got.id(), expected.id());
}
