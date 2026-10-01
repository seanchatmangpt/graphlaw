//! API stability falsifier: this file is a *downstream* crate, so it can only
//! compile if the public enums are `#[non_exhaustive]`-safe, that is, matchable
//! with a wildcard arm. A future variant must never break this file.
use graphlaw::attest::AttestError;
use graphlaw::dialect::{Dialect, Engine, RefusalKind};
use graphlaw::law::{Ceiling, LawError, LeaseReason, ReceiptReason, Step};
use graphlaw::policy::PolicyRefusalKind;
use graphlaw::receipt_store::StoreError;

#[test]
fn public_enums_match_with_wildcard_arms() {
    let d = match Dialect::N3 {
        Dialect::N3 => "n3",
        _ => "other",
    };
    let e = match Dialect::N3.engine() {
        Engine::Eyeron => "eyeron",
        _ => "other",
    };
    let k = match RefusalKind::Ambiguous {
        RefusalKind::Ambiguous => "ambiguous",
        _ => "other",
    };
    let c = match Ceiling::Observe {
        Ceiling::Observe => "observe",
        _ => "other",
    };
    let l = match LeaseReason::Expired {
        LeaseReason::Expired => "expired",
        _ => "other",
    };
    let r = match ReceiptReason::Unattested {
        ReceiptReason::Unattested => "unattested",
        _ => "other",
    };
    let p = match PolicyRefusalKind::BadMass {
        PolicyRefusalKind::BadMass => "bad_mass",
        _ => "other",
    };
    let s = match Step::EntailOwlRl {
        Step::EntailOwlRl => "owl",
        _ => "other",
    };
    let a = match AttestError::BadSignature {
        AttestError::BadSignature => "bad",
        _ => "other",
    };
    let st = match StoreError::Io(String::new()) {
        StoreError::Io(_) => "io",
        _ => "other",
    };
    let le = match (LawError::ReceiptRequired { step: "x".into() }) {
        LawError::ReceiptRequired { .. } => "req",
        _ => "other",
    };
    assert_eq!(
        [d, e, k, c, l, r, p, s, a, st, le],
        [
            "n3",
            "eyeron",
            "ambiguous",
            "observe",
            "expired",
            "unattested",
            "bad_mass",
            "owl",
            "bad",
            "io",
            "req"
        ]
    );
}

#[test]
fn abi_refusal_codes_are_stable_strings() {
    assert_eq!(LeaseReason::Expired.as_str(), "expired");
    assert_eq!(ReceiptReason::UntrustedKey.as_str(), "untrusted_key");
    assert_eq!(PolicyRefusalKind::DeadEnd.as_str(), "DeadEnd");
    assert_eq!(Ceiling::Construct.name(), "construct");
}
