//! Knowledge hooks (`kh:`): RDF-described, SPARQL-driven derivation rules.
//!
//! A hook pack is ordinary RDF (see `packs/self-monitoring-pack/hook.ttl`):
//!
//! ```turtle
//! smon:h a kh:Hook ; kh:name "..." ; kh:kind "sparql" ;
//!     kh:query "SELECT ..." ;              # trigger
//!     kh:effect "emit-delta" ;
//!     kh:action smon:a ; kh:priority 1 .
//! smon:a a kh:Action ;
//!     kh:handler <http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct> ;
//!     kh:query "CONSTRUCT ... WHERE ..." .
//! ```
//!
//! GraphLaw owns only the orchestration; every query is parsed and executed by
//! PurRDF's SPARQL engine, and the pack itself is read by querying it.
//!
//! Effect vocabulary. `kh:effect "emit-delta"` merges the action's CONSTRUCT
//! into the state; `kh:effect "refuse"` emits no delta — instead each firing
//! surfaces a [`Verdict::Refuse`] carrying the hook's `kh:reason` (default:
//! its `kh:name`) in [`Materialized::verdicts`]. A refuse hook is a named,
//! pack-declared fuse: it is surfaced as a verdict, never an error, and the
//! consumer decides what to refuse.
//!
//! Semantics. A hook fires once per *distinct solution row* of its trigger
//! `SELECT`. Firing runs the action's `CONSTRUCT` with that row's variables
//! pre-bound, and the constructed triples are merged into the state (each
//! firing mints its own blank nodes). Hooks run in ascending `kh:priority`
//! (ties by IRI) and rounds repeat until no new row appears, so the result is a
//! fixpoint. Each firing also records a deterministic marker
//! (`<urn:graphlaw:hook-firing:blake3> a kh:Firing ; kh:hook <hook>`) in the
//! state, so a row fires at most once *per state lineage*: materializing an
//! already-saturated state changes nothing, and blank-node minting cannot make
//! a stable pack diverge or duplicate. Unknown `kh:kind`, `kh:effect` or handler is refused
//! rather than skipped; exceeding [`MAX_ROUNDS`] or [`MAX_FIRINGS`] is refused
//! rather than truncated.

use std::collections::BTreeSet;

use purrdf::{RdfDatasetBuilder, SparqlEngine, SparqlRequest, SparqlResult, TermValue};

use crate::dialect::{Dialect, Refusal, RefusalKind, RefusalMessage};
use crate::law::LawState;
use purrdf::sparql::NativeSparqlEngine;

/// The `kh:` vocabulary namespace.
pub const KH: &str = "http://seanchatmangpt.github.io/graphlaw/kh#";
/// The only supported action handler.
pub const HANDLER_SPARQL_CONSTRUCT: &str =
    "http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct";
/// Hard ceiling on fixpoint rounds.
pub const MAX_ROUNDS: usize = 64;
/// Hard ceiling on total hook firings.
pub const MAX_FIRINGS: usize = 10_000;
/// Largest dataset, in quads, a pack may grow the state to. `MAX_FIRINGS` bounds
/// how often hooks fire, not how much each CONSTRUCT adds.
pub const MAX_STATE_QUADS: usize = 1_000_000;

/// What a hook firing does, from `kh:effect`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Merge the action's `CONSTRUCT` into the state (`kh:effect "emit-delta"`).
    EmitDelta,
    /// Emit no delta; surface a [`Verdict::Refuse`] instead
    /// (`kh:effect "refuse"`).
    Refuse,
}

impl Effect {
    fn parse(iri: &str, value: &str) -> Result<Self, Refusal> {
        match value {
            "emit-delta" => Ok(Effect::EmitDelta),
            "refuse" => Ok(Effect::Refuse),
            other => Err(refuse(
                RefusalKind::Unsupported,
                format!("{iri}: unsupported kh:effect {other:?}"),
            )),
        }
    }
}

/// The verdict one hook firing produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The firing merged its CONSTRUCT delta into the state.
    Fired,
    /// The firing refuses the actuation; the payload is the refuse reason
    /// (`kh:reason`, defaulting to the hook's `kh:name`).
    Refuse(String),
}

/// One hook firing's verdict, in firing order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookVerdict {
    /// IRI of the hook that fired.
    pub hook: String,
    /// Fixpoint round in which the hook fired.
    pub round: usize,
    /// The trigger row, as `(variable, term)` in projection order.
    pub row: Vec<(String, String)>,
    /// What the firing did.
    pub verdict: Verdict,
}

