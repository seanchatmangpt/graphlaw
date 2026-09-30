//! Independent plan admission: replay a candidate plan over a [`LawState`].
//!
//! A planner (ferroplan, an LLM, a hand-written script) only *proposes*. A
//! [`Plan`] is admitted by replaying its actions in order against the state:
//! every precondition triple must be present (and every `pre_not` triple
//! absent) in the current state, effects
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

/// Most N-Triples atoms (non-empty lines) summed over every field of one plan.
/// `MAX_ATOMS_PER_FIELD` bounds a single field; this bounds their sum, since
/// up to `MAX_PLAN_ACTIONS` actions of five fields each would otherwise multiply
/// it.
pub const MAX_PLAN_TOTAL_ATOMS: usize = 100_000;

/// One ground action: preconditions and effects as N-Triples text.
///
/// ```
/// use graphlaw::plan::Action;
///
/// let a = Action {
///     name: "open".into(),
///     pre: "<urn:d> <urn:p:is> <urn:v:closed> .".into(),
///     pre_not: "<urn:d> <urn:p:is> <urn:v:locked> .".into(),
///     add: "<urn:d> <urn:p:is> <urn:v:open> .".into(),
///     del: "<urn:d> <urn:p:is> <urn:v:closed> .".into(),
/// };
/// assert_eq!(a.name, "open");
/// ```
#[derive(Debug, Clone, Default)]
pub struct Action {
    /// Action name, for example `go a b`.
    pub name: String,
    /// Triples that must hold before the action.
    pub pre: String,
    /// Triples that must be ABSENT before the action (PDDL `(not p)`);
    /// closed-world over ground atoms. Checked after `pre`.
    pub pre_not: String,
    /// Triples added by the action.
    pub add: String,
    /// Triples removed by the action (removed before `add` is applied).
    pub del: String,
}

/// An ordered candidate plan and the goal it claims to reach.
///
/// ```
/// use graphlaw::{dialect::Dialect, law::LawState, plan::{Plan, Triple}};
///
/// let t = |o| Triple::iri("urn:d", "urn:p:is", o);
/// let start = LawState::parse(b"<urn:d> <urn:p:is> <urn:v:closed> .", Dialect::NTriples, None)?;
/// let plan = Plan::builder()
///     .action("open")
///     .requires(t("urn:v:closed")?)
///     .adds(t("urn:v:open")?)
///     .deletes(t("urn:v:closed")?)
///     .goal(t("urn:v:open")?)
///     .build()?;
/// let admitted = plan.admit(&start)?;
/// assert_eq!(admitted.receipts.len(), 1);
/// assert_eq!(admitted.plan_digest, plan.digest());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct Plan {
    /// Actions in replay order.
    pub actions: Vec<Action>,
    /// Triples that must hold after the last action.
    pub goal: String,
    /// Triples that must be absent after the last action.
    pub goal_not: String,
}

/// Why a [`Triple`] or a builder input was refused.
///
/// Marked `#[non_exhaustive]`: match with a wildcard arm.
///
/// ```
/// use graphlaw::plan::{Triple, TripleError};
///
/// match Triple::iri("urn:a>", "urn:p", "urn:o") {
///     Err(TripleError::InvalidIri { role, .. }) => assert_eq!(role, "subject"),
///     Err(_) => unreachable!("only IRI errors are produced here"),
///     Ok(_) => unreachable!("`>` is not legal inside an IRI"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TripleError {
    /// An IRI is empty or contains a character that N-Triples forbids in an
    /// IRIREF (`<`, `>`, `"`, `{`, `}`, `|`, `^`, backtick, backslash, space,
    /// or a control character).
    InvalidIri {
        /// Which position held the IRI: `subject`, `predicate` or `object`.
        role: &'static str,
        /// The rejected text.
        iri: String,
    },
    /// A builder was used in an invalid order or with an invalid name.
    Builder(String),
}

impl std::fmt::Display for TripleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TripleError::InvalidIri { role, iri } => {
                write!(f, "invalid IRI in {role} position: {iri:?}")
            }
            TripleError::Builder(m) => write!(f, "plan builder: {m}"),
        }
    }
}

impl std::error::Error for TripleError {}

