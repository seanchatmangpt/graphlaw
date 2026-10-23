//! TC-MEGA-FIBO-01 WO-14 — anti-vacuity court for the #41209 falsifier twins.
//!
//! Drives the REAL admission path (`LawState::parse` + `Step::AdmitShacl`
//! against the generated precision shapes) end to end:
//! P4.1 benign twin admits with an asserted receipt,
//! P4.2 the twins diverge at exactly ONE byte offset (0-based 532; the tail
//!      shifts by the +2-byte insertion, per ontology/README.md),
//! P4.3 the mutated twin refuses with exactly one typed violation at
//!      reg:Leg-41209 / reg:amount,
//! P4.4 zero state mutation: refuse-then-re-admit yields an identical
//!      receipt (no cache/state pollution from the refusal),
//! P4.5 constraint-causality flip: a single-byte in-pattern flip admits and
//!      a scale violation refuses — the constraint causes the refusal.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawError, LawState, Receipt, Step};
use graphlaw::receipt_store::receipt_digest;

const MEGA: &str = "docs/jira/v26.10.22/mega/tc-mega-fibo-01";

fn read(p: &str) -> String {
    std::fs::read_to_string(format!("{MEGA}/{p}")).expect("mega fixture")
}

/// One admission attempt through the real gate: parse the twin as Turtle,
/// then SHACL-validate it against the #41209 anchor shape
/// (51_precision_anchor_41209.ttl: reg:MoneyPrecision_USD_41209). One state
/// machine, one gate, no mocks. The parametric family (shape 50) is asserted
/// separately in p43 so the single-violation invariant stays exact.
fn admit(twin_ttl: &str) -> Result<(LawState, Receipt), LawError> {
    let state = LawState::parse(twin_ttl.as_bytes(), Dialect::Turtle, None)
        .expect("twin parses as Turtle");
    let shapes = read("ontology/shapes/51_precision_anchor_41209.ttl");
    state.transition(&Step::AdmitShacl { shapes_ttl: &shapes })
}

fn digest(r: &Receipt) -> String {
    receipt_digest(r)
}

/// P4.1 — the benign twin (1500.00) admits; the receipt is asserted.
#[test]
fn p41_benign_twin_admits_with_receipt() {
    let (state, r) = admit(&read("ontology/twins/41209_benign.ttl")).expect("benign admits");
    assert_eq!(r.step, "admit:shacl");
    assert_eq!(r.added, 0, "admission gate adds no quads");
    assert_eq!(r.parent, state.id());
    assert_eq!(r.child, state.id());
    let d = digest(&r);
    assert_eq!(d.len(), 64, "BLAKE3 hex digest length");
    assert!(d.chars().all(|c| c.is_ascii_hexdigit()), "digest hex: {d}");
}

/// P4.2 — the twins diverge at exactly ONE byte offset (0-based 532, the
/// first character of the decimal literal), +2 bytes of tail shift.
#[test]
fn p42_twins_diverge_at_exactly_one_byte_offset() {
    let b = read("ontology/twins/41209_benign.ttl");
    let m = read("ontology/twins/41209_mutated.ttl");
    let bb = b.as_bytes();
    let mb = m.as_bytes();
    let first_diff = bb
        .iter()
        .zip(mb)
        .position(|(x, y)| x != y)
        .expect("twins are not byte-identical");
    assert_eq!(first_diff, 532, "divergence begins at 0-based byte 532");
    assert_eq!(mb.len() - bb.len(), 2, "mutation inserts exactly 2 bytes");
    // The whole file is exactly the single-edit mutation the generator asserts.
    assert_eq!(
        m,
        b.replace("1500.00", "1500.0025"),
        "mutated == benign with the single literal edit"
    );
}

