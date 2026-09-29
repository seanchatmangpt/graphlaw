//! Authority leases (R_missing_authority): real states, real transitions.

use graphlaw::dialect::Dialect;
use graphlaw::law::{Ceiling, LawError, LawState, Lease, LeaseReason, Step};
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

fn lease(ceiling: Ceiling, scope: &[&str], expires: u64) -> Lease {
    Lease {
        id: "lease-1".into(),
        holder: "agent-7".into(),
        ceiling,
        scope: scope.iter().map(|s| s.to_string()).collect(),
        expires_unix: expires,
        issued_unix: 0,
    }
}

fn reason(r: Result<(LawState, graphlaw::law::Receipt), LawError>) -> LeaseReason {
    match r {
        Err(LawError::LeaseRefused { reason, .. }) => reason,
        other => panic!("expected LeaseRefused, got {other:?}"),
    }
}

#[test]
fn valid_lease_admits_and_receipt_carries_lease_id() {
    let l = lease(Ceiling::Construct, &["derive:rdfs"], 100);
    let (child, r) = base()
        .transition_leased_unverified(&l, &Step::EntailRdfs, 50)
        .unwrap();
    assert_eq!(r.lease_id.as_deref(), Some("lease-1"));
    assert!(child.quad_count() > base().quad_count());
    // The lease id is recorded as a triple and feeds back into the state.
    let recorded = receipt::record(&child, &r).unwrap();
    let nq = String::from_utf8(
        purrdf::serialize_dataset(
            recorded.dataset().as_ref(),
            "application/n-quads",
            purrdf::SerializeGraph::Dataset,
        )
        .unwrap(),
    )
    .unwrap();
    assert!(nq.contains("urn:graphlaw:p:lease_id"), "{nq}");
    assert!(nq.contains("\"lease-1\""), "{nq}");
}

#[test]
fn unleased_transition_has_no_lease_id() {
    let (_, r) = base().transition(&Step::EntailRdfs).unwrap();
    assert_eq!(r.lease_id, None);
}

#[test]
fn expired_lease_refused() {
    let l = lease(Ceiling::Construct, &["derive:rdfs"], 100);
    assert_eq!(
        reason(base().transition_leased_unverified(&l, &Step::EntailRdfs, 100)),
        LeaseReason::Expired
    );
}

#[test]
fn out_of_scope_step_refused() {
    let l = lease(Ceiling::Construct, &["derive:owl-rl"], 100);
    assert_eq!(
        reason(base().transition_leased_unverified(&l, &Step::EntailRdfs, 1)),
        LeaseReason::OutOfScope
    );
}

#[test]
fn ceiling_below_required_refused() {
    let l = lease(Ceiling::Select, &["derive:rdfs"], 100);
    assert_eq!(
        reason(base().transition_leased_unverified(&l, &Step::EntailRdfs, 1)),
        LeaseReason::Ceiling
    );
    // An observe-only lease still runs a gate.
    let g = lease(Ceiling::Observe, &["admit:require-receipt"], 100);
    let gate = Step::RequireReceipt { step: "x" };
    assert!(matches!(
        base().transition_leased_unverified(&g, &gate, 1),
        Err(LawError::ReceiptRequired { .. })
    ));
}

#[test]
fn refused_lease_yields_no_state() {
    let l = lease(Ceiling::Observe, &[], 100);
    assert!(
        base()
            .transition_leased_unverified(&l, &Step::EntailRdfs, 1)
            .is_err()
    );
}
