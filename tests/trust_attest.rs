//! Signed receipts and leases: real Ed25519 keys, real states, real transitions.
//! No test doubles; `FixedClock` is a real `Clock` implementation with a pinned instant.

use graphlaw::attest::{
    self, AttestError, Attestation, SigningKey, TrustedKeys, hex_decode, hex_encode,
};
use graphlaw::dialect::Dialect;
use graphlaw::law::{
    Ceiling, FixedClock, LawError, LawState, Lease, LeaseReason, Receipt, ReceiptReason, Step,
    SystemClock,
};
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

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn trusting(k: &SigningKey) -> TrustedKeys {
    let mut t = TrustedKeys::new();
    t.insert(k.verifying_key());
    t
}

fn real_receipt() -> (LawState, Receipt) {
    base().transition(&Step::EntailRdfs).unwrap()
}

fn lease(expires: u64, issued: u64) -> Lease {
    Lease {
        id: "lease-1".into(),
        holder: "agent-7".into(),
        ceiling: Ceiling::Construct,
        scope: vec!["derive:rdfs".into()],
        expires_unix: expires,
        issued_unix: issued,
    }
}

fn lease_reason(r: Result<(LawState, Receipt), LawError>) -> LeaseReason {
    match r {
        Err(LawError::LeaseRefused { reason, .. }) => reason,
        other => panic!("expected LeaseRefused, got {other:?}"),
    }
}

fn receipt_reason(r: Result<LawState, LawError>) -> ReceiptReason {
    match r {
        Err(LawError::ReceiptRefused { reason, .. }) => reason,
        other => panic!("expected ReceiptRefused, got {other:?}"),
    }
}

