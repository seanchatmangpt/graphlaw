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
//!
//! WP P07 added the remaining hard physical caps: total shapes bytes
//! ([`MAX_SHACL_SHAPES_BYTES`]), parsed quad count ([`MAX_SHACL_QUADS`]),
//! and per-shape link out-degree ([`MAX_SHACL_DEGREE`]) — each value picked
//! from the measured boundary cliff, not guessed. [`check_shape_limits`]
//! enforces all four in O(1)-to-O(graph) order (bytes before parse); the
//! legacy [`check_shape_depth`] now delegates to it (no bypass) for
//! the shipped depth court.

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

/// Maximum total byte size of a shapes graph admitted through
/// [`crate::law::Step::AdmitShacl`] / [`crate::sql_ast::validate_ast_shacl`]
/// / the ABI `shacl` op. Crossing it is a typed `ResourceLimit` refusal
/// BEFORE any parse work — the byte check is O(1) and precedes the parse in
/// [`check_shape_limits`].
///
/// MEASURED (macOS aarch64, `benches/shacl_boundary_cliff.rs` run
/// 2026-10-26, `benches/reports/shacl_boundary_cliff.md`): the raw engine
/// (parse_shapes + validate_dataset) p99 crosses the 5 ms Art. 14 leg
/// budget at the 64 KiB cell (13.5 ms) on the independent-shapes blob; at
/// 16 MiB it reaches 4.93 s and the guard's own walk 1.15 s — the cliff the
/// byte cap exists to eliminate. 128 KiB is the smallest power-of-two grid
/// point that clears the shipped trusted canonical surface
/// (`ontologies/sql-ast.ttl` + `ggen/sql-dialect/statement_classes.ttl`
/// concat = 66,181 bytes, fed on every `compile_turtle`) with ~2x margin.
/// At-cap admit therefore records an honest worst-case cost above the
/// hot-leg 5 ms ceiling (measured in the adversarial-at-caps table); the
/// Art. 14 statutory candidate class (KB-class shapes) sits three decades
/// under the cap and validates in microseconds.
pub const MAX_SHACL_SHAPES_BYTES: usize = 128 * 1024;

/// Maximum shape-link out-degree of a single shape (widest
/// `sh:node`/`sh:property`/`sh:not`/`sh:qualifiedValueShape` child set,
/// counting `sh:and`/`sh:or`/`sh:xone` list members).
///
/// MEASURED: engine p99 crosses 5 ms at degree 512 (9.2 ms); at degree 128
/// it is 2.9 ms — under the Art. 14 ceiling with the cap 4x below the first
/// crossing cell. The guard's own walk also passes 5 ms by degree 1024
/// (10.0 ms p99), so the cap protects the boundary check itself.
pub const MAX_SHACL_DEGREE: usize = 128;

/// Maximum quads in the parsed shapes dataset. Bounds the guard's own
/// adjacency build + DFS regardless of the byte cap (a dense tiny-syntax
/// graph can carry more quads than bytes). MEASURED: the 64 KiB blob cell
/// carries 3,325 quads; 16,384 sits ~5x above the largest under-bytes-cap
/// blob and ~4x above the trusted canonical concat's quad count (witnessed
/// in the boundary-cliff court), while staying 2x under the old
/// unmeasured placeholder of 32,768 — which per the table would admit
/// ~256 KiB-class graphs costing 67 ms engine p99.
pub const MAX_SHACL_QUADS: usize = 16_384;

/// Expansion budget for the depth search. The search explores shape-link
/// paths up to `MAX + 1` edges; a pathological shapes graph with an enormous
/// number of distinct shape-link paths could otherwise make the guard itself
/// a denial-of-service. Exceeding the budget is a fail-closed `ResourceLimit`
/// refusal.
pub const EXPANSION_BUDGET: u64 = 200_000;

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

