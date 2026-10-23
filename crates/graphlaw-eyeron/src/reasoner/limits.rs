use super::ReasonerError;
use crate::ast::*;
use std::collections::{BTreeSet, HashMap, HashSet};

// ---- reasoner.rs lines 41-41 ----
pub(crate) const DEFAULT_MAX_BACKWARD_DEPTH: usize = 32;

// ---- reasoner.rs lines 43-51 ----
pub(crate) const DEFAULT_MAX_MATCH_STEPS: usize = 200_000;
/// Default cap on the term text a single derived fact may carry, in bytes.
///
/// A rule such as `{ :a :s ?x . (?x ?x) string:concatenation ?y } => { :a :s ?y }`
/// doubles a literal on every firing, so it exhausts memory long before any
/// step counter runs out.  Bounding the size of a derived fact turns that
/// from an out-of-memory kill into an ordinary incomplete run.  16 MiB is far
/// larger than any fact real rule sets build and small enough that rejecting
/// one costs nothing.
// ---- reasoner.rs lines 52-52 ----
pub(crate) const DEFAULT_MAX_TERM_BYTES: usize = 16 * 1024 * 1024;

// ---- reasoner.rs lines 53-53 ----
pub(crate) const DEFAULT_MAX_TOTAL_BYTES: usize = 512 * 1024 * 1024;

// ---- reasoner.rs lines 54-54 ----
pub(crate) const DEFAULT_MAX_CLOSURE_FACTS: usize = 5_000_000;

// ---- reasoner.rs lines 55-57 ----
pub(crate) const DEFAULT_MAX_TOTAL_STEPS: usize = 500_000_000;
/// Cap on search nodes visited, and on solutions kept, by one `log:includes` or
/// `list:append` evaluation: both enumerate a cartesian space with no other bound.
// ---- reasoner.rs lines 58-58 ----
pub(crate) const MAX_BUILTIN_SEARCH_NODES: usize = 1_000_000;

// ---- reasoner.rs lines 59-59 ----
pub(crate) const MAX_BUILTIN_SOLUTIONS: usize = 10_000;

// ---- reasoner.rs lines 60-62 ----
pub(crate) const MAX_TABLED_BACKWARD_GOALS: usize = 100_000;
/// Fixed cost charged per derived fact on top of its term text: the fact is held in the
/// closure, the derived list and the dedup/index structures.
// ---- reasoner.rs lines 63-64 ----
pub(crate) const PER_FACT_OVERHEAD_BYTES: usize = 256;
/// Longest integer lexical form parsed to an exact `BigInt` (parsing is quadratic).
// ---- reasoner.rs lines 65-67 ----
pub(crate) const MAX_EXACT_LITERAL_CHARS: usize = 100_000;
/// Largest exact `math:product` result, in bits (about `MAX_EXACT_LITERAL_CHARS` digits), so a
/// product never yields a literal the next operation would re-parse inexactly.
// ---- reasoner.rs lines 68-72 ----
pub(crate) const MAX_EXACT_PRODUCT_BITS: u64 = 330_000;
// Multi-premise agenda matching is a win for small state-machine examples,
// but on generated rule sets such as deep-taxonomy-100000 it makes every
// broad subject/predicate fact probe unrelated multi-premise checks.  Keep
// the original single-premise hot path for large programs.

// ---- reasoner.rs lines 73-75 ----
pub(crate) const MULTI_PREMISE_AGENDA_RULE_LIMIT: usize = 2048;

#[derive(Debug, Clone)]
// ---- reasoner.rs lines 76-93 ----
pub(crate) struct SearchBudget {
    pub(crate) steps: usize,
    pub(crate) nested_match_steps: usize,
    pub(crate) max_iterations: usize,
    pub(crate) max_steps: usize,
    pub(crate) max_backward_depth: usize,
    pub(crate) max_backward_solutions_per_goal: usize,
    pub(crate) max_term_bytes: usize,
    pub(crate) max_total_bytes: usize,
    pub(crate) max_closure_facts: usize,
    pub(crate) max_total_steps: usize,
    pub(crate) limits_reached: BTreeSet<ReasonerLimit>,
    pub(crate) errors: Vec<ReasonerError>,
    pub(crate) error_seen: HashSet<ReasonerError>,
    pub(crate) completed_backward_goals: HashMap<String, Vec<Triple>>,
    pub(crate) next_variable_scope: usize,
}

/// A safety limit that prevented the reasoner from proving a complete fixpoint.
///
/// Variants are added as new limits are introduced; downstream `match` needs a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ReasonerLimit {
    Iterations,
    MatchSteps,
    BackwardDepth,
    BackwardSolutionsPerGoal,
    TermSize,
    TermDepth,
    ClosureSize,
    ClosureFacts,
}