fn check_iri(role: &'static str, iri: &str) -> Result<(), TripleError> {
    let bad = iri.is_empty()
        || iri.chars().any(|c| {
            (c as u32) <= 0x20
                || c as u32 == 0x7f
                || matches!(c, '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\')
        });
    if bad {
        Err(TripleError::InvalidIri {
            role,
            iri: iri.to_string(),
        })
    } else {
        Ok(())
    }
}

/// N-Triples STRING_LITERAL_QUOTE escaping: quote, backslash, and every
/// control character, so the text can never terminate the literal early.
fn escape_literal(text: &str) -> String {
    let mut o = String::with_capacity(text.len() + 2);
    o.push('"');
    for c in text.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                o.push_str(&format!("\\u{:04X}", c as u32));
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// One ground triple, rendered as a single N-Triples line (`<s> <p> <o> .`).
///
/// Construction validates IRIs and escapes literals, so a value of this type
/// is always exactly one well-formed triple and callers never hand-write
/// N-Triples.
///
/// ```
/// use graphlaw::plan::Triple;
///
/// let t = Triple::iri("urn:d:door", "urn:p:is", "urn:v:open")?;
/// assert_eq!(t.as_str(), "<urn:d:door> <urn:p:is> <urn:v:open> .");
/// let l = Triple::literal("urn:d:door", "urn:p:note", "a\"b\nc")?;
/// assert_eq!(l.as_str(), r#"<urn:d:door> <urn:p:note> "a\"b\nc" ."#);
/// # Ok::<(), graphlaw::plan::TripleError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Triple(String);

impl Triple {
    /// A triple whose object is an IRI. All three IRIs are validated.
    pub fn iri(s: &str, p: &str, o: &str) -> Result<Self, TripleError> {
        check_iri("subject", s)?;
        check_iri("predicate", p)?;
        check_iri("object", o)?;
        Ok(Triple(format!("<{s}> <{p}> <{o}> .")))
    }

    /// A triple whose object is a plain string literal; `text` is escaped.
    pub fn literal(s: &str, p: &str, text: &str) -> Result<Self, TripleError> {
        check_iri("subject", s)?;
        check_iri("predicate", p)?;
        Ok(Triple(format!("<{s}> <{p}> {} .", escape_literal(text))))
    }

    /// The N-Triples line.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Triple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn join(ts: &[Triple]) -> String {
    ts.iter().map(Triple::as_str).collect::<Vec<_>>().join("\n")
}

/// Builds one [`Action`] from typed [`Triple`]s.
///
/// ```
/// use graphlaw::plan::{Action, Triple};
///
/// let a = Action::builder("open")
///     .requires(Triple::iri("urn:d", "urn:p:is", "urn:v:closed")?)
///     .adds(Triple::iri("urn:d", "urn:p:is", "urn:v:open")?)
///     .deletes(Triple::iri("urn:d", "urn:p:is", "urn:v:closed")?)
///     .build();
/// assert_eq!(a.name, "open");
/// assert_eq!(a.add, "<urn:d> <urn:p:is> <urn:v:open> .");
/// # Ok::<(), graphlaw::plan::TripleError>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct ActionBuilder {
    name: String,
    pre: Vec<Triple>,
    pre_not: Vec<Triple>,
    add: Vec<Triple>,
    del: Vec<Triple>,
}

impl ActionBuilder {
    /// Start an action called `name`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// Add a precondition triple.
    pub fn requires(mut self, t: Triple) -> Self {
        self.pre.push(t);
        self
    }

    /// Add a triple that must be absent before the action (`(not p)`).
    pub fn requires_not(mut self, t: Triple) -> Self {
        self.pre_not.push(t);
        self
    }

    /// Add a triple asserted by the action.
    pub fn adds(mut self, t: Triple) -> Self {
        self.add.push(t);
        self
    }

    /// Add a triple retracted by the action.
    pub fn deletes(mut self, t: Triple) -> Self {
        self.del.push(t);
        self
    }

    /// Finish the action.
    pub fn build(self) -> Action {
        Action {
            name: self.name,
            pre: join(&self.pre),
            pre_not: join(&self.pre_not),
            add: join(&self.add),
            del: join(&self.del),
        }
    }
}

impl Action {
    /// Start an [`ActionBuilder`].
    pub fn builder(name: impl Into<String>) -> ActionBuilder {
        ActionBuilder::new(name)
    }
}

/// Builds a [`Plan`] from typed [`Triple`]s.
///
/// ```
/// use graphlaw::plan::{Plan, Triple};
///
/// let t = |o| Triple::iri("urn:d", "urn:p:is", o);
/// let plan = Plan::builder()
///     .action("open")
///     .requires(t("urn:v:closed")?)
///     .adds(t("urn:v:open")?)
///     .deletes(t("urn:v:closed")?)
///     .goal(t("urn:v:open")?)
///     .build()?;
/// assert_eq!(plan.actions.len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct PlanBuilder {
    done: Vec<Action>,
    current: Option<ActionBuilder>,
    goal: Vec<Triple>,
    goal_not: Vec<Triple>,
    error: Option<TripleError>,
}

impl PlanBuilder {
    /// Start the next action; any action in progress is finished first.
    pub fn action(mut self, name: impl Into<String>) -> Self {
        self.flush();
        self.current = Some(ActionBuilder::new(name));
        self
    }

    fn flush(&mut self) {
        if let Some(a) = self.current.take() {
            self.done.push(a.build());
        }
    }

    fn with_current(mut self, what: &str, f: impl FnOnce(ActionBuilder) -> ActionBuilder) -> Self {
        match self.current.take() {
            Some(a) => self.current = Some(f(a)),
            None if self.error.is_none() => {
                self.error = Some(TripleError::Builder(format!(
                    "`{what}` called before any `action`"
                )));
            }
            None => {}
        }
        self
    }