/// Hard physical caps on a shapes graph at the admission boundary. The
/// cliffs they sit on are MEASURED, not guessed — see
/// `benches/shacl_boundary_cliff.rs` /
/// `benches/reports/shacl_boundary_cliff.md` for the p50/p95/p99 sweep that
/// justified each value (WP P07).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShaclLimitExceeded {
    /// `shapes_ttl.len()` crossed [`MAX_SHACL_SHAPES_BYTES`].
    Bytes {
        /// Observed byte length of the shapes graph text.
        observed: usize,
    },
    /// Quad count of the parsed shapes dataset crossed [`MAX_SHACL_QUADS`].
    Quads {
        /// Observed quad count.
        observed: usize,
    },
    /// Shape-link out-degree of one shape crossed [`MAX_SHACL_DEGREE`] (the
    /// widest `sh:node`/`sh:property`/`sh:not`/`sh:qualifiedValueShape`
    /// child set, counting `sh:and`/`sh:or`/`sh:xone` list members).
    Degree {
        /// Observed out-degree of the widest shape.
        observed: usize,
    },
    /// Shape-nesting depth crossed [`MAX_SHACL_VALIDATION_DEPTH`] (or the
    /// expansion budget ran out — fail closed).
    Depth {
        /// Observed nesting depth.
        observed: usize,
    },
    /// The shapes graph did not parse under the engine's own parser. Not a
    /// size cap: callers fall through to `parse_shapes` for the engine's typed
    /// parse refusal (see [`ShaclLimitExceeded::is_parse_failure`]).
    Unparseable {
        /// Number of parse errors reported.
        observed: usize,
    },
    /// The guard's own depth-search expansion budget ([`EXPANSION_BUDGET`])
    /// ran out before boundedness could be proven — fail closed, reported as
    /// its own limit rather than a fabricated depth.
    ExpansionBudget {
        /// Expansions consumed when the budget ran out.
        observed: usize,
    },
}

impl ShaclLimitExceeded {
    /// The typed limit name, shared by every call-site rendering (`law.rs`
    /// violation component, ABI `ResourceLimit` code, sql-rdf refusal
    /// message): `shacl_validation_{bytes,quads,degree,depth}`.
    pub fn limit_name(&self) -> &'static str {
        match self {
            Self::Bytes { .. } => "shacl_validation_bytes",
            Self::Quads { .. } => "shacl_validation_quads",
            Self::Degree { .. } => "shacl_validation_degree",
            Self::Depth { .. } => "shacl_validation_depth",
            Self::ExpansionBudget { .. } => "shacl_validation_expansion_budget",
            Self::Unparseable { .. } => "shacl_validation_unparseable",
        }
    }

    /// True when the guard refused because the engine's parser rejects the
    /// shapes graph; the caller's `parse_shapes` then yields the typed
    /// engine refusal.
    pub fn is_parse_failure(&self) -> bool {
        matches!(self, Self::Unparseable { .. })
    }

    /// Observed value that crossed the cap.
    pub fn observed(&self) -> usize {
        match self {
            Self::Bytes { observed }
            | Self::Quads { observed }
            | Self::Degree { observed }
            | Self::Depth { observed }
            | Self::Unparseable { observed }
            | Self::ExpansionBudget { observed } => *observed,
        }
    }

    /// The cap that was crossed.
    pub fn cap(&self) -> usize {
        match self {
            Self::Bytes { .. } => MAX_SHACL_SHAPES_BYTES,
            Self::Quads { .. } => MAX_SHACL_QUADS,
            Self::Degree { .. } => MAX_SHACL_DEGREE,
            Self::Depth { .. } => MAX_SHACL_VALIDATION_DEPTH,
            Self::Unparseable { .. } => 0,
            Self::ExpansionBudget { .. } => EXPANSION_BUDGET as usize,
        }
    }
}

impl std::fmt::Display for ShaclLimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SHACL shapes-graph {} exceeded: observed {} > max {} (failing closed)",
            self.limit_name().trim_start_matches("shacl_validation_"),
            self.observed(),
            self.cap()
        )
    }
}

impl std::error::Error for ShaclLimitExceeded {}

/// Legacy entry point. Routes through [`check_shape_limits`] so no entry
/// point bypasses the byte/quad/degree/depth/expansion caps.
pub fn check_shape_depth(shapes_ttl: &str) -> Result<(), ShaclLimitExceeded> {
    check_shape_limits(shapes_ttl)
}