/// One `kh:Hook` with its resolved action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hook {
    /// IRI identifying the hook.
    pub iri: String,
    /// Human-readable hook name.
    pub name: String,
    /// Trigger `SELECT`.
    pub trigger: String,
    /// Action `CONSTRUCT`.
    pub construct: String,
    /// Firing priority; lower values fire first.
    pub priority: i64,
    /// What a firing of this hook does.
    pub effect: Effect,
    /// Refuse reason for `Effect::Refuse` hooks (`kh:reason`; `None` falls
    /// back to [`Hook::name`] in the verdict).
    pub reason: Option<String>,
}

/// A validated, priority-ordered set of hooks.
#[derive(Debug, Clone, Default)]
pub struct HookPack {
    hooks: Vec<Hook>,
}

/// One firing of one hook for one trigger row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Firing {
    /// IRI of the hook that fired.
    pub hook: String,
    /// Fixpoint round in which the hook fired.
    pub round: usize,
    /// The trigger row, as `(variable, term)` in projection order.
    pub row: Vec<(String, String)>,
    /// Quads this firing added to the state.
    pub added: usize,
}

/// The fixpoint of a pack over a state.
#[derive(Debug, Clone)]
pub struct Materialized {
    /// The state after all hooks reached fixpoint.
    pub state: LawState,
    /// Every firing, in order.
    pub firings: Vec<Firing>,
    /// One verdict per firing, in the same order as [`Materialized::firings`].
    pub verdicts: Vec<HookVerdict>,
    /// Number of rounds executed.
    pub rounds: usize,
}

fn refuse(kind: RefusalKind, message: impl Into<String>) -> Refusal {
    Refusal {
        kind,
        dialect: Some(Dialect::Sparql),
        engine: Some(crate::dialect::Engine::PurRdf),
        message: RefusalMessage::Dynamic(message.into()),
    }
}

fn select(
    state: &LawState,
    query: &str,
    subs: &[(String, TermValue)],
) -> Result<Vec<Vec<Option<TermValue>>>, Refusal> {
    match NativeSparqlEngine::new()
        .query(
            state.dataset(),
            SparqlRequest {
                query,
                base_iri: None,
                substitutions: subs,
            },
        )
        .map_err(|e| Refusal::engine(Dialect::Sparql, e))?
    {
        SparqlResult::Solutions { rows, .. } => Ok(rows),
        other => Err(refuse(
            RefusalKind::Unsupported,
            format!("expected SELECT solutions, got {other:?}"),
        )),
    }
}

fn lexical(t: &Option<TermValue>) -> Option<String> {
    match t {
        Some(TermValue::Literal { lexical_form, .. }) => Some(lexical_form.clone()),
        Some(TermValue::Iri(i)) => Some(i.clone()),
        _ => None,
    }
}

fn local_name(iri: &str) -> String {
    iri.rsplit(['#', '/']).next().unwrap_or(iri).to_string()
}

impl HookPack {
    /// Read every `kh:Hook` in `pack`. A hook that cannot be fully resolved
    /// (missing action, query, kind or effect) is refused, never dropped.
    pub fn load(pack: &LawState) -> Result<Self, Refusal> {
        let declared = select(
            pack,
            &format!("SELECT DISTINCT ?h WHERE {{ ?h a <{KH}Hook> }}"),
            &[],
        )?;
        let rows = select(
            pack,
            &format!(
                "SELECT DISTINCT ?h ?name ?kind ?q ?effect ?handler ?aq ?prio ?reason WHERE {{ \
                   ?h a <{KH}Hook> ; <{KH}kind> ?kind ; <{KH}query> ?q ; \
                      <{KH}effect> ?effect ; <{KH}action> ?act . \
                   ?act a <{KH}Action> ; <{KH}handler> ?handler ; <{KH}query> ?aq . \
                   OPTIONAL {{ ?h <{KH}name> ?name }} \
                   OPTIONAL {{ ?h <{KH}priority> ?prio }} \
                   OPTIONAL {{ ?h <{KH}reason> ?reason }} }}"
            ),
            &[],
        )?;
        let mut hooks = Vec::new();
        for r in &rows {
            let iri = lexical(&r[0])
                .ok_or_else(|| refuse(RefusalKind::Unsupported, "kh:Hook must be an IRI"))?;
            let kind = lexical(&r[2]).unwrap_or_default();
            let effect = lexical(&r[4]).unwrap_or_default();
            let handler = lexical(&r[5]).unwrap_or_default();
            if kind != "sparql" {
                return Err(refuse(
                    RefusalKind::Unsupported,
                    format!("{iri}: unsupported kh:kind {kind:?}"),
                ));
            }
            let effect = Effect::parse(&iri, &effect)?;
            if handler != HANDLER_SPARQL_CONSTRUCT {
                return Err(refuse(
                    RefusalKind::Unsupported,
                    format!("{iri}: unsupported kh:handler <{handler}>"),
                ));
            }
            let priority = match &r[7] {
                Some(TermValue::Literal { lexical_form, .. }) => {
                    lexical_form.parse().map_err(|_| {
                        refuse(
                            RefusalKind::Unsupported,
                            format!("{iri}: kh:priority is not an integer"),
                        )
                    })?
                }
                _ => 0,
            };
            hooks.push(Hook {
                name: lexical(&r[1]).unwrap_or_else(|| local_name(&iri)),
                iri,
                trigger: lexical(&r[3]).unwrap_or_default(),
                construct: lexical(&r[6]).unwrap_or_default(),
                priority,
                effect,
                reason: lexical(&r[8]),
            });
        }
        if hooks.len() != declared.len() {
            return Err(refuse(
                RefusalKind::Unsupported,
                format!(
                    "{} kh:Hook(s) declared but {} fully resolved (each needs kh:kind, kh:query, kh:effect and a kh:Action with kh:handler and kh:query)",
                    declared.len(),
                    hooks.len()
                ),
            ));
        }
        hooks.sort_by(|a, b| (a.priority, &a.iri).cmp(&(b.priority, &b.iri)));
        // Reject queries PurRDF cannot parse now, not at first firing.
        for h in &hooks {
            let parser = purrdf::sparql::SparqlParser::new();
            parser.parse_query(&h.trigger).map_err(|e| {
                Refusal::engine(Dialect::Sparql, format!("{}: trigger: {e:?}", h.iri))
            })?;
            parser.parse_query(&h.construct).map_err(|e| {
                Refusal::engine(Dialect::Sparql, format!("{}: action: {e:?}", h.iri))
            })?;
        }
        Ok(HookPack { hooks })
    }

