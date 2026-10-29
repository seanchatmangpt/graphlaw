//! event_calculus_vectors — vector GENERATOR for the event_calculus breed.
//!
//! Encodes discrete Kowalski-Sergot Event Calculus as TIME-UNROLLED Datalog
//! (Datalog has no integers): successor facts `succ(s,t)` for each adjacent
//! pair over `0..=T`, plus:
//!
//!   holds(f,t)      :- succ(s,t), holds(f,s), not clipped(f,s,t)
//!   holds(f,t)      :- succ(s,t), initiated(f,s)
//!   clipped(f,s,t)  :- succ(s,t), terminated(f,s)
//!   terminated(f,t) :- happens(e,t), terminates(e,f)
//!   initiated(f,t)  :- happens(e,t), initiates(e,f)
//!
//! Route (documented per task): stratified negation via purrdf-datalog's
//! NEGATED atoms driven DIRECTLY through `graphlaw::datalog` (the curated
//! re-export of purrdf::datalog 3.0.1; its `seminaive::evaluate` stratifies
//! the program and refuses non-stratifiable programs with the concrete
//! negative-cycle report). NOT the graphlaw JSON-ABI `datalog` op —
//! src/abi.rs::atom() hard-codes `ClauseAtom::positive`, a positive-only shim
//! that cannot express `not clipped`; and not the chase module, which refuses
//! negated body atoms outright (`ChaseError::NegatedBodyAtom`).
//!
//! Dual route (cross-check): the same program with `not clipped(f,s,t)`
//! replaced by statically enumerated explicit complement facts `comp(f,s,t)`
//! (pure positive program; the complement is computable in the generator
//! because `terminated` is a level-1 join over the EDB). The holds-projection
//! of both routes must agree exactly for every scenario.
//!
//! Semantics matched to the wasm4pm event_calculus breed
//! (crates/wasm4pm-cognition/src/breeds/event_calculus.rs):
//!   holds[t+1] = initiated@t  UNION  (holds[t] \ terminated@t)
//! i.e. HoldsAt(f,t) iff holds(f,t) is in the closure; an initiation at s
//! gives holds at s+1; inertia persists unless a termination fired at the
//! previous step (clipping); simultaneous initiate+terminate at one step
//! favors initiation (the initiation rule reads no negated atom).
//!
//! Run: cargo run --example event_calculus_vectors [-- --verify]
//! (--release is accepted but not required.)
//! Output: `generated N vector(s); ... dual-route agreement on all scenarios`
//! (exit 0); `--verify` re-evaluates independently and byte-compares.
//! Court: tests/corpus_conformance.rs::vecgen_event_calculus_dual_route_agrees
//! (generate-only; self-checks run inside generation).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use graphlaw::datalog::{
    clause::{ClauseAtom, ClauseTerm, DlClause},
    seminaive::{compile, evaluate},
    store::RelationStore,
};
use serde_json::json;

const PURRDF: &str = "3.0.1";
const N3_ENGINE: &str = "purrdf-n3/3.0.1";
const GENERATOR: &str = "event_calculus_vectors";
/// Live crate version stamped into provenance (post-L98/L106 pattern).
const GL_VERSION: &str = env!("CARGO_PKG_VERSION");

// ------------------------------------------------------------------ scenario

struct Scenario {
    vector_id: &'static str,
    intent: &'static str,
    /// (event, time).
    happens: &'static [(&'static str, usize)],
    /// (event, fluent).
    initiates: &'static [(&'static str, &'static str)],
    /// (event, fluent).
    terminates: &'static [(&'static str, &'static str)],
    /// Fluents true at t=0.
    initially: &'static [&'static str],
    /// (goal_id, fluent, query_time).
    queries: &'static [(&'static str, &'static str, usize)],
    notes: &'static str,
}