/// P4.3 — the mutated twin (1500.0025) refuses with exactly one typed
/// violation: focus reg:Leg-41209, path reg:amount, sh:pattern component,
/// ISO 4217 message, sh:Violation severity.
#[test]
fn p43_mutated_twin_refuses_with_one_typed_violation() {
    let err = admit(&read("ontology/twins/41209_mutated.ttl")).unwrap_err();
    let LawError::NotAdmitted { violations, results } = err else {
        panic!("expected NotAdmitted, got {err:?}");
    };
    assert_eq!(violations, 1, "exactly one violating constraint");
    let v = &results[0];
    assert!(
        v.focus.trim_end_matches('>').ends_with("Leg-41209"),
        "focus: {}",
        v.focus
    );
    assert!(
        v.path
            .as_deref()
            .is_some_and(|p| p.trim_end_matches('>').ends_with("amount")),
        "path: {:?}",
        v.path
    );
    assert!(
        v.component.contains("PatternConstraintComponent"),
        "component: {}",
        v.component
    );
    assert!(v.message.contains("4217"), "message: {}", v.message);
    assert_eq!(v.severity, "Violation");

    // The parametric per-currency family (shape 50, MoneyPrecision_USD)
    // independently refuses the same focus node with the same component:
    // the twin cannot slip past the family by outrunning the anchor.
    let state = LawState::parse(
        read("ontology/twins/41209_mutated.ttl").as_bytes(),
        Dialect::Turtle,
        None,
    )
    .unwrap();
    let family = read("ontology/shapes/50_currency_precision.ttl");
    let LawError::NotAdmitted { violations: fv, results: fres } =
        state.transition(&Step::AdmitShacl { shapes_ttl: &family }).unwrap_err()
    else {
        panic!("family shape must also refuse the mutated twin");
    };
    assert_eq!(fv, 1);
    assert!(fres[0].component.contains("PatternConstraintComponent"));
    assert!(fres[0].focus.trim_end_matches('>').ends_with("Leg-41209"));
}

/// P4.4 — zero state mutation: admit benign, refuse mutated, re-admit benign.
/// The second benign receipt is IDENTICAL to the first: the refusal
/// polluted nothing.
#[test]
fn p44_zero_state_mutation_across_refusal() {
    let benign = read("ontology/twins/41209_benign.ttl");
    let mutated = read("ontology/twins/41209_mutated.ttl");

    let (s1, r1) = admit(&benign).expect("first benign admission");
    // A vacuous pass here would make the identity assertion meaningless, so
    // the refusal is itself asserted before identity is checked.
    assert!(admit(&mutated).is_err(), "mutated twin must refuse");
    let (s2, r2) = admit(&benign).expect("second benign admission");

    assert_eq!(s1.id(), s2.id(), "state id identical after the refusal");
    assert_eq!(r1, r2, "receipts equal field-for-field");
    assert_eq!(digest(&r1), digest(&r2), "receipt digests identical");
}

/// P4.5 — constraint causality: within the same lexical family, a single-byte
/// flip that STAYS inside the sh:pattern admits; a 3-decimal lexical form
/// built from that same byte region refuses. The constraint causes the
/// refusal, not the literal's identity.
#[test]
fn p45_constraint_causality_flip() {
    let mut bytes =
        std::fs::read(format!("{MEGA}/ontology/twins/41209_benign.ttl")).expect("benign bytes");
    // Benign literal layout: 0-based 524 = opening quote, 525..=531 =
    // "1500.00" (dot at 529), 532 = closing quote. The mutation diverges at
    // 532 because the mutated literal is longer, not because its bytes move.
    assert_eq!(bytes[524], b'"', "byte 524 opens the decimal literal");
    assert_eq!(&bytes[525..532], b"1500.00");
    // Single-byte flip inside the pattern: 1500.00 -> 1500.09 must admit.
    bytes[531] = b'9'; // last scale digit
    let in_pattern = String::from_utf8(bytes).expect("utf-8");
    assert!(
        admit(&in_pattern).is_ok(),
        "1500.09 is inside the sh:pattern and must admit"
    );
    // Extend past 2 decimals from that same literal: 1500.09 -> 1500.0925 refuses.
    let out_of_pattern = in_pattern.replace("1500.09", "1500.0925");
    assert!(
        admit(&out_of_pattern).is_err(),
        "a 4-decimal lexical form must refuse"
    );
}
