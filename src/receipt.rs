//! Receipt feedback: `R_t -> O*_{t+1}`.
//!
//! [`record`] writes a [`Receipt`] into a [`LawState`] as RDF, so the record of
//! a prior transition becomes part of the observation the *next* admission
//! sees (and changes the state id). [`require`] is the gate behind
//! `Step::RequireReceipt`: it refuses unless the state holds a recorded receipt
//! for the named step whose subject is keyed by that receipt's own `child` id.
//!
//! **[`require`] proves presence only**: anyone who can write triples can write
//! a receipt. [`record_signed`] additionally records an Ed25519
//! [`Attestation`] and [`require_signed`] refuses unless the receipt for the
//! step carries a valid attestation by a trusted key over its recorded fields.

use std::collections::BTreeSet;

use crate::attest::{
    AttestError, Attestation, ReceiptFields, TrustedKeys, receipt_payload, verify_bytes,
};
use crate::dialect::{Dialect, Refusal, parse_rdf};
use crate::law::{LawError, LawState, Receipt, ReceiptReason};

const NS: &str = "urn:graphlaw:";

fn subject(child: &str) -> String {
    format!("<{NS}receipt:{child}>")
}

fn pred(name: &str) -> String {
    format!("<{NS}p:{name}>")
}

fn lit(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn lines(state: &LawState) -> Result<Vec<String>, Refusal> {
    let bytes = purrdf::serialize_dataset(
        state.dataset().as_ref(),
        "application/n-quads",
        purrdf::SerializeGraph::Dataset,
    )
    .map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
    let text = String::from_utf8(bytes).map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}

/// Add `receipt` to `state` as RDF triples about `urn:graphlaw:receipt:<child>`.
/// The record is unattested: see [`record_signed`] for the verifiable form.
pub fn record(state: &LawState, receipt: &Receipt) -> Result<LawState, LawError> {
    record_facts(state, receipt, None)
}

/// [`record`] plus the receipt's [`Attestation`] (`attest_payload_sha256`,
/// `attest_key_id`, `attest_signature`). Recording does not verify; the gate
/// [`require_signed`] does, against the verifier's trusted keys.
pub fn record_signed(
    state: &LawState,
    receipt: &Receipt,
    attestation: &Attestation,
) -> Result<LawState, LawError> {
    record_facts(state, receipt, Some(attestation))
}

fn record_facts(
    state: &LawState,
    receipt: &Receipt,
    att: Option<&Attestation>,
) -> Result<LawState, LawError> {
    let s = subject(&receipt.child);
    let facts = [
        ("parent", receipt.parent.clone()),
        ("child", receipt.child.clone()),
        ("step", receipt.step.to_string()),
        ("authority", receipt.authority.authority.to_string()),
        ("revision", receipt.authority.revision.to_string()),
        ("added", receipt.added.to_string()),
    ];
    let mut facts = facts.to_vec();
    if let Some(id) = &receipt.lease_id {
        facts.push(("lease_id", id.clone()));
    }
    if let Some(d) = &receipt.plan_sha256 {
        facts.push(("plan_sha256", d.clone()));
    }
    if let Some(d) = &receipt.subject_sha256 {
        facts.push(("subject_sha256", d.clone()));
    }
    if let Some(a) = att {
        facts.push(("attest_payload_sha256", a.payload_sha256.clone()));
        facts.push(("attest_key_id", a.key_id.clone()));
        facts.push(("attest_signature", a.signature.clone()));
    }
    let mut all: BTreeSet<String> = lines(state)?.into_iter().collect();
    let mut nt = String::new();
    for (p, o) in facts {
        nt.push_str(&format!("{s} {} {} .\n", pred(p), lit(&o)));
    }
    let ds = parse_rdf(nt.as_bytes(), Dialect::NTriples, None)?;
    let tmp = LawState::from_dataset(ds)?;
    all.extend(lines(&tmp)?);
    let doc = all.iter().map(|l| format!("{l}\n")).collect::<String>();
    Ok(LawState::parse(doc.as_bytes(), Dialect::NQuads, None)?)
}

fn objects_of<'a>(ls: &'a [String], subj: &str, p: &str) -> Vec<&'a str> {
    let prefix = format!("{subj} {p} \"");
    ls.iter()
        .filter_map(|l| l.strip_prefix(&prefix))
        .filter_map(|r| r.split_once('"').map(|(v, _)| v))
        .collect()
}