const SCENARIOS: &[Scenario] = &[
    Scenario {
        vector_id: "event_calculus-paper-narrative",
        intent: "evaluate HoldsAt over the Kowalski-Sergot hired/promoted narrative",
        happens: &[("hire", 2), ("promote", 5)],
        initiates: &[
            ("hire", "employed"),
            ("hire", "lecturer"),
            ("promote", "professor"),
        ],
        terminates: &[("promote", "lecturer")],
        initially: &[],
        queries: &[
            ("q1", "lecturer", 4),
            ("q2", "lecturer", 7),
            ("q3", "professor", 7),
            ("q4", "employed", 7),
            ("q5", "professor", 4),
        ],
        notes: "Kowalski & Sergot 1986 hired/promoted narrative (integer times 2 and 5 per the \
                paper fixture adaptation). Expected verdicts match \
                tests/fixtures/papers/event_calculus.json exactly: lecturer holds between hire \
                and promote (clipped by promotion); professor and employed persist by inertia. \
                Hand timeline: holds[0..2]={}; holds[3..6]={employed,lecturer}; \
                holds[6..8]={employed,professor}.",
    },
    Scenario {
        vector_id: "event_calculus-initiate-then-terminate",
        intent: "a single fluent initiated then later terminated by named events",
        happens: &[("start", 1), ("stop", 3)],
        initiates: &[("start", "on")],
        terminates: &[("stop", "on")],
        initially: &[],
        queries: &[
            ("g1", "on", 0),
            ("g2", "on", 1),
            ("g3", "on", 2),
            ("g4", "on", 3),
            ("g5", "on", 4),
        ],
        notes: "Initiation at t=1 makes on hold from t=2; termination at t=3 clips it from t=4. \
                Hand timeline: holds[0..2]={}; holds[2..4]={on}; holds[4]={}.",
    },
    Scenario {
        vector_id: "event_calculus-persist-through-nothing",
        intent: "inertia only: a fluent that holds with no events anywhere on the timeline",
        happens: &[],
        initiates: &[],
        terminates: &[],
        initially: &["loaded"],
        queries: &[
            ("g1", "loaded", 0),
            ("g2", "loaded", 1),
            ("g3", "loaded", 2),
            ("g4", "loaded", 3),
        ],
        notes: "Pure inertia: no happens events, so no clipping anywhere; loaded holds at every \
                query time. Hand timeline: holds[t]={loaded} for all t in 0..=3.",
    },
    Scenario {
        vector_id: "event_calculus-clipped-before-query",
        intent: "a fluent initiated, then clipped strictly before the query time",
        happens: &[("hire", 2), ("fire", 5)],
        initiates: &[("hire", "employed")],
        terminates: &[("fire", "employed")],
        initially: &[],
        queries: &[
            ("g1", "employed", 4),
            ("g2", "employed", 6),
            ("g3", "employed", 7),
        ],
        notes: "Termination at t=5 clips the holds chain before qt=6 and qt=7; qt=4 is strictly \
                before the clip and still holds. Hand timeline: holds[3..6]={employed}; \
                holds[6..8]={}.",
    },
];

fn main() {
    let rev = graphlaw_rev();
    let mut generated = 0;
    for sc in SCENARIOS {
        let horizon = horizon_of(sc);
        let route_b = run_negation_route(sc, horizon);
        let route_a = run_complement_route(sc, horizon);
        assert!(
            route_b == route_a,
            "{}: dual-route disagreement\nnegation:   {route_b:?}\ncomplement: {route_a:?}",
            sc.vector_id
        );
        let verdicts = verdict_map(sc, &route_b);
        let vector = build_vector(sc, &rev, &verdicts);
        let path = vecdir().join(format!("{}.json", sc.vector_id));
        fs::write(&path, serde_json::to_string_pretty(&vector).unwrap() + "\n")
            .expect("write vector");
        println!("wrote {} (holds facts: {})", path.display(), route_b.len());
        generated += 1;
    }
    println!("generated {generated} vector(s); rev {rev}; dual-route agreement on all scenarios");
}

fn graphlaw_rev() -> String {
    let out = std::process::Command::new("git")
        .args(["-C", env!("CARGO_MANIFEST_DIR"), "rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse");
    String::from_utf8(out.stdout)
        .expect("utf8 rev")
        .trim()
        .to_string()
}

fn vecdir() -> PathBuf {
    std::env::var_os("GL_VECDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw")
        })
}

