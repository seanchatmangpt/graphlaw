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
use crate::dialect::{Dialect, Refusal, RefusalKind, RefusalMessage, parse_rdf};
use crate::law::{LawError, LawState, Receipt, ReceiptReason};

const NS: &str = "urn:graphlaw:";

fn subject(child: &str) -> String {
    format!("{NS}receipt:{child}")
}

fn pred(name: &str) -> String {
    format!("{NS}p:{name}")
}

/// Stable refusal code: a receipt field that cannot be embedded in an IRI.
/// Append-only; see `docs/refusals.md` (register entry is coordinator-owned).
pub const REFUSAL_RECEIPT_IRI_INVALID: &str = "RECEIPT-IRI-INVALID";

/// Stable refusal code: a re-record of a child whose recorded fields differ
/// from what is already in the state (would poison the subject).
/// Append-only; registered in `docs/refusals.md`.
pub const REFUSAL_RECEIPT_RECORD_CONFLICT: &str = "RECEIPT-RECORD-CONFLICT";

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// One literal-object quad read directly from the dataset, with its graph
/// context and literal datatype/language preserved. No text re-parsing, so
/// escaped quotes, backslashes and newlines in literals round-trip exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Fact {
    graph: Option<String>,
    subject: String,
    predicate: String,
    lexical: String,
    datatype: String,
    language: Option<String>,
}

impl Fact {
    /// A recorded-receipt fact lives in the default graph as a plain
    /// `xsd:string` literal; anything else is not what [`record`] writes.
    fn is_plain_default(&self) -> bool {
        self.graph.is_none() && self.language.is_none() && self.datatype == XSD_STRING
    }
}

fn facts_of(state: &LawState) -> Vec<Fact> {
    let ds = state.dataset();
    let mut out = Vec::new();
    for q in ds.quads() {
        if let (
            purrdf::TermValue::Iri(s),
            purrdf::TermValue::Iri(p),
            purrdf::TermValue::Literal {
                lexical_form,
                datatype,
                language,
                ..
            },
        ) = (ds.term_value(q.s), ds.term_value(q.p), ds.term_value(q.o))
        {
            let graph = q.g.map(|g| match ds.term_value(g) {
                purrdf::TermValue::Iri(i) => i,
                other => format!("{other:?}"),
            });
            out.push(Fact {
                graph,
                subject: s,
                predicate: p,
                lexical: lexical_form,
                datatype,
                language,
            });
        }
    }
    out.retain(Fact::is_plain_default);
    out
}

/// Refuse (typed) any IRI text that could escape `<...>` in N-Triples.
fn check_iri(iri: &str) -> Result<(), LawError> {
    let bad = iri.chars().find(|c| {
        matches!(c, '<' | '>' | '"' | ' ' | '{' | '}' | '|' | '\\' | '^' | '`')
            || (*c as u32) <= 0x20
            || ((*c as u32) >= 0x7f && (*c as u32) <= 0x9f)
    });
    match bad {
        None => Ok(()),
        Some(c) => Err(LawError::Refused(Refusal {
            kind: RefusalKind::Unsupported,
            dialect: Some(Dialect::NTriples),
            engine: Some(Dialect::NTriples.engine()),
            message: RefusalMessage::Dynamic(format!(
                "[{REFUSAL_RECEIPT_IRI_INVALID}] receipt IRI contains forbidden character {c:?}"
            )),
        })),
    }
}

fn lit(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                o.push_str(&format!("\\u{:04X}", c as u32))
            }
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

/// [`record`] plus the receipt's [`Attestation`] (`attest_payload_blake3`,
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
    check_iri(&s)?;
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
    if let Some(d) = &receipt.plan_blake3 {
        facts.push(("plan_blake3", d.clone()));
    }
    if let Some(d) = &receipt.subject_blake3 {
        facts.push(("subject_blake3", d.clone()));
    }
    if let Some(a) = att {
        facts.push(("attest_payload_blake3", a.payload_blake3.clone()));
        facts.push(("attest_key_id", a.key_id.clone()));
        facts.push(("attest_signature", a.signature.clone()));
    }
    // Idempotence / conflict law (attested form). Re-recording the exact same receipt (or
    // attaching an attestation to a previously unattested one) is a no-op or
    // a pure extension; any differing value for an already-recorded predicate,
    // or a dropped optional field, would leave the subject ambiguous and is
    // refused with `RECEIPT-RECORD-CONFLICT`.
    let existing = facts_of(state);
    let is_attest = |p: &str| p.starts_with("attest_");
    let conflict = |why: String| {
        LawError::Refused(Refusal {
            kind: RefusalKind::Unsupported,
            dialect: Some(Dialect::NTriples),
            engine: Some(Dialect::NTriples.engine()),
            message: RefusalMessage::Dynamic(format!(
                "[{REFUSAL_RECEIPT_RECORD_CONFLICT}] receipt `{}` {why}",
                receipt.child
            )),
        })
    };
    let mut missing: Vec<(&str, String)> = Vec::new();
    // Plain `record` keeps its additive semantics (two distinct receipts may
    // legitimately share a `child` when a step is a state no-op); the
    // conflict law binds the attested form, whose subject must stay unique.
    let enforce = att.is_some();
    if !enforce {
        missing = facts.clone();
    }
    for (p, o) in facts.iter().filter(|_| enforce) {
        match objects_of(&existing, &s, &pred(p)).as_slice() {
            [] => missing.push((*p, o.clone())),
            [x] if *x == o.as_str() => {}
            _ => return Err(conflict(format!("is already recorded with a different `{p}`"))),
        }
    }
    let prefix = format!("{NS}p:");
    for f in existing.iter().filter(|f| enforce && f.subject == s) {
        if let Some(name) = f.predicate.strip_prefix(&prefix)
            && !is_attest(name)
            && !facts.iter().any(|(p, _)| *p == name)
        {
            return Err(conflict(format!(
                "is already recorded with `{name}` that this record omits"
            )));
        }
    }
    if enforce && missing.is_empty() {
        return Ok(state.clone());
    }
    let mut all: BTreeSet<String> = lines(state)?.into_iter().collect();
    let mut nt = String::new();
    for (p, o) in missing {
        nt.push_str(&format!("<{s}> <{}> {} .\n", pred(p), lit(&o)));
    }
    let ds = parse_rdf(nt.as_bytes(), Dialect::NTriples, None)?;
    let tmp = LawState::from_dataset(ds)?;
    all.extend(lines(&tmp)?);
    let doc = all.iter().map(|l| format!("{l}\n")).collect::<String>();
    Ok(LawState::parse(doc.as_bytes(), Dialect::NQuads, None)?)
}