    /// Add a precondition to the current action.
    pub fn requires(self, t: Triple) -> Self {
        self.with_current("requires", |a| a.requires(t))
    }

    /// Add a must-be-absent triple to the current action.
    pub fn requires_not(self, t: Triple) -> Self {
        self.with_current("requires_not", |a| a.requires_not(t))
    }

    /// Add an asserted triple to the current action.
    pub fn adds(self, t: Triple) -> Self {
        self.with_current("adds", |a| a.adds(t))
    }

    /// Add a retracted triple to the current action.
    pub fn deletes(self, t: Triple) -> Self {
        self.with_current("deletes", |a| a.deletes(t))
    }

    /// Add a goal triple.
    pub fn goal(mut self, t: Triple) -> Self {
        self.goal.push(t);
        self
    }

    /// Add a triple that must be absent after the last action.
    pub fn goal_not(mut self, t: Triple) -> Self {
        self.goal_not.push(t);
        self
    }

    /// Finish the plan; refuses if a builder method was misused.
    pub fn build(mut self) -> Result<Plan, TripleError> {
        self.flush();
        if let Some(e) = self.error {
            return Err(e);
        }
        if let Some(a) = self.done.iter().find(|a| a.name.trim().is_empty()) {
            return Err(TripleError::Builder(format!(
                "action name must not be empty (got {:?})",
                a.name
            )));
        }
        Ok(Plan {
            actions: self.done,
            goal: join(&self.goal),
            goal_not: join(&self.goal_not),
        })
    }
}

impl Plan {
    /// Start a [`PlanBuilder`].
    pub fn builder() -> PlanBuilder {
        PlanBuilder::default()
    }
}

/// A plan that was admitted: the final state and one receipt per action.
#[derive(Debug, Clone)]
pub struct Admitted {
    /// The final state after the last action.
    pub state: LawState,
    /// One receipt per applied action.
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

fn atom_lines(nt: &str) -> usize {
    nt.lines().filter(|l| !l.trim().is_empty()).count()
}

fn check_total_atoms(plan: &Plan) -> Result<(), LawError> {
    let total = plan
        .actions
        .iter()
        .fold(0usize, |n, a| {
            [&a.pre, &a.pre_not, &a.add, &a.del]
                .into_iter()
                .fold(n, |n, f| n.saturating_add(atom_lines(f)))
        })
        .saturating_add(atom_lines(&plan.goal))
        .saturating_add(atom_lines(&plan.goal_not));
    if total > MAX_PLAN_TOTAL_ATOMS {
        return Err(LawError::Refused(Refusal {
            kind: RefusalKind::ResourceLimit,
            dialect: Some(Dialect::NTriples),
            engine: Some(Dialect::NTriples.engine()),
            message: format!(
                "resource limit `plan_total_atoms` exceeded: {total} > {MAX_PLAN_TOTAL_ATOMS}"
            ),
        }));
    }
    Ok(())
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

fn forbidden(absent: &BTreeSet<String>, have: &BTreeSet<String>) -> Vec<String> {
    absent.intersection(have).cloned().collect()
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
    /// `pre_not` (per action) and `goal_not` are included, in sorted key
    /// position, only when non-empty, so digests of positive-only plans are
    /// unchanged.
    pub fn canonical_json(&self) -> String {
        let actions = self
            .actions
            .iter()
            .map(|a| {
                let pre_not = if a.pre_not.is_empty() {
                    String::new()
                } else {
                    format!(",\"pre_not\":{}", json_str(&a.pre_not))
                };
                format!(
                    "{{\"add\":{},\"del\":{},\"name\":{},\"pre\":{}{pre_not}}}",
                    json_str(&a.add),
                    json_str(&a.del),
                    json_str(&a.name),
                    json_str(&a.pre)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let goal_not = if self.goal_not.is_empty() {
            String::new()
        } else {
            format!(",\"goal_not\":{}", json_str(&self.goal_not))
        };
        format!(
            "{{\"actions\":[{actions}],\"goal\":{}{goal_not}}}",
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
        check_total_atoms(self)?;
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
                    violated_absent: Vec::new(),
                });
            }
            let banned = forbidden(&atoms(&action.pre_not)?, &have);
            if !banned.is_empty() {
                return Err(LawError::PlanRefused {
                    index,
                    action: action.name.clone(),
                    missing: Vec::new(),
                    violated_absent: banned,
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
                violated_absent: Vec::new(),
            });
        }
        let banned = forbidden(&atoms(&self.goal_not)?, &have);
        if !banned.is_empty() {
            return Err(LawError::PlanRefused {
                index: self.actions.len(),
                action: "<goal>".to_string(),
                missing: Vec::new(),
                violated_absent: banned,
            });
        }
        Ok(Admitted {
            state,
            receipts,
            plan_digest,
        })
    }
}