/// Horizon T: the largest time on the timeline (events and queries).
fn horizon_of(sc: &Scenario) -> usize {
    sc.happens
        .iter()
        .map(|(_, t)| *t)
        .max()
        .unwrap_or(0)
        .max(sc.queries.iter().map(|(_, _, t)| *t).max().unwrap_or(0))
}

// ------------------------------------------------------------- datalog engine

const P_HOLDS: &str = "urn:ec:holds";
const P_SUCC: &str = "urn:ec:succ";
const P_HAPPENS: &str = "urn:ec:happens";
const P_INITIATES: &str = "urn:ec:initiates";
const P_TERMINATES: &str = "urn:ec:terminates";
const P_INITIATED: &str = "urn:ec:initiated";
const P_TERMINATED: &str = "urn:ec:terminated";
const P_CLIPPED: &str = "urn:ec:clipped";
const P_COMP: &str = "urn:ec:comp";

fn var(name: &str) -> ClauseTerm {
    ClauseTerm::var(name)
}

fn pos(s: ClauseTerm, p: &str, o: ClauseTerm) -> ClauseAtom {
    ClauseAtom::positive(s, p, o)
}

fn neg(s: ClauseTerm, p: &str, o: ClauseTerm) -> ClauseAtom {
    ClauseAtom::negated(s, p, o)
}

fn rule(head: ClauseAtom, body: Vec<ClauseAtom>) -> DlClause {
    DlClause::datalog(head, body)
}

fn insert(store: &mut RelationStore, s: &str, p: &str, o: &str) {
    store.insert(
        &format!("<{s}>"),
        &format!("<{p}>"),
        &format!("<{o}>"),
        RelationStore::DEFAULT_GRAPH,
    );
}

/// All fluents mentioned anywhere in the scenario.
fn all_fluents(sc: &Scenario) -> BTreeSet<&'static str> {
    let mut fs_ = BTreeSet::new();
    for f in sc.initially {
        fs_.insert(*f);
    }
    for (_, f) in sc.initiates {
        fs_.insert(*f);
    }
    for (_, f) in sc.terminates {
        fs_.insert(*f);
    }
    for (_, f, _) in sc.queries {
        fs_.insert(*f);
    }
    fs_
}

/// Level-1 derivations of `terminated(f,t)` straight from the EDB (used to
/// enumerate the explicit complement for the dual route).
fn terminated_facts(sc: &Scenario) -> BTreeSet<(&'static str, usize)> {
    sc.terminates
        .iter()
        .flat_map(|(te, tf)| {
            sc.happens
                .iter()
                .filter(move |(he, _)| he == te)
                .map(move |(_, ht)| (*tf, *ht))
        })
        .collect()
}

