//! Receipt fields with escaped literal content (quote, backslash, newline,
//! unicode) round-trip exactly through `record_signed` / `require_signed`.

use graphlaw::attest::{SigningKey, TrustedKeys, sign_receipt};
use graphlaw::dialect::Dialect;
use graphlaw::law::{LawState, Receipt, ReceiptReason, Step, LawError};
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

fn nasty(r: Receipt, s: &str) -> Receipt {
    Receipt {
        lease_id: Some(format!("lease\"{s}\\end")),
        plan_blake3: Some(format!("plan\\\"{s}\nline2\r\nline3")),
        subject_blake3: Some(format!("sub \"q\" \\\\ \u{e9}\u{4e2d}\u{1f600} {s}")),
        ..r
    }
}

#[test]
fn escaped_literals_round_trip_signed() {
    let (_, r) = base().transition(&Step::EntailRdfs).unwrap();
    let r = nasty(r, "a");
    let key = SigningKey::from_seed([9; 32]);
    let att = sign_receipt(&key, &r);
    let mut trusted = TrustedKeys::new();
    trusted.insert(key.verifying_key());
    let st = receipt::record_signed(&base(), &r, &att).unwrap();
    receipt::require_signed(&st, r.step, &trusted).unwrap();
    receipt::require(&st, r.step).unwrap();
}

#[test]
fn edited_escaped_field_is_refused() {
    let (_, r) = base().transition(&Step::EntailRdfs).unwrap();
    let signed = nasty(r.clone(), "a");
    let key = SigningKey::from_seed([9; 32]);
    let att = sign_receipt(&key, &signed);
    let mut trusted = TrustedKeys::new();
    trusted.insert(key.verifying_key());
    // Recorded with a different escaped field than was signed.
    let tampered = nasty(r, "b");
    let st = receipt::record_signed(&base(), &tampered, &att).unwrap();
    match receipt::require_signed(&st, tampered.step, &trusted) {
        Err(LawError::ReceiptRefused { reason, .. }) => {
            assert_eq!(reason, ReceiptReason::BadSignature)
        }
        other => panic!("expected BadSignature, got {other:?}"),
    }
}