/// Refuse unless a receipt for `step` is recorded in `state`, with its subject
/// keyed by its own `child` id (a receipt about another subject is not accepted).
pub fn require(state: &LawState, step: &str) -> Result<(), LawError> {
    let ls = lines(state)?;
    let step_prefix = pred("step");
    let child_pred = pred("child");
    let ok = ls.iter().any(|l| {
        let Some((subj, rest)) = l.split_once(' ') else {
            return false;
        };
        let Some(obj) = rest.strip_prefix(&format!("{step_prefix} \"")) else {
            return false;
        };
        if obj.split_once('"').map(|(v, _)| v) != Some(step) {
            return false;
        }
        objects_of(&ls, subj, &child_pred)
            .iter()
            .any(|c| subj == subject(c))
    });
    if ok {
        Ok(())
    } else {
        Err(LawError::ReceiptRequired {
            step: step.to_string(),
        })
    }
}

/// Refuse unless a receipt for `step` is recorded, keyed by its own `child`
/// id, **and** carries an attestation by a key in `trusted` that verifies over
/// the receipt's recorded fields (parent, child, step, authority, revision,
/// added, lease id, plan digest, subject digest). Hand-written triples, a
/// receipt whose fields were edited after signing, or one signed by an
/// untrusted key are all refused with [`LawError::ReceiptRefused`].
pub fn require_signed(state: &LawState, step: &str, trusted: &TrustedKeys) -> Result<(), LawError> {
    let ls = lines(state)?;
    let step_prefix = format!("{} \"", pred("step"));
    let mut first_reason: Option<ReceiptReason> = None;
    for l in &ls {
        let Some((subj, rest)) = l.split_once(' ') else {
            continue;
        };
        let Some(obj) = rest.strip_prefix(&step_prefix) else {
            continue;
        };
        if obj.split_once('"').map(|(v, _)| v) != Some(step) {
            continue;
        }
        let Some(child) = single(&ls, subj, "child") else {
            continue;
        };
        if subj != subject(child) {
            continue;
        }
        let reason = match check_attested(&ls, subj, step, child, trusted) {
            Ok(()) => return Ok(()),
            Err(r) => r,
        };
        first_reason.get_or_insert(reason);
    }
    Err(match first_reason {
        None => LawError::ReceiptRequired {
            step: step.to_string(),
        },
        Some(reason) => LawError::ReceiptRefused {
            step: step.to_string(),
            reason,
        },
    })
}

/// The one value of predicate `name` on `subj`; `None` when absent or ambiguous.
fn single<'a>(ls: &'a [String], subj: &str, name: &str) -> Option<&'a str> {
    match objects_of(ls, subj, &pred(name)).as_slice() {
        [x] => Some(*x),
        _ => None,
    }
}

fn check_attested(
    ls: &[String],
    subj: &str,
    step: &str,
    child: &str,
    trusted: &TrustedKeys,
) -> Result<(), ReceiptReason> {
    let one = |name: &str| single(ls, subj, name);
    let (Some(payload), Some(key_id), Some(signature)) = (
        one("attest_payload_sha256"),
        one("attest_key_id"),
        one("attest_signature"),
    ) else {
        return Err(ReceiptReason::Unattested);
    };
    let att = Attestation {
        payload_sha256: payload.to_string(),
        key_id: key_id.to_string(),
        signature: signature.to_string(),
    };
    let field = |name: &str| one(name).ok_or(ReceiptReason::BadSignature);
    // Optional fields: absent is `null` in the payload; ambiguous (>1) is refused.
    let opt = |name: &str| -> Result<Option<&str>, ReceiptReason> {
        match objects_of(ls, subj, &pred(name)).as_slice() {
            [] => Ok(None),
            [x] => Ok(Some(*x)),
            _ => Err(ReceiptReason::BadSignature),
        }
    };
    let added: u64 = field("added")?
        .parse()
        .map_err(|_| ReceiptReason::BadSignature)?;
    let fields = ReceiptFields {
        parent: field("parent")?,
        child,
        step,
        authority: field("authority")?,
        revision: field("revision")?,
        added,
        lease_id: opt("lease_id")?,
        plan_sha256: opt("plan_sha256")?,
        subject_sha256: opt("subject_sha256")?,
    };
    verify_bytes(receipt_payload(&fields).as_bytes(), &att, trusted).map_err(|e| match e {
        AttestError::UnknownKey => ReceiptReason::UntrustedKey,
        _ => ReceiptReason::BadSignature,
    })
}
