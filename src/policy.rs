//! Independent admission of a FOND policy proposed by a planner (ferroplan).
//!
//! `admit(problem_json, policy_json)` re-verifies STRONG-CYCLIC from the problem
//! alone: (a) every reachable non-goal state has a policy entry, (b) every
//! outcome is a real transition of the problem and each (state, action) mass sums
//! to 1_000_000 ppm, (c) the goal is reachable from every reachable state under
//! the policy. Goal = fact-subset check (numeric bounds are not modelled).
//! The JSON entry points require feature `abi` (serde_json).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Why a policy was refused.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::policy::PolicyRefusalKind;
///
/// let k = PolicyRefusalKind::DeadEnd;
/// let code = match k {
///     PolicyRefusalKind::DeadEnd => "dead_end",
///     _ => "other", // required: new kinds may be added
/// };
/// assert_eq!(code, "dead_end");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PolicyRefusalKind {
    /// A reachable state lacks a policy entry.
    MissingEntry,
    /// An outcome names a state the domain does not allow.
    InventedOutcome,
    /// Outcome probabilities do not sum to one million ppm.
    BadMass,
    /// A reachable non-goal state has no applicable action.
    DeadEnd,
    /// The policy document is not well formed.
    Malformed,
}

impl PolicyRefusalKind {
    /// Stable wire name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingEntry => "MissingEntry",
            Self::InventedOutcome => "InventedOutcome",
            Self::BadMass => "BadMass",
            Self::DeadEnd => "DeadEnd",
            Self::Malformed => "Malformed",
        }
    }
}

/// Typed refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyRefused {
    /// Machine-readable refusal class.
    pub kind: PolicyRefusalKind,
    /// State the refusal concerns.
    pub state: String,
    /// Action the refusal concerns.
    pub action: String,
    /// Diagnostic text.
    pub message: String,
}

impl PolicyRefused {
    fn new(kind: PolicyRefusalKind, state: &str, action: &str, message: String) -> Self {
        Self {
            kind,
            state: state.to_string(),
            action: action.to_string(),
            message,
        }
    }
}

/// Admitted strong-cyclic policy summary.
///
/// ```
/// use std::collections::{BTreeMap, BTreeSet};
/// use graphlaw::policy::{admit_parsed, Entry, Outcome, Problem};
///
/// let facts = |f: &[&str]| f.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>();
/// let problem = Problem {
///     states: BTreeMap::from([
///         ("s0".to_string(), facts(&["start"])),
///         ("s1".to_string(), facts(&["done"])),
///     ]),
///     initial: vec!["s0".into()],
///     goal_facts: facts(&["done"]),
///     unsafe_states: BTreeSet::new(),
///     transitions: BTreeSet::from([("go".to_string(), "s0".to_string(), "s1".to_string())]),
/// };
/// let policy = [Entry {
///     state: "s0".into(),
///     action: "go".into(),
///     outcomes: vec![Outcome { state: "s1".into(), probability_ppm: 1_000_000 }],
/// }];
/// let admitted = admit_parsed(&problem, &policy)?;
/// assert_eq!(admitted.goal_states, vec!["s1".to_string()]);
/// assert!(admitted.to_ntriples().contains("STRONG_CYCLIC"));
/// # Ok::<(), graphlaw::policy::PolicyRefused>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyAdmitted {
    /// Initial state names.
    pub initial_states: Vec<String>,
    /// Reachable states in sorted order.
    pub reachable: Vec<String>,
    /// Goal state names.
    pub goal_states: Vec<String>,
    /// (state, action) for every reachable non-goal state.
    pub entries: Vec<(String, String)>,
}

const NS: &str = "https://graphlaw.dev/policy#";
const MASS: u64 = 1_000_000;