    /// The hooks of this pack, in priority order.
    pub fn hooks(&self) -> &[Hook] {
        &self.hooks
    }

    /// Run the pack to a fixpoint over `state`.
    pub fn materialize(&self, state: &LawState) -> Result<Materialized, Refusal> {
        let mut current = state.clone();
        let mut fired: BTreeSet<String> = select(
            state,
            &format!("SELECT ?f WHERE {{ ?f a <{KH}Firing> }}"),
            &[],
        )?
        .iter()
        .filter_map(|r| lexical(&r[0]))
        .collect();
        let mut firings = Vec::new();
        let mut verdicts: Vec<HookVerdict> = Vec::new();
        for round in 1..=MAX_ROUNDS {
            let mut progressed = false;
            for hook in &self.hooks {
                let (vars, rows) = trigger_solutions(&current, &hook.trigger)?;
                for row in rows {
                    let bound: Vec<(String, TermValue)> = vars
                        .iter()
                        .zip(row)
                        .filter_map(|(v, t)| t.map(|t| (v.clone(), t)))
                        .collect();
                    let marker = firing_marker(&hook.iri, &bound);
                    // Refuse hooks are fuses and always evaluate: a caller-supplied
                    // `kh:Firing` marker can never pre-suppress them. For other
                    // effects the seeded markers are ADVISORY idempotency only
                    // (no receipt-attested prior-state notion exists here).
                    let seen = !fired.insert(marker.clone());
                    if seen && hook.effect != Effect::Refuse {
                        continue;
                    }
                    if firings.len() >= MAX_FIRINGS {
                        return Err(refuse(
                            RefusalKind::Unsupported,
                            format!("hook pack exceeded {MAX_FIRINGS} firings"),
                        ));
                    }
                    progressed = true;
                    // A refuse firing is a fuse: it halts the transaction. The
                    // typed refusal is returned and no state is committed —
                    // `state` is borrowed and `current` is dropped, so the
                    // caller's prior state is unchanged.
                    if hook.effect == Effect::Refuse {
                        let reason = hook.reason.clone().unwrap_or_else(|| hook.name.clone());
                        return Err(refuse(
                            RefusalKind::Unsupported,
                            format!("hook-refused: {}: {reason}", hook.iri),
                        ));
                    }
                    let mut b = RdfDatasetBuilder::new();
                    b.push_dataset(current.dataset())
                        .map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
                    match NativeSparqlEngine::new()
                        .query(
                            current.dataset(),
                            SparqlRequest {
                                query: &hook.construct,
                                base_iri: None,
                                substitutions: &bound,
                            },
                        )
                        .map_err(|e| Refusal::engine(Dialect::Sparql, e))?
                    {
                        SparqlResult::Graph(g) => b
                            .push_dataset(&g)
                            .map_err(|e| Refusal::engine(Dialect::NQuads, e))?,
                        other => {
                            return Err(refuse(
                                RefusalKind::Unsupported,
                                format!("{}: action must be CONSTRUCT, got {other:?}", hook.iri),
                            ));
                        }
                    }
                    let (f, ty, class, hook_p, hook_o) = (
                        b.intern_iri(&marker),
                        b.intern_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                        b.intern_iri(&format!("{KH}Firing")),
                        b.intern_iri(&format!("{KH}hook")),
                        b.intern_iri(&hook.iri),
                    );
                    let (f, ty, class, hook_p, hook_o) = (
                        f.map_err(|e| Refusal::engine(Dialect::NQuads, e))?,
                        ty.map_err(|e| Refusal::engine(Dialect::NQuads, e))?,
                        class.map_err(|e| Refusal::engine(Dialect::NQuads, e))?,
                        hook_p.map_err(|e| Refusal::engine(Dialect::NQuads, e))?,
                        hook_o.map_err(|e| Refusal::engine(Dialect::NQuads, e))?,
                    );
                    b.push_quad(f, ty, class, None)
                        .map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
                    b.push_quad(f, hook_p, hook_o, None)
                        .map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
                    let merged = b
                        .freeze()
                        .map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
                    let next = LawState::from_dataset(merged)?;
                    if next.quad_count() > MAX_STATE_QUADS {
                        return Err(refuse(
                            RefusalKind::ResourceLimit,
                            format!(
                                "resource limit `state_quads` exceeded: {} > {MAX_STATE_QUADS}",
                                next.quad_count()
                            ),
                        ));
                    }
                    let row: Vec<(String, String)> = bound
                        .iter()
                        .map(|(v, t)| (v.clone(), canonical_term(t)))
                        .collect();
                    firings.push(Firing {
                        hook: hook.iri.clone(),
                        round,
                        row: row.clone(),
                        added: next.quad_count().saturating_sub(current.quad_count()),
                    });
                    verdicts.push(HookVerdict {
                        hook: hook.iri.clone(),
                        round,
                        row,
                        verdict: Verdict::Fired,
                    });
                    current = next;
                }
            }
            if !progressed {
                return Ok(Materialized {
                    state: current,
                    firings,
                    verdicts,
                    rounds: round,
                });
            }
        }
        Err(refuse(
            RefusalKind::Unsupported,
            format!("hook pack did not reach a fixpoint in {MAX_ROUNDS} rounds"),
        ))
    }
}