#[test]
fn a_hand_written_receipt_triples_pass_require_but_not_require_signed() {
    let forged = LawState::parse(
        b"<urn:graphlaw:receipt:sha256:ff> <urn:graphlaw:p:step> \"admit:shacl\" .\n\
          <urn:graphlaw:receipt:sha256:ff> <urn:graphlaw:p:child> \"sha256:ff\" .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    // The old gate proves presence only: forged triples satisfy it.
    receipt::require(&forged, "admit:shacl").unwrap();
    // The signed gate refuses them.
    let trusted = trusting(&key(1));
    assert_eq!(
        receipt_reason(
            receipt::require_signed(&forged, "admit:shacl", &trusted).map(|()| forged.clone())
        ),
        ReceiptReason::Unattested
    );
    // A step with no receipt at all keeps the ReceiptRequired refusal.
    assert!(matches!(
        receipt::require_signed(&forged, "derive:n3", &trusted),
        Err(LawError::ReceiptRequired { .. })
    ));
}

#[test]
fn a_signed_receipt_recorded_in_state_satisfies_the_signed_gate() {
    let (child, r) = real_receipt();
    let k = key(1);
    let att = attest::sign_receipt(&k, &r);
    let recorded = receipt::record_signed(&child, &r, &att).unwrap();
    let trusted = trusting(&k);
    receipt::require_signed(&recorded, "derive:rdfs", &trusted).unwrap();
    // The same gate as a transition step.
    let gate = Step::RequireSignedReceipt {
        step: "derive:rdfs",
        trusted: &trusted,
    };
    let (_, gr) = recorded.transition(&gate).unwrap();
    assert_eq!(gr.step, "admit:require-signed-receipt");
    // A merely recorded (unattested) receipt is refused by the signed gate.
    let plain = receipt::record(&child, &r).unwrap();
    assert_eq!(
        receipt_reason(
            receipt::require_signed(&plain, "derive:rdfs", &trusted).map(|()| plain.clone())
        ),
        ReceiptReason::Unattested
    );
}

#[test]
fn c_receipt_signed_by_an_untrusted_key_is_refused() {
    let (child, r) = real_receipt();
    let mallory = key(9);
    let recorded = receipt::record_signed(&child, &r, &attest::sign_receipt(&mallory, &r)).unwrap();
    let trusted = trusting(&key(1));
    assert_eq!(
        receipt_reason(
            receipt::require_signed(&recorded, "derive:rdfs", &trusted).map(|()| recorded.clone())
        ),
        ReceiptReason::UntrustedKey
    );
}

#[test]
fn d_tampering_a_signed_receipt_payload_fails_verification() {
    let (child, r) = real_receipt();
    let k = key(1);
    let trusted = trusting(&k);
    let att = attest::sign_receipt(&k, &r);
    attest::verify_receipt(&r, &att, &trusted).unwrap();

    // every signed field, mutated after signing, is refused
    let mut m = r.clone();
    m.added += 1;
    assert_eq!(
        attest::verify_receipt(&m, &att, &trusted),
        Err(AttestError::BadSignature)
    );
    let mut m = r.clone();
    m.parent.push('0');
    assert_eq!(
        attest::verify_receipt(&m, &att, &trusted),
        Err(AttestError::BadSignature)
    );
    let m = r.clone().with_subject("sha256:aa");
    assert_eq!(
        attest::verify_receipt(&m, &att, &trusted),
        Err(AttestError::BadSignature)
    );
    let mut m = r.clone();
    m.lease_id = Some("other".into());
    assert_eq!(
        attest::verify_receipt(&m, &att, &trusted),
        Err(AttestError::BadSignature)
    );

    // one flipped signature byte
    let mut sig = hex_decode(&att.signature).unwrap();
    sig[0] ^= 1;
    let bad = Attestation {
        signature: hex_encode(&sig),
        ..att.clone()
    };
    assert_eq!(
        attest::verify_receipt(&r, &bad, &trusted),
        Err(AttestError::BadSignature)
    );

    // an attestation copied onto a receipt with different recorded fields: the
    // state-level gate refuses it too.
    let mut edited = r.clone();
    edited.added += 5;
    let recorded = receipt::record_signed(&child, &edited, &att).unwrap();
    assert_eq!(
        receipt_reason(
            receipt::require_signed(&recorded, "derive:rdfs", &trusted).map(|()| recorded.clone())
        ),
        ReceiptReason::BadSignature
    );
}

#[test]
fn subject_digest_is_part_of_the_signed_payload_and_survives_the_state() {
    let (child, r) = real_receipt();
    let r = r.with_subject("sha256:0123");
    let k = key(1);
    let recorded = receipt::record_signed(&child, &r, &attest::sign_receipt(&k, &r)).unwrap();
    receipt::require_signed(&recorded, "derive:rdfs", &trusting(&k)).unwrap();
}

#[test]
fn b_expired_signed_lease_is_refused_and_no_caller_field_can_revive_it() {
    let issuer = key(2);
    let trusted = trusting(&issuer);
    let signed = attest::sign_lease(&issuer, lease(100, 0));
    let step = Step::EntailRdfs;

    // Verifier's clock says 200: expired. There is no `now` parameter to pass.
    assert_eq!(
        lease_reason(base().transition_authorized(&signed, &trusted, &FixedClock(200), 60, &step)),
        LeaseReason::Expired
    );
    // Boundary: expiry is `now >= expires`.
    assert_eq!(
        lease_reason(base().transition_authorized(&signed, &trusted, &FixedClock(100), 60, &step)),
        LeaseReason::Expired
    );
    // The same lease is valid when the *verifier's* clock is earlier: time
    // comes from the Clock the verifier supplies, and only from there.
    let (child, r) = base()
        .transition_authorized(&signed, &trusted, &FixedClock(99), 60, &step)
        .unwrap();
    assert_eq!(r.lease_id.as_deref(), Some("lease-1"));
    assert!(child.quad_count() > base().quad_count());
    // The system clock is far past 100.
    assert_eq!(
        lease_reason(base().transition_authorized(&signed, &trusted, &SystemClock, 60, &step)),
        LeaseReason::Expired
    );
}

#[test]
fn c_lease_signed_by_untrusted_key_or_edited_after_signing_is_refused() {
    let issuer = key(2);
    let trusted = trusting(&issuer);
    let step = Step::EntailRdfs;
    let clock = FixedClock(1);

    let forged = attest::sign_lease(&key(8), lease(100, 0));
    assert_eq!(
        lease_reason(base().transition_authorized(&forged, &trusted, &clock, 60, &step)),
        LeaseReason::UntrustedKey
    );

    let mut widened = attest::sign_lease(&issuer, lease(100, 0));
    widened.lease.expires_unix = u64::MAX;
    assert_eq!(
        lease_reason(base().transition_authorized(&widened, &trusted, &clock, 60, &step)),
        LeaseReason::BadSignature
    );
    let mut wider_scope = attest::sign_lease(&issuer, lease(100, 0));
    wider_scope.lease.scope.push("derive:n3".into());
    assert_eq!(
        lease_reason(base().transition_authorized(&wider_scope, &trusted, &clock, 60, &step)),
        LeaseReason::BadSignature
    );
}

#[test]
fn issuer_clock_lead_beyond_the_skew_bound_is_refused() {
    let issuer = key(2);
    let trusted = trusting(&issuer);
    let signed = attest::sign_lease(&issuer, lease(10_000, 1_000));
    let step = Step::EntailRdfs;
    // verifier at 100, issuer claims 1000: 1000 > 100 + 60
    assert_eq!(
        lease_reason(base().transition_authorized(&signed, &trusted, &FixedClock(100), 60, &step)),
        LeaseReason::ClockSkew
    );
    // exactly at the bound is tolerated: 1000 <= 100 + 900
    base()
        .transition_authorized(&signed, &trusted, &FixedClock(100), 900, &step)
        .unwrap();
}

#[test]
fn scope_and_ceiling_still_bind_a_verified_lease() {
    let issuer = key(2);
    let trusted = trusting(&issuer);
    let mut l = lease(100, 0);
    l.scope = vec!["derive:owl-rl".into()];
    let signed = attest::sign_lease(&issuer, l);
    assert_eq!(
        lease_reason(base().transition_authorized(
            &signed,
            &trusted,
            &FixedClock(1),
            60,
            &Step::EntailRdfs
        )),
        LeaseReason::OutOfScope
    );
    let mut l = lease(100, 0);
    l.ceiling = Ceiling::Select;
    let signed = attest::sign_lease(&issuer, l);
    assert_eq!(
        lease_reason(base().transition_authorized(
            &signed,
            &trusted,
            &FixedClock(1),
            60,
            &Step::EntailRdfs
        )),
        LeaseReason::Ceiling
    );
}

#[test]
fn signing_is_deterministic_and_attestation_json_round_trips() {
    let (_, r) = real_receipt();
    let k = key(1);
    let a1 = attest::sign_receipt(&k, &r);
    assert_eq!(a1, attest::sign_receipt(&k, &r));
    assert_eq!(Attestation::from_json(&a1.to_json()).unwrap(), a1);
    assert!(matches!(
        Attestation::from_json("{\"key_id\":\"zz\"}"),
        Err(AttestError::Malformed(_))
    ));
    assert_eq!(a1.key_id, k.verifying_key().key_id());
}
