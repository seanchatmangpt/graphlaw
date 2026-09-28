//! Receipt feedback: `R_t -> O*_{t+1}`.
//!
//! [`record`] writes a [`Receipt`] into a [`LawState`] as RDF, so the record of
//! a prior transition becomes part of the observation the *next* admission
//! sees (and changes the state id). [`require`] is the gate behind
//! `Step::RequireReceipt`: it refuses unless the state holds a recorded receipt
//! for the named step whose subject is keyed by that receipt's own `child` id.

use std::collections::BTreeSet;

use crate::dialect::{Dialect, Refusal, parse_rdf};
use crate::law::{LawError, LawState, Receipt};

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
pub fn record(state: &LawState, receipt: &Receipt) -> Result<LawState, LawError> {
    let s = subject(&receipt.child);
    let facts = [
        ("parent", receipt.parent.clone()),
        ("child", receipt.child.clone()),
        ("step", receipt.step.to_string()),
        ("authority", receipt.authority.authority.to_string()),
        ("revision", receipt.authority.revision.to_string()),
        ("added", receipt.added.to_string()),
    ];
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
