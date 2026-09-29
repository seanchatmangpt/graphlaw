//! Independent plan admission: replay a candidate plan over a [`LawState`].
//!
//! A planner (ferroplan, an LLM, a hand-written script) only *proposes*. A
//! [`Plan`] is admitted by replaying its actions in order against the state:
//! every precondition triple must be present in the current state, effects
//! delete then add triples, and the goal triples must hold at the end. The
//! first violated precondition (or an unmet goal) refuses the whole plan; no
//! state is produced. Every applied action yields a child state and a
//! [`Receipt`], so an admitted plan is a replayable chain of content-addressed
//! states.
//!
//! Triples are N-Triples text and are compared as canonical N-Quads lines
//! produced by PurRDF's serializer, so GraphLaw adds no RDF semantics of its
//! own. Blank nodes are refused: plan atoms are ground.

use std::collections::BTreeSet;

use crate::dialect::{Dialect, Refusal, RefusalKind, parse_rdf};
use crate::law::{LawError, LawState, Receipt};
use crate::{BACKEND_AUTHORITIES, BackendAuthority};

/// Capability the replay is executed under: RDF parsing/serialization/identity.
const CAPABILITY: &str = "RDF 1.2 / codecs / storage IR";

/// One ground action: preconditions and effects as N-Triples text.
#[derive(Debug, Clone, Default)]
pub struct Action {
    pub name: String,
    /// Triples that must hold before the action.
    pub pre: String,
    /// Triples added by the action.
    pub add: String,
    /// Triples removed by the action (removed before `add` is applied).
    pub del: String,
}

/// An ordered candidate plan and the goal it claims to reach.
#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub actions: Vec<Action>,
    /// Triples that must hold after the last action.
    pub goal: String,
}

/// A plan that was admitted: the final state and one receipt per action.
#[derive(Debug, Clone)]
pub struct Admitted {
    pub state: LawState,
    pub receipts: Vec<Receipt>,
    /// [`Plan::digest`] of the admitted plan.
    pub plan_digest: String,
}

fn authority() -> BackendAuthority {
    *BACKEND_AUTHORITIES
        .iter()
        .find(|a| a.capability == CAPABILITY)
        .expect("plan replay maps to a declared authority")
}

fn refuse(message: impl Into<String>) -> LawError {
    LawError::Refused(Refusal {
        kind: RefusalKind::Unsupported,
        dialect: Some(Dialect::NTriples),
        engine: Some(Dialect::NTriples.engine()),
        message: message.into(),
    })
}

fn serialize_lines(ds: &purrdf::RdfDataset) -> Result<BTreeSet<String>, Refusal> {
    let bytes =
        purrdf::serialize_dataset(ds, "application/n-quads", purrdf::SerializeGraph::Dataset)
            .map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
    let text = String::from_utf8(bytes).map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}

/// Ground atoms: N-Triples text as a set of canonical N-Quads lines.
fn atoms(nt: &str) -> Result<BTreeSet<String>, LawError> {
    if nt.trim().is_empty() {
        return Ok(BTreeSet::new());
    }
    let ds = parse_rdf(nt.as_bytes(), Dialect::NTriples, None)?;
    let lines = serialize_lines(&ds)?;
    if lines
        .iter()
        .any(|l| l.split_whitespace().any(|t| t.starts_with("_:")))
    {
        return Err(refuse(
            "plan atoms must be ground (blank nodes are refused)",
        ));
    }
    Ok(lines)
}

fn state_lines(state: &LawState) -> Result<BTreeSet<String>, LawError> {
    Ok(serialize_lines(state.dataset())?)
}

fn missing(required: &BTreeSet<String>, have: &BTreeSet<String>) -> Vec<String> {
    required.difference(have).cloned().collect()
}

fn rebuild(lines: &BTreeSet<String>) -> Result<LawState, LawError> {
    let doc = lines.iter().map(|l| format!("{l}\n")).collect::<String>();
    Ok(LawState::parse(doc.as_bytes(), Dialect::NQuads, None)?)
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

impl Plan {
    /// Canonical JSON of the plan (actions in order, keys sorted, no
    /// whitespace): `{"actions":[{"add","del","name","pre"}],"goal"}`.
    pub fn canonical_json(&self) -> String {
        let actions = self
            .actions
            .iter()
            .map(|a| {
                format!(
                    "{{\"add\":{},\"del\":{},\"name\":{},\"pre\":{}}}",
                    json_str(&a.add),
                    json_str(&a.del),
                    json_str(&a.name),
                    json_str(&a.pre)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"actions\":[{actions}],\"goal\":{}}}",
            json_str(&self.goal)
        )
    }

    /// `sha256:` of [`Plan::canonical_json`].
    pub fn digest(&self) -> String {
        use sha2::{Digest, Sha256};
        let d = Sha256::digest(self.canonical_json().as_bytes());
        format!(
            "sha256:{}",
            d.iter().map(|b| format!("{b:02x}")).collect::<String>()
        )
    }

    /// Replay the plan over `start`. Refuses at the first violated
    /// precondition, or after the last action if the goal does not hold.
    pub fn admit(&self, start: &LawState) -> Result<Admitted, LawError> {
        let mut state = start.clone();
        let mut have = state_lines(&state)?;
        let mut receipts = Vec::with_capacity(self.actions.len());
        let plan_digest = self.digest();
        for (index, action) in self.actions.iter().enumerate() {
            let need = atoms(&action.pre)?;
            let gap = missing(&need, &have);
            if !gap.is_empty() {
                return Err(LawError::PlanRefused {
                    index,
                    action: action.name.clone(),
                    missing: gap,
                });
            }
            let del = atoms(&action.del)?;
            let add = atoms(&action.add)?;
            let next: BTreeSet<String> = have.difference(&del).cloned().chain(add).collect();
            let child = rebuild(&next)?;
            receipts.push(Receipt {
                parent: state.id().to_string(),
                child: child.id().to_string(),
                step: "plan-action",
                authority: authority(),
                added: child.quad_count().saturating_sub(state.quad_count()),
                lease_id: None,
                plan_sha256: Some(plan_digest.clone()),
                subject_sha256: None,
            });
            state = child;
            have = next;
        }
        let gap = missing(&atoms(&self.goal)?, &have);
        if !gap.is_empty() {
            return Err(LawError::PlanRefused {
                index: self.actions.len(),
                action: "<goal>".to_string(),
                missing: gap,
            });
        }
        Ok(Admitted {
            state,
            receipts,
            plan_digest,
        })
    }
}