/// Refuse `shapes_ttl` at the first hard physical cap it crosses: bytes
/// ([`MAX_SHACL_SHAPES_BYTES`], O(1) before any parse) -> quads
/// ([`MAX_SHACL_QUADS`]) -> degree ([`MAX_SHACL_DEGREE`]) -> depth
/// ([`MAX_SHACL_VALIDATION_DEPTH`]). Every crossing is a typed
/// [`ShaclLimitExceeded`] — fail closed, never a stack overflow, never an
/// unbounded engine call.
pub fn check_shape_limits(shapes_ttl: &str) -> Result<(), ShaclLimitExceeded> {
    check_shape_limits_with_base(shapes_ttl, None)
}

/// [`check_shape_limits`] parsing with the same `base` IRI the caller's
/// later `parse_shapes` uses, so relative-IRI shapes graphs that only parse
/// under a base are measured, not skipped.
pub fn check_shape_limits_with_base(
    shapes_ttl: &str,
    base: Option<&str>,
) -> Result<(), ShaclLimitExceeded> {
    if shapes_ttl.len() > MAX_SHACL_SHAPES_BYTES {
        return Err(ShaclLimitExceeded::Bytes {
            observed: shapes_ttl.len(),
        });
    }
    // ONE parse function: the exact one `parse_shapes` runs
    // (`text_ingest::parse_turtle_document`, same text, same base). A parse
    // failure is never `Ok` -- that was a parser differential letting inputs
    // skip every cap. It is reported as `Unparseable`; callers fall through
    // to `parse_shapes`, which fails identically with the engine's own typed
    // refusal.
    match parse_shapes_for_guard(shapes_ttl, base) {
        Ok(ds) => check_dataset_inner(&ds),
        Err(errors) => Err(ShaclLimitExceeded::Unparseable {
            observed: errors.len().max(1),
        }),
    }
}

/// The single shapes-graph parse shared by the guard and the engine: exactly
/// the function and arguments `purrdf::shapes::engine::parse_shapes` uses.
pub fn parse_shapes_for_guard(
    shapes_ttl: &str,
    base: Option<&str>,
) -> Result<std::sync::Arc<RdfDataset>, Vec<String>> {
    purrdf::shapes::text_ingest::parse_turtle_document(shapes_ttl, base).map(|d| d.dataset)
}

/// Build the shape-link adjacency over `ds` and refuse at the first cap the
/// profile enforces.
fn check_dataset_inner(ds: &RdfDataset) -> Result<(), ShaclLimitExceeded> {
    let quads = ds.quads().count();
    if quads > MAX_SHACL_QUADS {
        return Err(ShaclLimitExceeded::Quads { observed: quads });
    }

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

    // Degree cap: the widest expanded child set of any shape. Checked on the
    // expanded map so list members count individually (AllLimits only — the
    for children in edges.values() {
        if children.len() > MAX_SHACL_DEGREE {
            return Err(ShaclLimitExceeded::Degree {
                observed: children.len(),
            });
        }
    }

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

/// Depth-first bounded path search. Returns `Err` once a simple path longer
/// than `MAX_SHACL_VALIDATION_DEPTH` edges exists.
fn dfs(
    node: TermId,
    depth: usize,
    edges: &BTreeMap<TermId, Vec<TermId>>,
    on_path: &mut BTreeSet<TermId>,
    budget: &mut u64,
) -> Result<(), ShaclLimitExceeded> {
    if depth > MAX_SHACL_VALIDATION_DEPTH {
        return Err(ShaclLimitExceeded::Depth { observed: depth });
    }
    if !on_path.insert(node) {
        return Ok(()); // revisit within one path: cut (validator tolerates it)
    }
    if let Some(children) = edges.get(&node) {
        for &child in children {
            if *budget == 0 {
                // Fail closed: cannot prove the graph is bounded within
                // budget; typed as its own limit, not a fabricated depth.
                return Err(ShaclLimitExceeded::ExpansionBudget {
                    observed: EXPANSION_BUDGET as usize,
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