fn put(out: &mut String, tag: char, text: &str) {
    // length-prefixed: no separator collision is possible
    out.push(tag);
    out.push_str(&text.len().to_string());
    out.push(':');
    out.push_str(text);
}

fn encode_term(out: &mut String, t: &TermValue) {
    match t {
        TermValue::Iri(i) => put(out, 'I', i),
        TermValue::Blank { label, scope } => {
            put(out, 'B', label);
            put(out, 'S', &scope.ordinal().to_string());
        }
        TermValue::Literal {
            lexical_form,
            datatype,
            language,
            direction,
        } => {
            put(out, 'L', lexical_form);
            put(out, 'D', datatype);
            put(out, 'G', language.as_deref().unwrap_or(""));
            put(out, 'R', direction.map_or("", |d| d.as_str()));
        }
        TermValue::Triple { s, p, o } => {
            out.push('T');
            encode_term(out, &**s);
            encode_term(out, &**p);
            encode_term(out, &**o);
            out.push('.');
        }
    }
}

/// Canonical, explicit encoding of one term (no `Debug`).
pub fn canonical_term(t: &TermValue) -> String {
    let mut out = String::new();
    encode_term(&mut out, t);
    out
}

/// The idempotency marker IRI of one hook firing: BLAKE3 over the hook IRI and
/// the bound variables sorted by name, each explicitly length-prefix encoded.
/// Independent of projection/row order and of Rust `Debug` formatting.
pub fn firing_marker(hook_iri: &str, bound: &[(String, TermValue)]) -> String {
    let mut vars: Vec<(&String, String)> =
        bound.iter().map(|(v, t)| (v, canonical_term(t))).collect();
    vars.sort();
    let mut buf = String::new();
    put(&mut buf, 'H', hook_iri);
    for (v, t) in vars {
        put(&mut buf, 'V', v);
        buf.push_str(&t);
    }
    format!(
        "urn:graphlaw:hook-firing:{}",
        blake3::Hash::from_bytes(blake3::derive_key(
            "graphlaw hooks firing v1",
            buf.as_bytes()
        ))
        .to_hex()
    )
}

/// Run a trigger SELECT exactly once, returning its variables and rows together.
pub fn trigger_solutions(
    state: &LawState,
    trigger: &str,
) -> Result<(Vec<String>, Vec<Vec<Option<TermValue>>>), Refusal> {
    match NativeSparqlEngine::new()
        .query(
            state.dataset(),
            SparqlRequest {
                query: trigger,
                base_iri: None,
                substitutions: &[],
            },
        )
        .map_err(|e| Refusal::engine(Dialect::Sparql, e))?
    {
        SparqlResult::Solutions {
            variables, rows, ..
        } => Ok((variables, rows)),
        other => Err(refuse(
            RefusalKind::Unsupported,
            format!("trigger must be SELECT, got {other:?}"),
        )),
    }
}
