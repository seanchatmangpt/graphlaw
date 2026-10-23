//! Bounded SHACL shape-nesting guard for [`crate::law::Step::AdmitShacl`].
//!
//! The retired `graphlaw` path dependency carried
//! `MAX_SHACL_VALIDATION_DEPTH` inside its own SHACL evaluator; when the
//! crates.io `graphlaw` crate replaced it, SHACL evaluation became a pure
//! re-export of `purrdf::shapes` — whose validator recurses over `sh:node` /
//! `sh:property` shape chains with revisit-only cycle detection and **no
//! acyclic-chain depth bound**. A shapes graph with a long acyclic chain of
//! shapes therefore stack-overflows the process (SIGABRT, not a catchable
//! panic). This module restores the lost bound at the graphlaw boundary:
//! before a shapes graph is handed to the purrdf validator, its shape-link
//! nesting depth is computed by a search whose own recursion is bounded at
//! `MAX_SHACL_VALIDATION_DEPTH + 2` frames, and graphs deeper than the cap
//! are refused with a typed
//! [`crate::dialect::RefusalKind::ResourceLimit`] refusal — fail closed.

use std::collections::{BTreeMap, BTreeSet};

use purrdf::{RdfDataset, TermId, TermValue};

/// Maximum SHACL shape-nesting depth admitted through
/// [`crate::law::Step::AdmitShacl`] (`sh:node` / `sh:property` / `sh:not` /
/// `sh:qualifiedValueShape` chains, plus `sh:and`/`sh:or`/`sh:xone` list
/// members). Crossing it is a typed `ResourceLimit` refusal, never a stack
/// overflow.
///
/// The value is deliberately conservative: each link edge costs the purrdf
/// validator roughly two recursion levels (`validate_shape` ->
/// `validate_property_shape` -> `conforms_to_shape`), and the bound must be
/// safe even on a 1 MiB thread stack (measured: 32 pure `sh:node` levels
/// validate on 1 MiB, while a chain whose nesting reaches ~48 levels
/// overflows — hence 16 edges ≈ 32 levels).
///
/// (Historical note: this constant name restores the guard of the same name
/// from the retired `graphlaw` path dependency, where it lived inside
/// the SHACL evaluator itself. The crates.io `graphlaw` line never carried
/// it — the bound was lost in the path-dep → crates.io migration.)
pub const MAX_SHACL_VALIDATION_DEPTH: usize = 16;

/// Expansion budget for the depth search. The search explores shape-link
/// paths up to `MAX + 1` edges; a pathological shapes graph with an enormous
/// number of distinct shape-link paths could otherwise make the guard itself
/// a denial-of-service. Exceeding the budget is a fail-closed `ResourceLimit`
/// refusal.
const EXPANSION_BUDGET: u64 = 200_000;

const SH: &str = "http://www.w3.org/ns/shacl#";
/// `sh:` predicates whose object is another shape the validator recurses
/// into.
const SHAPE_LINK_PREDICATES: &[&str] = &["node", "property", "not", "qualifiedValueShape"];
/// `sh:` predicates whose object is an RDF list of shapes.
const SHAPE_LIST_PREDICATES: &[&str] = &["and", "or", "xone"];

const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";

/// Why the depth guard refused a shapes graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthExceeded {
    /// Observed shape-nesting depth (link edges along the deepest simple
    /// path). Always `> MAX_SHACL_VALIDATION_DEPTH`, or `MAX + 1` when the
    /// expansion budget ran out before boundedness could be proven
    /// (fail closed).
    pub observed: usize,
}

impl std::fmt::Display for DepthExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SHACL shape nesting depth exceeded: observed {}",
            self.observed
        )
    }
}

impl std::error::Error for DepthExceeded {}

/// Refuse `shapes_ttl` if its shape-nesting depth exceeds
/// [`MAX_SHACL_VALIDATION_DEPTH`], returning the observed depth.
///
/// The guard's own recursion is bounded at `MAX_SHACL_VALIDATION_DEPTH + 2`
/// frames, so the guard can never overflow the stack on any input.
pub fn check_shape_depth(shapes_ttl: &str) -> Result<(), DepthExceeded> {
    // A parse error here is NOT a depth violation: let the real parse
    // surface from `parse_shapes` below as its own engine refusal.
    match purrdf::parse_dataset(shapes_ttl.as_bytes(), "text/turtle", None) {
        Ok(ds) => check_dataset_depth(&ds),
        Err(_) => Ok(()),
    }
}

