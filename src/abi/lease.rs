//! Lease admission: **Phi_lease => Valid before dispatch**.
//!
//! A `law` request's lease is validated exactly once, up front, into an
//! opaque [`LeaseContext`]; the step loop then consumes the proof and runs a
//! pure scope/ceiling predicate only. No signature check, no clock read and
//! no expiry judgement happens inside the step loop, so an invalid lease can
//! never pass step N-1 and refuse at step N.
//!
//! Lease policy (explicit, typed): [`LeasePolicy::NoneAllowed`] (the default,
//! preserving every lease-less rdfs/shacl/n3 chain) lets a request carry no
//! lease; [`LeasePolicy::Required`] refuses a lease-less request with
//! `LeaseRequired` before any step runs. A host sets the process-wide policy
//! with [`set_lease_policy`]; a request may only tighten it with
//! `"lease_policy": "required"` (it can never loosen a `Required` host).

use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Value, json};

use super::ops::governance::{lease_obj, trusted_keys_of};
use super::{Fail, Res, bad, bad_refusal, str_field};
use crate::attest::Attestation;
use crate::law::{
    Ceiling, Clock, DEFAULT_MAX_SKEW_SECS, LawError, Lease, LeaseReason, SignedLease,
};

/// Whether a `law` request may run without any lease.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeasePolicy {
    /// A request without a lease runs unleased (compatibility default).
    NoneAllowed,
    /// A request without a lease is refused (`LeaseRequired`) before any step.
    Required,
}

static LEASE_REQUIRED: AtomicBool = AtomicBool::new(false);

/// Set the process-wide lease policy (host-owned; default `NoneAllowed`).
pub fn set_lease_policy(policy: LeasePolicy) {
    LEASE_REQUIRED.store(policy == LeasePolicy::Required, Ordering::SeqCst);
}

/// The current process-wide lease policy.
pub fn lease_policy() -> LeasePolicy {
    if LEASE_REQUIRED.load(Ordering::SeqCst) {
        LeasePolicy::Required
    } else {
        LeasePolicy::NoneAllowed
    }
}

/// Process policy tightened (never loosened) by the request's `lease_policy`.
fn effective_policy(v: &Value) -> Res<LeasePolicy> {
    let host = lease_policy();
    match v.get("lease_policy") {
        None | Some(Value::Null) => Ok(host),
        Some(Value::String(s)) if s == "none-allowed" => Ok(host),
        Some(Value::String(s)) if s == "required" => Ok(LeasePolicy::Required),
        Some(_) => Err(bad("`lease_policy` must be \"none-allowed\" or \"required\"")),
    }
}

/// How the lease in a [`LeaseContext`] was validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::abi) enum LeaseContextKind {
    /// No lease supplied and policy is `NoneAllowed`: authorization is a no-op.
    None,
    /// Ed25519 attestation verified; skew and expiry judged once against the
    /// module clock at admission.
    SignedVerified,
    /// Unsigned (`unverified_lease: true`), pinned to the caller's clock.
    /// Proves nothing about authority; a typed, loud downgrade.
    UnverifiedPinned { now_unix: u64 },
}

/// Validated execution proof. Constructed only by [`LeaseContext::admit`].
#[derive(Debug, Clone)]
pub(in crate::abi) struct LeaseContext {
    lease: Option<Lease>,
    kind: LeaseContextKind,
}

fn refuse(lease: &Lease, step: &str, reason: LeaseReason) -> LawError {
    LawError::LeaseRefused {
        lease_id: lease.id.clone(),
        step: step.to_string(),
        reason,
    }
}