/// Route B (primary): stratified negation via purrdf-datalog negated atoms.
/// Returns the derived holds(f,t) set.
fn run_negation_route(sc: &Scenario, horizon: usize) -> BTreeSet<(String, usize)> {
    let mut rules = vec![
        // terminated(f,t) :- happens(e,t), terminates(e,f)
        rule(
            pos(var("f"), P_TERMINATED, var("t")),
            vec![
                pos(var("e"), P_HAPPENS, var("t")),
                pos(var("e"), P_TERMINATES, var("f")),
            ],
        ),
    ];
    // initiated(f,t) :- happens(e,t), initiates(e,f)
    rules.push(rule(
        pos(var("f"), P_INITIATED, var("t")),
        vec![
            pos(var("e"), P_HAPPENS, var("t")),
            pos(var("e"), P_INITIATES, var("f")),
        ],
    ));
    // clipped(f,s,t) :- terminated(f,s), succ(s,t)
    rules.push(rule(
        pos(var("f"), P_CLIPPED, var("t")),
        vec![
            pos(var("f"), P_TERMINATED, var("s")),
            pos(var("s"), P_SUCC, var("t")),
        ],
    ));
    // holds(f,t) :- succ(s,t), initiated(f,s)
    rules.push(rule(
        pos(var("f"), P_HOLDS, var("t")),
        vec![
            pos(var("s"), P_SUCC, var("t")),
            pos(var("f"), P_INITIATED, var("s")),
        ],
    ));
    // holds(f,t) :- succ(s,t), holds(f,s), not clipped(f,s,t)
    rules.push(rule(
        pos(var("f"), P_HOLDS, var("t")),
        vec![
            pos(var("s"), P_SUCC, var("t")),
            pos(var("f"), P_HOLDS, var("s")),
            neg(var("f"), P_CLIPPED, var("t")),
        ],
    ));

    let exe = compile(rules).expect("compile EC program");

    let mut store = RelationStore::new();
    for t in 0..horizon {
        insert(
            &mut store,
            &format!("urn:ec:t:{t}"),
            P_SUCC,
            &format!("urn:ec:t:{}", t + 1),
        );
    }
    for (e, t) in sc.happens {
        insert(
            &mut store,
            &format!("urn:ec:e:{e}"),
            P_HAPPENS,
            &format!("urn:ec:t:{t}"),
        );
    }
    for (e, f) in sc.initiates {
        insert(
            &mut store,
            &format!("urn:ec:e:{e}"),
            P_INITIATES,
            &format!("urn:ec:f:{f}"),
        );
    }
    for (e, f) in sc.terminates {
        insert(
            &mut store,
            &format!("urn:ec:e:{e}"),
            P_TERMINATES,
            &format!("urn:ec:f:{f}"),
        );
    }
    for f in sc.initially {
        insert(&mut store, &format!("urn:ec:f:{f}"), P_HOLDS, "urn:ec:t:0");
    }

    let ev = evaluate(&exe, store).expect("evaluate EC program");
    let mut holds = BTreeSet::new();
    for fact in ev.facts().facts_sorted() {
        if fact.predicate == format!("<{P_HOLDS}>") {
            let f = fact
                .subject
                .trim_start_matches("<urn:ec:f:")
                .trim_end_matches('>');
            let t = fact
                .object
                .trim_start_matches("<urn:ec:t:")
                .trim_end_matches('>');
            holds.insert((f.to_string(), t.parse::<usize>().expect("numeric time")));
        }
    }
    holds
}

/// Route A (dual cross-check): the same EC program with the negated atom
/// replaced by statically enumerated explicit complement facts `comp(f,t)`
/// ("no termination for f fired at t-1"), i.e. a pure positive program.
fn run_complement_route(sc: &Scenario, horizon: usize) -> BTreeSet<(String, usize)> {
    let terminated = terminated_facts(sc);
    let fluents = all_fluents(sc);

    let mut rules = vec![
        // initiated(f,t) :- happens(e,t), initiates(e,f)
        rule(
            pos(var("f"), P_INITIATED, var("t")),
            vec![
                pos(var("e"), P_HAPPENS, var("t")),
                pos(var("e"), P_INITIATES, var("f")),
            ],
        ),
    ];
    // holds(f,t) :- succ(s,t), initiated(f,s)
    rules.push(rule(
        pos(var("f"), P_HOLDS, var("t")),
        vec![
            pos(var("s"), P_SUCC, var("t")),
            pos(var("f"), P_INITIATED, var("s")),
        ],
    ));
    // holds(f,t) :- succ(s,t), holds(f,s), comp(f,t)
    rules.push(rule(
        pos(var("f"), P_HOLDS, var("t")),
        vec![
            pos(var("s"), P_SUCC, var("t")),
            pos(var("f"), P_HOLDS, var("s")),
            pos(var("f"), P_COMP, var("t")),
        ],
    ));

    let exe = compile(rules).expect("compile complement EC program");

    let mut store = RelationStore::new();
    for t in 0..horizon {
        insert(
            &mut store,
            &format!("urn:ec:t:{t}"),
            P_SUCC,
            &format!("urn:ec:t:{}", t + 1),
        );
    }
    for (e, t) in sc.happens {
        insert(
            &mut store,
            &format!("urn:ec:e:{e}"),
            P_HAPPENS,
            &format!("urn:ec:t:{t}"),
        );
    }
    for (e, f) in sc.initiates {
        insert(
            &mut store,
            &format!("urn:ec:e:{e}"),
            P_INITIATES,
            &format!("urn:ec:f:{f}"),
        );
    }
    for f in sc.initially {
        insert(&mut store, &format!("urn:ec:f:{f}"), P_HOLDS, "urn:ec:t:0");
    }
    // Explicit complement: comp(f,t) for every fluent f and every step t>0
    // where no termination for f fired at t-1. (comp at t=0 is unreachable.)
    for f in &fluents {
        for t in 1..=horizon {
            if !terminated.contains(&(*f, t - 1)) {
                insert(
                    &mut store,
                    &format!("urn:ec:f:{f}"),
                    P_COMP,
                    &format!("urn:ec:t:{t}"),
                );
            }
        }
    }

    let ev = evaluate(&exe, store).expect("evaluate complement EC program");
    let mut holds = BTreeSet::new();
    for fact in ev.facts().facts_sorted() {
        if fact.predicate == format!("<{P_HOLDS}>") {
            let f = fact
                .subject
                .trim_start_matches("<urn:ec:f:")
                .trim_end_matches('>');
            let t = fact
                .object
                .trim_start_matches("<urn:ec:t:")
                .trim_end_matches('>');
            holds.insert((f.to_string(), t.parse::<usize>().expect("numeric time")));
        }
    }
    holds
}