/// Compute shape-link nesting depth over the parsed shapes graph and refuse
/// when it exceeds the cap.
fn check_dataset_depth(ds: &RdfDataset) -> Result<(), DepthExceeded> {
    // Shape-link adjacency: shape -> shapes the validator recurses into.
    let mut edges: BTreeMap<TermId, Vec<TermId>> = BTreeMap::new();
    // rdf:rest / rdf:first chains, for sh:and/sh:or/sh:xone list expansion.
    let mut rest: BTreeMap<TermId, TermId> = BTreeMap::new();
    let mut first: BTreeMap<TermId, TermId> = BTreeMap::new();

    for q in ds.quads() {
        let Some(p) = iri(ds, q.p) else {
            continue;
        };
        if let Some(local) = p.strip_prefix(SH) {
            if SHAPE_LINK_PREDICATES.contains(&local) || SHAPE_LIST_PREDICATES.contains(&local) {
                edges.entry(q.s).or_default().push(q.o);
            }
        } else if p == RDF_REST {
            rest.insert(q.s, q.o);
        } else if p == RDF_FIRST {
            first.insert(q.s, q.o);
        }
    }

    // Expand list objects in place: a `sh:and`/`sh:or`/`sh:xone` edge whose
    // object is an RDF list head contributes each list member as a child.
    let mut expanded: BTreeMap<TermId, Vec<TermId>> = BTreeMap::new();
    for (parent, children) in &mut edges {
        let mut rewritten: Vec<TermId> = Vec::with_capacity(children.len());
        for child in children.drain(..) {
            if first.contains_key(&child) {
                // Walk the rdf:rest chain iteratively; cycle-safe.
                let mut head = child;
                let mut seen = BTreeSet::new();
                loop {
                    if !seen.insert(head) {
                        break;
                    }
                    match first.get(&head) {
                        Some(item) => rewritten.push(*item),
                        None => break,
                    }
                    match rest.get(&head) {
                        Some(next) => head = *next,
                        None => break,
                    }
                }
            } else {
                rewritten.push(child);
            }
        }
        expanded.insert(*parent, rewritten);
    }
    let edges = expanded;

    // Bounded-depth path search: recursion bounded at MAX + 2 frames; cycles
    // cut by a path-local visited set (the purrdf validator tolerates shape
    // revisits, so a pure cycle must NOT be refused — only unbounded
    // *acyclic* nesting is dangerous).
    let mut budget = EXPANSION_BUDGET;
    let mut on_path: BTreeSet<TermId> = BTreeSet::new();
    for root in edges.keys().copied().collect::<Vec<_>>() {
        dfs(root, 0, &edges, &mut on_path, &mut budget)?;
    }
    Ok(())
}

/// Depth-first bounded path search. Returns `Err(observed_depth)` once a
/// simple path longer than `MAX_SHACL_VALIDATION_DEPTH` edges exists.
fn dfs(
    node: TermId,
    depth: usize,
    edges: &BTreeMap<TermId, Vec<TermId>>,
    on_path: &mut BTreeSet<TermId>,
    budget: &mut u64,
) -> Result<(), DepthExceeded> {
    if depth > MAX_SHACL_VALIDATION_DEPTH {
        return Err(DepthExceeded { observed: depth });
    }
    if !on_path.insert(node) {
        return Ok(()); // revisit within one path: cut (validator tolerates it)
    }
    if let Some(children) = edges.get(&node) {
        for &child in children {
            if *budget == 0 {
                // Fail closed: cannot prove the graph is bounded within
                // budget. Signal depth cap exceeded.
                return Err(DepthExceeded {
                    observed: MAX_SHACL_VALIDATION_DEPTH + 1,
                });
            }
            *budget -= 1;
            dfs(child, depth + 1, edges, on_path, budget)?;
        }
    }
    on_path.remove(&node);
    Ok(())
}

fn iri(ds: &RdfDataset, id: TermId) -> Option<String> {
    match ds.term_value(id) {
        TermValue::Iri(i) => Some(i),
        _ => None,
    }
}