impl LeaseContext {
    /// Total function wire -> proof. Signature, key trust, issuer skew and
    /// expiry are decided HERE, once, at `clock`; `first_step` only labels a
    /// refusal. Returns `Ok` only if the lease is valid for the whole request.
    pub(in crate::abi) fn admit(
        v: &Value,
        clock: &dyn Clock,
        first_step: &str,
    ) -> Result<LeaseContext, Fail> {
        let policy = effective_policy(v)?;
        match (v.get("signed_lease"), v.get("lease")) {
            (None, None) => {
                if policy == LeasePolicy::Required {
                    return Err(Fail {
                        refusal: bad_refusal(
                            "a lease is required: supply `signed_lease` + `trusted_keys` \
                             (lease policy `required`)",
                        ),
                        details: Some(json!({"code": "LeaseRequired"})),
                    });
                }
                Ok(LeaseContext {
                    lease: None,
                    kind: LeaseContextKind::None,
                })
            }
            (Some(_), Some(_)) => Err(bad("give `signed_lease` or `lease`, not both")),
            (Some(sl), None) => {
                let a = sl
                    .get("attestation")
                    .ok_or_else(|| bad("`signed_lease` needs an `attestation`"))?;
                let attestation = Attestation::from_json(
                    &json!({
                        "key_id": str_field(a, "key_id")?,
                        "payload_blake3": str_field(a, "payload_blake3")?,
                        "signature": str_field(a, "signature")?,
                    })
                    .to_string(),
                )
                .map_err(|e| bad(format!("`attestation`: {e}")))?;
                let max_skew_secs = match v.get("max_skew_secs") {
                    None | Some(Value::Null) => DEFAULT_MAX_SKEW_SECS,
                    Some(n) => n
                        .as_u64()
                        .ok_or_else(|| bad("`max_skew_secs` must be an unsigned integer"))?,
                };
                let signed = SignedLease {
                    lease: lease_obj(
                        sl.get("lease")
                            .ok_or_else(|| bad("`signed_lease` needs a `lease`"))?,
                    )?,
                    attestation,
                };
                let trusted = trusted_keys_of(v)?;
                crate::attest::verify_lease(&signed, &trusted).map_err(|e| {
                    super::ops::governance::law_err(refuse(
                        &signed.lease,
                        first_step,
                        match e {
                            crate::attest::AttestError::UnknownKey => LeaseReason::UntrustedKey,
                            _ => LeaseReason::BadSignature,
                        },
                    ))
                })?;
                let now = clock.now_unix();
                if signed.lease.issued_unix > now.saturating_add(max_skew_secs) {
                    return Err(super::ops::governance::law_err(refuse(
                        &signed.lease,
                        first_step,
                        LeaseReason::ClockSkew,
                    )));
                }
                if now >= signed.lease.expires_unix {
                    return Err(super::ops::governance::law_err(refuse(
                        &signed.lease,
                        first_step,
                        LeaseReason::Expired,
                    )));
                }
                Ok(LeaseContext {
                    lease: Some(signed.lease),
                    kind: LeaseContextKind::SignedVerified,
                })
            }
            (None, Some(l)) => {
                if v.get("unverified_lease") != Some(&Value::Bool(true)) {
                    return Err(Fail {
                        refusal: bad_refusal(
                            "an unsigned `lease` is refused: supply `signed_lease` + \
                             `trusted_keys`, or set `\"unverified_lease\": true` to accept a \
                             lease no one signed",
                        ),
                        details: Some(json!({"code": "UnverifiedLeaseRefused"})),
                    });
                }
                let now = v
                    .get("now_unix")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| bad("an unverified `lease` requires integer `now_unix`"))?;
                let lease = lease_obj(l)?;
                if now >= lease.expires_unix {
                    return Err(super::ops::governance::law_err(refuse(
                        &lease,
                        first_step,
                        LeaseReason::Expired,
                    )));
                }
                Ok(LeaseContext {
                    lease: Some(lease),
                    kind: LeaseContextKind::UnverifiedPinned { now_unix: now },
                })
            }
        }
    }

    /// Id of the admitted lease, if any.
    pub(in crate::abi) fn id(&self) -> Option<String> {
        self.lease.as_ref().map(|l| l.id.clone())
    }

    #[allow(dead_code)]
    pub(in crate::abi) fn kind(&self) -> LeaseContextKind {
        self.kind
    }

    /// Pure predicate over admitted facts: scope then ceiling. Never checks a
    /// signature and never reads a clock.
    pub(in crate::abi) fn authorize(&self, step: &str, required: Ceiling) -> Result<(), LawError> {
        let Some(l) = &self.lease else { return Ok(()) };
        if !l.scope.iter().any(|s| s == step) {
            return Err(refuse(l, step, LeaseReason::OutOfScope));
        }
        if required > l.ceiling {
            return Err(refuse(l, step, LeaseReason::Ceiling));
        }
        Ok(())
    }

    /// Ceiling-only predicate for steps with no scope row (`record-receipts`):
    /// a presented lease must still reach `required`; no lease is a no-op.
    pub(in crate::abi) fn authorize_unscoped(
        &self,
        step: &str,
        required: Ceiling,
    ) -> Result<(), LawError> {
        match &self.lease {
            Some(l) if required > l.ceiling => Err(refuse(l, step, LeaseReason::Ceiling)),
            _ => Ok(()),
        }
    }
}
