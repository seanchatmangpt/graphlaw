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

impl std::fmt::Display for PolicyRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "policy refused {} on state `{}` action `{}`: {}",
            self.kind.as_str(),
            self.state,
            self.action,
            self.message
        )
    }
}

impl std::error::Error for PolicyRefused {}

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
/// Mirror of `abi::MAX_POLICY_ENTRIES` (that module is feature-gated; this one is not).
const POLICY_CAP: usize = 100_000;
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
        use std::fmt::Write as _;
        let mut out = String::new();
        let a = format!("<{NS}admission>");
        let _ = writeln!(out,
            "{a} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{NS}AdmittedPolicy> ."
        );
        let _ = writeln!(out, "{a} <{NS}standing> \"STRONG_CYCLIC\" .");
        for s in &self.initial_states {
            let _ = writeln!(out, "{a} <{NS}initialState> {} .", lit(s));
        }
        for s in &self.reachable {
            let _ = writeln!(out, "{a} <{NS}reachableState> {} .", lit(s));
        }
        for s in &self.goal_states {
            let _ = writeln!(out, "{a} <{NS}goalState> {} .", lit(s));
        }
        for (i, (s, act)) in self.entries.iter().enumerate() {
            let e = format!("<{NS}entry/{i}>");
            let _ = writeln!(out, "{a} <{NS}entry> {e} .");
            let _ = writeln!(out, "{e} <{NS}state> {} .", lit(s));
            let _ = writeln!(out, "{e} <{NS}action> {} .", lit(act));
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
    let cap = POLICY_CAP;
    for (what, n) in [
        ("states", problem.states.len()),
        ("transitions", problem.transitions.len()),
        ("policy entries", policy.len()),
    ] {
        if n > cap {
            return Err(malformed(format!("{what} count {n} exceeds cap {cap}")));
        }
    }
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
            mass = mass.checked_add(o.probability_ppm).ok_or_else(|| {
                PolicyRefused::new(
                    PolicyRefusalKind::BadMass,
                    &s,
                    &e.action,
                    format!("probability mass overflows u64 for `{s}`/`{}`", e.action),
                )
            })?;
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

    /// Required array field: missing or wrong-typed is Malformed.
    fn req_arr<'a>(v: &'a Value, k: &str, ctx: &str) -> Result<&'a Vec<Value>, PolicyRefused> {
        v.get(k)
            .and_then(Value::as_array)
            .ok_or_else(|| malformed(format!("{ctx}: missing or non-array `{k}`")))
    }

    /// Optional array field: absent or explicit `null` is empty (serde
    /// `Option::None`); present but wrong-typed is Malformed.
    fn opt_arr<'a>(v: &'a Value, k: &str, ctx: &str) -> Result<&'a [Value], PolicyRefused> {
        match v.get(k) {
            None | Some(Value::Null) => Ok(&[]),
            Some(Value::Array(a)) => Ok(a),
            Some(_) => Err(malformed(format!("{ctx}: non-array `{k}`"))),
        }
    }

    fn parse_problem(text: &str) -> Result<Problem, PolicyRefused> {
        let v: Value =
            serde_json::from_str(text).map_err(|e| malformed(format!("problem: {e}")))?;
        let mut p = Problem::default();
        for s in req_arr(&v, "states", "problem")? {
            let id = str_field(s, "id", "state")?;
            let mut facts = BTreeSet::new();
            for f in opt_arr(s, "facts", &format!("state `{id}`"))? {
                facts.insert(
                    f.as_str()
                        .ok_or_else(|| malformed(format!("state `{id}` facts: non-string")))?
                        .to_string(),
                );
            }
            p.states.insert(id.to_string(), facts);
        }
        for s in req_arr(&v, "initial_states", "problem")? {
            p.initial.push(
                s.as_str()
                    .ok_or_else(|| malformed("initial_states: non-string".into()))?
                    .to_string(),
            );
        }
        let goal = v
            .get("goal")
            .ok_or_else(|| malformed("problem: missing `goal`".into()))?;
        for f in req_arr(goal, "facts", "goal")? {
            p.goal_facts.insert(
                f.as_str()
                    .ok_or_else(|| malformed("goal.facts: non-string".into()))?
                    .to_string(),
            );
        }
        for s in opt_arr(&v, "unsafe_states", "problem")? {
            p.unsafe_states.insert(
                s.as_str()
                    .ok_or_else(|| malformed("unsafe_states: non-string".into()))?
                    .to_string(),
            );
        }
        for t in req_arr(&v, "transitions", "problem")? {
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
            for o in req_arr(e, "outcomes", "entry")? {
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