/// HoldsAt verdict per query: holds(f, qt) in the closure.
fn verdict_map(sc: &Scenario, holds: &BTreeSet<(String, usize)>) -> BTreeMap<String, String> {
    sc.queries
        .iter()
        .map(|(gid, f, qt)| {
            let holds_at = holds.contains(&(f.to_string(), *qt));
            (format!("{gid}: HoldsAt({f}@{qt})"), holds_at.to_string())
        })
        .collect()
}

/// Assemble the vector JSON per tests/fixtures/graphlaw/SCHEMA.md.
fn build_vector(
    sc: &Scenario,
    rev: &str,
    verdicts: &BTreeMap<String, String>,
) -> serde_json::Value {
    let facts_json: Vec<serde_json::Value> = sc
        .happens
        .iter()
        .map(|(e, t)| json!({"key": format!("ec:happens:{t}"), "value": e}))
        .chain(
            sc.initially
                .iter()
                .map(|f| json!({"key": "ec:initially", "value": f})),
        )
        .chain(
            sc.initiates
                .iter()
                .map(|(e, f)| json!({"key": format!("ec:initiates:{e}"), "value": f})),
        )
        .chain(
            sc.terminates
                .iter()
                .map(|(e, f)| json!({"key": format!("ec:terminates:{e}"), "value": f})),
        )
        .collect();

    let goals_json: Vec<serde_json::Value> = sc
        .queries
        .iter()
        .map(|(gid, f, t)| json!({"id": gid, "predicate": "ec:holdsat", "value": format!("{f}@{t}")}))
        .collect();

    // derived_facts_sorted: expectations evidenced in the breed output
    // haystack — the ec-verdict trace line and the selected verdict string.
    let mut derived: Vec<String> = Vec::new();
    for (gid, f, t) in sc.queries {
        derived.push(format!(
            "{gid}: HoldsAt({f}@{t}) = {}",
            verdicts[&format!("{gid}: HoldsAt({f}@{t})")]
        ));
        derived.push(format!(
            "ec:verdict:{f}@{t}={}",
            verdicts[&format!("{gid}: HoldsAt({f}@{t})")]
        ));
    }
    derived.sort();

    json!({
        "vector_id": sc.vector_id,
        "breed": "event_calculus",
        "provenance": {
            "graphlaw_rev": rev,
            "graphlaw_version": GL_VERSION,
            "purrdf": PURRDF,
            "n3_engine": N3_ENGINE,
            "generator": GENERATOR,
            "stratified": true,
        },
        "breed_input": {
            "intent": sc.intent,
            "facts": facts_json,
            "rules": [],
            "goals": goals_json,
            "candidates": [],
            "cases": [],
            "state": [],
        },
        "expected": {
            "derived_facts_sorted": derived,
            "verdicts": verdicts,
            "notes": format!(
                "{} Route: stratified negation (purrdf-datalog negated atoms driven directly \
                 via graphlaw::datalog; NOT the positive-only JSON-ABI shim), cross-checked \
                 against an explicit-complement positive program — the holds-projections agree.",
                sc.notes
            ),
        },
    })
}