fn lit(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '"' => o.push_str("\\\""),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

impl PolicyAdmitted {
    /// Deterministic N-Triples rendering of the admission.
    pub fn to_ntriples(&self) -> String {
        let mut out = String::new();
        let a = format!("<{NS}admission>");
        out.push_str(&format!(
            "{a} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{NS}AdmittedPolicy> .\n"
        ));
        out.push_str(&format!("{a} <{NS}standing> \"STRONG_CYCLIC\" .\n"));
        for s in &self.initial_states {
            out.push_str(&format!("{a} <{NS}initialState> {} .\n", lit(s)));
        }
        for s in &self.reachable {
            out.push_str(&format!("{a} <{NS}reachableState> {} .\n", lit(s)));
        }
        for s in &self.goal_states {
            out.push_str(&format!("{a} <{NS}goalState> {} .\n", lit(s)));
        }
        for (i, (s, act)) in self.entries.iter().enumerate() {
            let e = format!("<{NS}entry/{i}>");
            out.push_str(&format!("{a} <{NS}entry> {e} .\n"));
            out.push_str(&format!("{e} <{NS}state> {} .\n", lit(s)));
            out.push_str(&format!("{e} <{NS}action> {} .\n", lit(act)));
        }
        out
    }
}

/// One outcome of a policy entry.
#[derive(Clone, Debug)]
pub struct Outcome {
    /// State reached by the outcome.
    pub state: String,
    /// Probability in parts per million.
    pub probability_ppm: u64,
}

/// One policy entry.
#[derive(Clone, Debug)]
pub struct Entry {
    /// State the entry applies to.
    pub state: String,
    /// Action chosen in the state.
    pub action: String,
    /// Possible outcomes of the action.
    pub outcomes: Vec<Outcome>,
}

/// Parsed problem: state facts, initial states, goal facts, real transitions.
#[derive(Clone, Debug, Default)]
pub struct Problem {
    /// Facts holding in each named state.
    pub states: BTreeMap<String, BTreeSet<String>>,
    /// Initial state names.
    pub initial: Vec<String>,
    /// Facts that define the goal.
    pub goal_facts: BTreeSet<String>,
    /// States that must never be reached.
    pub unsafe_states: BTreeSet<String>,
    /// (action, from, to)
    pub transitions: BTreeSet<(String, String, String)>,
}

fn malformed(message: String) -> PolicyRefused {
    PolicyRefused::new(PolicyRefusalKind::Malformed, "", "", message)
}

/// Core admission over parsed structures.
pub fn admit_parsed(problem: &Problem, policy: &[Entry]) -> Result<PolicyAdmitted, PolicyRefused> {
    if problem.initial.is_empty() {
        return Err(malformed("problem has no initial states".into()));
    }
    for s in &problem.initial {
        if !problem.states.contains_key(s) {
            return Err(malformed(format!("initial state `{s}` is not declared")));
        }
    }
    let mut by_state: BTreeMap<&str, &Entry> = BTreeMap::new();
    for e in policy {
        if by_state.insert(e.state.as_str(), e).is_some() {
            return Err(PolicyRefused::new(
                PolicyRefusalKind::Malformed,
                &e.state,
                &e.action,
                format!("duplicate policy entry for state `{}`", e.state),
            ));
        }
    }
    let is_goal = |s: &str| {
        problem
            .states
            .get(s)
            .is_some_and(|f| problem.goal_facts.is_subset(f))
    };

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    for s in &problem.initial {
        if seen.insert(s.clone()) {
            queue.push_back(s.clone());
        }
    }
    let mut succ: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut entries = Vec::new();
    while let Some(s) = queue.pop_front() {
        if problem.unsafe_states.contains(&s) {
            return Err(PolicyRefused::new(
                PolicyRefusalKind::DeadEnd,
                &s,
                "",
                format!("policy reaches unsafe state `{s}`"),
            ));
        }
        if is_goal(&s) {
            continue;
        }
        let Some(e) = by_state.get(s.as_str()) else {
            return Err(PolicyRefused::new(
                PolicyRefusalKind::MissingEntry,
                &s,
                "",
                format!("reachable non-goal state `{s}` has no policy entry"),
            ));
        };
        let mut mass = 0u64;
        let mut next = Vec::new();
        for o in &e.outcomes {
            let key = (e.action.clone(), s.clone(), o.state.clone());
            if !problem.transitions.contains(&key) {
                return Err(PolicyRefused::new(
                    PolicyRefusalKind::InventedOutcome,
                    &s,
                    &e.action,
                    format!(
                        "outcome `{}` of action `{}` from `{s}` is not a transition of the problem",
                        o.state, e.action
                    ),
                ));
            }
            mass += o.probability_ppm;
            next.push(o.state.clone());
        }
        if mass != MASS {
            return Err(PolicyRefused::new(
                PolicyRefusalKind::BadMass,
                &s,
                &e.action,
                format!(
                    "probability mass {mass} ppm != {MASS} for `{s}`/`{}`",
                    e.action
                ),
            ));
        }
        entries.push((s.clone(), e.action.clone()));
        for n in &next {
            if seen.insert(n.clone()) {
                queue.push_back(n.clone());
            }
        }
        succ.insert(s, next);
    }

    // (c) backward closure from goal states over the policy graph.
    // Worklist over reverse edges: each state is visited once, O(S + E), where
    // repeated full sweeps were O(S^2) on a long chain.
    let mut good: BTreeSet<String> = seen.iter().filter(|s| is_goal(s)).cloned().collect();
    let mut preds: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (s, ns) in &succ {
        for n in ns {
            preds.entry(n.as_str()).or_default().push(s.as_str());
        }
    }
    let mut work: Vec<String> = good.iter().cloned().collect();
    while let Some(n) = work.pop() {
        for p in preds.get(n.as_str()).into_iter().flatten() {
            if !good.contains(*p) {
                good.insert((*p).to_string());
                work.push((*p).to_string());
            }
        }
    }
    if let Some(s) = seen.iter().find(|s| !good.contains(*s)) {
        let action = by_state.get(s.as_str()).map_or("", |e| e.action.as_str());
        return Err(PolicyRefused::new(
            PolicyRefusalKind::DeadEnd,
            s,
            action,
            format!("goal is unreachable under the policy from `{s}`"),
        ));
    }
    let goal_states = seen.iter().filter(|s| is_goal(s)).cloned().collect();
    Ok(PolicyAdmitted {
        initial_states: problem.initial.clone(),
        reachable: seen.into_iter().collect(),
        goal_states,
        entries,
    })
}

#[cfg(feature = "abi")]
mod json {
    use super::*;
    use serde_json::Value;

    fn str_field<'a>(v: &'a Value, k: &str, ctx: &str) -> Result<&'a str, PolicyRefused> {
        v.get(k)
            .and_then(Value::as_str)
            .ok_or_else(|| malformed(format!("{ctx}: missing string `{k}`")))
    }

    fn parse_problem(text: &str) -> Result<Problem, PolicyRefused> {
        let v: Value =
            serde_json::from_str(text).map_err(|e| malformed(format!("problem: {e}")))?;
        let mut p = Problem::default();
        for s in v
            .get("states")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let id = str_field(s, "id", "state")?;
            let facts = s
                .get("facts")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect();
            p.states.insert(id.to_string(), facts);
        }
        for s in v
            .get("initial_states")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            p.initial.push(
                s.as_str()
                    .ok_or_else(|| malformed("initial_states: non-string".into()))?
                    .to_string(),
            );
        }
        for f in v
            .pointer("/goal/facts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            p.goal_facts.insert(
                f.as_str()
                    .ok_or_else(|| malformed("goal.facts: non-string".into()))?
                    .to_string(),
            );
        }
        for s in v
            .get("unsafe_states")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            p.unsafe_states.insert(s.to_string());
        }
        for t in v
            .get("transitions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            p.transitions.insert((
                str_field(t, "action", "transition")?.to_string(),
                str_field(t, "from", "transition")?.to_string(),
                str_field(t, "to", "transition")?.to_string(),
            ));
        }
        Ok(p)
    }

    fn parse_policy(text: &str) -> Result<Vec<Entry>, PolicyRefused> {
        let v: Value = serde_json::from_str(text).map_err(|e| malformed(format!("policy: {e}")))?;
        let arr = match &v {
            Value::Array(a) => a,
            Value::Object(_) => v
                .get("policy")
                .and_then(Value::as_array)
                .ok_or_else(|| malformed("policy: object lacks `policy` array".into()))?,
            _ => return Err(malformed("policy: expected array or plan object".into())),
        };
        let mut out = Vec::new();
        for e in arr {
            let mut outcomes = Vec::new();
            for o in e
                .get("outcomes")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                outcomes.push(Outcome {
                    state: str_field(o, "state", "outcome")?.to_string(),
                    probability_ppm: o
                        .get("probability_ppm")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| malformed("outcome: missing probability_ppm".into()))?,
                });
            }
            out.push(Entry {
                state: str_field(e, "state", "entry")?.to_string(),
                action: str_field(e, "action", "entry")?.to_string(),
                outcomes,
            });
        }
        Ok(out)
    }

    /// Admit a policy (ferroplan `UniversalPlan` object or bare entry array)
    /// against a `PlanningProblem` JSON.
    pub fn admit(problem_json: &str, policy_json: &str) -> Result<PolicyAdmitted, PolicyRefused> {
        admit_parsed(&parse_problem(problem_json)?, &parse_policy(policy_json)?)
    }
}

#[cfg(feature = "abi")]
pub use json::admit;