fn objects_of<'a>(fs: &'a [Fact], subj: &str, p: &str) -> Vec<&'a str> {
    fs.iter()
        .filter(|f| f.subject == subj && f.predicate == p)
        .map(|f| f.lexical.as_str())
        .collect()
}

/// Refuse unless a receipt for `step` is recorded in `state`, with its subject
/// keyed by its own `child` id (a receipt about another subject is not accepted).
pub fn require(state: &LawState, step: &str) -> Result<(), LawError> {
    let fs = facts_of(state);
    let step_pred = pred("step");
    let child_pred = pred("child");
    let ok = fs.iter().any(|f| {
        f.predicate == step_pred
            && f.lexical == step
            && objects_of(&fs, &f.subject, &child_pred)
                .iter()
                .any(|c| f.subject == subject(c))
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
    let ls = facts_of(state);
    let step_pred = pred("step");
    let mut first_reason: Option<ReceiptReason> = None;
    // Deterministic candidate order: dataset iteration order is unspecified,
    // so the reported refusal reason must not depend on it.
    let mut candidates: Vec<&Fact> = ls
        .iter()
        .filter(|f| f.predicate == step_pred && f.lexical == step)
        .collect();
    candidates.sort_by(|a, b| a.subject.cmp(&b.subject));
    candidates.dedup_by(|a, b| a.subject == b.subject);
    for f in candidates {
        let subj = &f.subject;
        let Some(child) = single(&ls, subj, "child") else {
            continue;
        };
        if subj != &subject(child) {
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
fn single<'a>(ls: &'a [Fact], subj: &str, name: &str) -> Option<&'a str> {
    match objects_of(ls, subj, &pred(name)).as_slice() {
        [x] => Some(*x),
        _ => None,
    }
}

fn check_attested(
    ls: &[Fact],
    subj: &str,
    step: &str,
    child: &str,
    trusted: &TrustedKeys,
) -> Result<(), ReceiptReason> {
    let one = |name: &str| single(ls, subj, name);
    let (Some(payload), Some(key_id), Some(signature)) = (
        one("attest_payload_blake3"),
        one("attest_key_id"),
        one("attest_signature"),
    ) else {
        return Err(ReceiptReason::Unattested);
    };
    let att = Attestation {
        payload_blake3: payload.to_string(),
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
        plan_blake3: opt("plan_blake3")?,
        subject_blake3: opt("subject_blake3")?,
    };
    verify_bytes(receipt_payload(&fields).as_bytes(), &att, trusted).map_err(|e| match e {
        AttestError::UnknownKey => ReceiptReason::UntrustedKey,
        _ => ReceiptReason::BadSignature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attest::{SigningKey, sign_receipt};
    use crate::law::Step;

    fn base() -> LawState {
        LawState::parse(
            b"<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n\
              <urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n",
            Dialect::NTriples,
            None,
        )
        .unwrap()
    }

    #[test]
    fn re_recording_the_same_signed_receipt_is_idempotent() {
        let (child, r) = base().transition(&Step::EntailRdfs).unwrap();
        let k = SigningKey::from_seed([1; 32]);
        let mut trusted = TrustedKeys::new();
        trusted.insert(k.verifying_key());
        let att = sign_receipt(&k, &r);
        let once = record_signed(&child, &r, &att).unwrap();
        let twice = record_signed(&once, &r, &att).unwrap();
        assert_eq!(once.id(), twice.id());
        require_signed(&twice, "derive:rdfs", &trusted).unwrap();
    }

    #[test]
    fn conflicting_re_record_is_refused_and_does_not_poison_the_subject() {
        let (child, r) = base().transition(&Step::EntailRdfs).unwrap();
        let k = SigningKey::from_seed([1; 32]);
        let mut trusted = TrustedKeys::new();
        trusted.insert(k.verifying_key());
        let att = sign_receipt(&k, &r);
        let once = record_signed(&child, &r, &att).unwrap();
        let mut edited = r.clone();
        edited.added += 5;
        let refused = record_signed(&once, &edited, &att).unwrap_err();
        assert!(
            format!("{refused:?}").contains(REFUSAL_RECEIPT_RECORD_CONFLICT),
            "{refused:?}"
        );
        // The original receipt still verifies: the failed re-record left no trace.
        require_signed(&once, "derive:rdfs", &trusted).unwrap();
    }

    #[test]
    fn attesting_a_previously_unattested_record_extends_it() {
        let (child, r) = base().transition(&Step::EntailRdfs).unwrap();
        let k = SigningKey::from_seed([1; 32]);
        let mut trusted = TrustedKeys::new();
        trusted.insert(k.verifying_key());
        let plain = record(&child, &r).unwrap();
        let signed = record_signed(&plain, &r, &sign_receipt(&k, &r)).unwrap();
        require_signed(&signed, "derive:rdfs", &trusted).unwrap();
    }
}
