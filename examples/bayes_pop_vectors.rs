//! bayes_pop_vectors — GraphLaw stratified-Datalog vector generator for two
//! wasm4pm cognition breeds.
//!
//!   * bayesian_network dsep — Bayes-ball directional reachability as
//!     stratified Datalog over node/edge facts:
//!     stratum 1 (EDB): node/par/obs/start/end facts + extensional
//!     disequality ddiff;
//!     stratum 2: ancestor-of-observation seeding (anc, recursive),
//!     unobserved gating unobs(N) :- node(N), !obs(N) (negation as
//!     failure), and moralization marry(P,Q) :- par(C,P), par(C,Q),
//!     ddiff(P,Q) (marry co-parents);
//!     stratum 3: the Bayes-ball direction state machine, up/down
//!     two-phase reachability as recursive positive rules:
//!     up(S)  :- start(S).
//!     down(C) :- up(N), par(C,N), unobs(N).   // up at unobs n -> children down
//!     up(P)  :- up(N), par(N,P), unobs(N).    // up at unobs n -> parents up
//!     down(C) :- down(N), par(C,N), unobs(N). // down at unobs n -> children down
//!     up(P)  :- down(N), par(N,P), anc(N).    // down at n in anc -> parents up
//!     reach(E) :- up(E). reach(E) :- down(E).
//!     dsep_ok(E) :- end(E), !reach(E).
//!   * partial_order_plan VALIDATION — verifier, not deriver. Given the
//!     breed's emitted plan (causal links + orderings), compute the order
//!     relation's transitive closure (ordtc, the Chicago court's canonical
//!     case), acyclicity cyclic(S) :- ordtc(S,S), precondition support
//!     supp/unsupp via causal links whose source adds the condition, and
//!     threats threat(L,T): a deleter of the link condition not forced
//!     before the link source nor after the link destination.
//!
//! Negation in a rule premise is spelled with a leading '!' (negation as
//! failure; the negated atom's variables are bound by positive atoms in the
//! same rule so the engine's stratification check passes). Disequality is
//! extensional: ddiff/pdiff fact pairs enumerated by the generator.
//!
//! Vectors conform to fixtures/graphlaw/SCHEMA.md v1: breed-native
//! breed_input (all seven BreedInput fields), expected.verdicts as an object
//! whose values are CONFIRM-evidenced in the breed output haystack
//! (dsep explanation strings; the POP plan string; `refused` for the lawful
//! deny path). The Datalog court program + fixpoint travel in
//! provenance.datalog_program (descriptive, replayable). The notes field
//! marks each family's class: "verifier, not deriver" — the POP vectors
//! validate a plan/candidate, they do not derive one; the dsep vectors
//! verify reachability semantics against a hand-checked verdict.
//!
//! Run from /Users/sac/graphlaw:
//!   cargo run --example bayes_pop_vectors [-- --verify]
//!   (GL_VECDIR overrides the output dir)

use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::PathBuf;

use graphlaw::datalog::clause::{ClauseAtom, ClauseTerm, DlClause};
use graphlaw::datalog::seminaive::{compile, evaluate};
use graphlaw::datalog::store::RelationStore;
use serde_json::{Value, json};

const PURRDF: &str = "2.0.2";
const EYERON: &str = "0.7.8";

// ------------------------------------------------------------- engine plumbing

fn iri(s: &str) -> ClauseTerm {
    ClauseTerm::iri(s)
}

fn vr(s: &str) -> ClauseTerm {
    ClauseTerm::var(s)
}

fn posv(s: &str, p: &str, o: &str) -> ClauseAtom {
    ClauseAtom::positive(vr(s), p.to_string(), vr(o))
}

fn negv(s: &str, p: &str, o: &str) -> ClauseAtom {
    ClauseAtom::negated(vr(s), p.to_string(), vr(o))
}

/// Translate a breed-style key ("pred:a,b" / "pred:a" / "pred") to [s, p, o],
/// matching breed_oracle_gen's convention.
fn atom_to_triple(key: &str) -> [String; 3] {
    let (pred, args) = match key.split_once(':') {
        Some((p, rest)) => (p, rest.split(',').collect::<Vec<_>>()),
        None => (key, vec![]),
    };
    assert!(args.len() <= 2, "arity > 2 not supported: {key}");
    match args.len() {
        0 => [pred.to_string(), "-".into(), "-".into()],
        1 => [args[0].to_string(), pred.to_string(), "()".into()],
        _ => [args[0].to_string(), pred.to_string(), args[1..].join(",")],
    }
}

/// Build one rule from breed-style conclusion + premise strings. A premise
/// prefixed '!' becomes a negated (negation-as-failure) body atom.
fn term(t: &str) -> ClauseTerm {
    if t.starts_with('?') {
        vr(t)
    } else {
        iri(t)
    }
}

fn rule(conclusion: &str, premises: &[&str]) -> DlClause {
    let head_terms = atom_to_triple(conclusion);
    let head = ClauseAtom::positive(
        term(&head_terms[0]),
        head_terms[1].clone(),
        term(&head_terms[2]),
    );
    let body: Vec<ClauseAtom> = premises
        .iter()
        .map(|p| {
            let [s, pred, o] = atom_to_triple(p);
            if let Some(rest) = pred.strip_prefix('!') {
                negv(&s, rest, &o)
            } else {
                posv(&s, &pred, &o)
            }
        })
        .collect();
    DlClause::datalog(head, body)
}

/// Compile + evaluate; returns the full fixpoint as "pred:s,o" surface keys.
fn eval_fixpoint(rules: Vec<DlClause>, edb_keys: &[&str]) -> BTreeSet<String> {
    let exe = compile(rules).expect("datalog compile");
    let mut store = RelationStore::new();
    for k in edb_keys {
        let [s, p, o] = atom_to_triple(k);
        store.insert(
            &format!("<{s}>"),
            &format!("<{p}>"),
            &format!("<{o}>"),
            RelationStore::DEFAULT_GRAPH,
        );
    }
    let ev = evaluate(&exe, store).expect("datalog evaluate");
    let mut out = BTreeSet::new();
    for f in ev.facts().facts_sorted() {
        let un = |x: &str| {
            x.trim_start_matches('<')
                .trim_end_matches('>')
                .to_string()
        };
        let (s, p, o) = (un(&f.subject), un(&f.predicate), un(&f.object));
        let key = match o.as_str() {
            "-" => p.clone(),
            "()" => format!("{p}:{s}"),
            _ => format!("{p}:{s},{o}"),
        };
        out.insert(key);
    }
    out
}

/// Extensional disequality pairs over a set of constants (predicate pfx).
fn diff_keys(pfx: &str, items: &[&str]) -> Vec<String> {
    let mut v = Vec::new();
    for a in items {
        for b in items {
            if a != b {
                v.push(format!("{pfx}:{a},{b}"));
            }
        }
    }
    v
}

// ------------------------------------------------------------ dsep program

/// The shared dsep rule set (premise '!' = negation as failure).
fn dsep_rules() -> Vec<DlClause> {
    vec![
        // stratum 2: ancestor-of-observation seeding + transitive closure.
        rule("anc:?0", &["obs:?0"]),
        rule("anc:?0,?1", &["obs:?0", "par:?0,?1"]),
        rule("anc:?0,?2", &["anc:?0,?1", "par:?1,?2"]),
        // stratum 2: unobserved gating.
        rule("unobs:?0", &["node:?0", "!obs:?0"]),
        // stratum 2: moralization — marry co-parents of a common child.
        rule("marry:?0,?1", &["par:?2,?0", "par:?2,?1", "ddiff:?0,?1"]),
        // stratum 3: Bayes-ball direction state machine.
        rule("up:?0", &["start:?0"]),
        rule("down:?1", &["up:?0", "par:?1,?0", "unobs:?0"]),
        rule("up:?1", &["up:?0", "par:?0,?1", "unobs:?0"]),
        rule("down:?1", &["down:?0", "par:?1,?0", "unobs:?0"]),
        rule("up:?1", &["down:?0", "par:?0,?1", "anc:?0"]),
        // verdict layer.
        rule("reach:?0", &["up:?0"]),
        rule("reach:?0", &["down:?0"]),
        rule("dsep_ok:?0", &["end:?0", "!reach:?0"]),
    ]
}

/// Native Bayes-ball port of the breed's state machine
/// (crates/wasm4pm-cognition/src/breeds/bayesian_network.rs:470-557) for the
/// in-generator differential check.
fn bayes_ball_native(
    edges: &[(usize, usize)], // (parent, child)
    n: usize,
    start: usize,
    end: usize,
    obs: &BTreeSet<usize>,
) -> bool {
    // dsep=true means separated (end NOT reachable).
    let mut parents: Vec<Vec<usize>> = vec![vec![]; n];
    let mut children: Vec<Vec<usize>> = vec![vec![]; n];
    for &(p, c) in edges {
        parents[c].push(p);
        children[p].push(c);
    }
    let mut ancestors_of_obs = BTreeSet::new();
    let mut q: VecDeque<usize> = obs.iter().copied().collect();
    ancestors_of_obs.extend(obs.iter().copied());
    while let Some(x) = q.pop_front() {
        for &p in &parents[x] {
            if ancestors_of_obs.insert(p) {
                q.push_back(p);
            }
        }
    }
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum Dir {
        Up,
        Down,
    }
    let mut visited = BTreeSet::new();
    let mut rq: VecDeque<(usize, Dir)> = VecDeque::new();
    visited.insert((start, Dir::Up));
    rq.push_back((start, Dir::Up));
    while let Some((node, dir)) = rq.pop_front() {
        if node == end {
            return false;
        }
        let is_obs = obs.contains(&node);
        match dir {
            Dir::Up => {
                if !is_obs {
                    for &p in &parents[node] {
                        if visited.insert((p, Dir::Up)) {
                            rq.push_back((p, Dir::Up));
                        }
                    }
                    for &c in &children[node] {
                        if visited.insert((c, Dir::Down)) {
                            rq.push_back((c, Dir::Down));
                        }
                    }
                }
            }
            Dir::Down => {
                if !is_obs {
                    for &c in &children[node] {
                        if visited.insert((c, Dir::Down)) {
                            rq.push_back((c, Dir::Down));
                        }
                    }
                    if ancestors_of_obs.contains(&node) {
                        for &p in &parents[node] {
                            if visited.insert((p, Dir::Up)) {
                                rq.push_back((p, Dir::Up));
                            }
                        }
                    }
                } else {
                    for &p in &parents[node] {
                        if visited.insert((p, Dir::Up)) {
                            rq.push_back((p, Dir::Up));
                        }
                    }
                }
            }
        }
    }
    true
}

// ------------------------------------------------------------ POP program

/// The shared POP plan-validation rule set.
fn pop_rules() -> Vec<DlClause> {
    vec![
        // transitive closure of the ordering relation (Chicago canonical case).
        rule("ordtc:?0,?1", &["ord:?0,?1"]),
        rule("ordtc:?0,?2", &["ord:?0,?1", "ordtc:?1,?2"]),
        // acyclicity.
        rule("cyclic:?0", &["ordtc:?0,?0"]),
        // precondition support: a causal link to S with condition C whose
        // source F adds C.
        rule(
            "supp:?0,?1",
            &[
                "pre:?0,?1",
                "link:?2",
                "linkfrom:?2,?3",
                "linkto:?2,?0",
                "linkcond:?2,?1",
                "add:?3,?1",
            ],
        ),
        rule("unsupp:?0,?1", &["pre:?0,?1", "!supp:?0,?1"]),
        // threats: step T deletes link L's condition, T is neither the link's
        // source nor destination (extensional diff), and T is not forced
        // before the source nor after the destination.
        rule(
            "threat:?0,?1",
            &[
                "link:?0",
                "linkfrom:?0,?2",
                "linkto:?0,?3",
                "linkcond:?0,?4",
                "del:?1,?4",
                "pdiff:?1,?2",
                "pdiff:?1,?3",
                "!ordtc:?1,?2",
                "!ordtc:?3,?1",
            ],
        ),
    ]
}

/// Verdict helpers over a fixpoint.
fn has_pred(fix: &BTreeSet<String>, pred: &str) -> bool {
    fix.iter().any(|k| {
        k.split_once(':')
            .map(|(p, _)| p == pred)
            .unwrap_or(false)
    })
}

fn dsep_verdict(fix: &BTreeSet<String>, end: &str) -> &'static str {
    if fix.contains(&format!("dsep_ok:{end}")) {
        "dsep"
    } else {
        "not_dsep"
    }
}

fn plan_verdict(fix: &BTreeSet<String>) -> &'static str {
    if has_pred(fix, "cyclic") || has_pred(fix, "unsupp") || has_pred(fix, "threat") {
        "invalid"
    } else {
        "valid"
    }
}

// ------------------------------------------------------------------- cases

struct DsepCase {
    vector_id: &'static str,
    nodes: &'static [&'static str],
    /// (child, parent) directed edges.
    edges: &'static [(&'static str, &'static str)],
    /// cpt: facts for the breed-native input (key, value).
    cpts: &'static [(&'static str, &'static str)],
    obs: &'static [&'static str],
    start: &'static str,
    end: &'static str,
    expected: &'static str, // hand-checked
    notes: &'static str,
}

struct PopCase {
    vector_id: &'static str,
    steps: &'static [&'static str],
    ord: &'static [(&'static str, &'static str)],
    pre: &'static [(&'static str, &'static str)],
    add: &'static [(&'static str, &'static str)],
    del: &'static [(&'static str, &'static str)],
    /// (link id, from, to, condition)
    links: &'static [(&'static str, &'static str, &'static str, &'static str)],
    /// breed-native surface: (op, pre|add|del) fact pairs, rules, state,
    /// goals, intent, and the verdict label/value.
    native_ops: &'static [(&'static str, &'static str)],
    native_rules: &'static [(&'static str, &'static str, &'static str)],
    native_state: &'static [(&'static str, &'static str)],
    native_goals: &'static [(&'static str, &'static str)],
    native_intent: &'static str,
    expected: &'static str,
    notes: &'static str,
}

fn dsep_cases() -> Vec<DsepCase> {
    vec![
        // (a) paper fixture — Pearl (1988) burglary/earthquake/alarm network,
        // cross-referenced against tests/fixtures/papers/bayesian_network.json
        // (same network the posterior fixture uses).
        DsepCase {
            vector_id: "bayesian_network-dsep-paper-b-m-a",
            nodes: &["A", "B", "E", "J", "M"],
            edges: &[("A", "B"), ("A", "E"), ("J", "A"), ("M", "A")],
            cpts: &[
                ("cpt:A|B,E", "0.001,0.29,0.94,0.95"),
                ("cpt:B", "0.001"),
                ("cpt:E", "0.002"),
                ("cpt:J|A", "0.05,0.90"),
                ("cpt:M|A", "0.01,0.70"),
            ],
            obs: &["A"],
            start: "B",
            end: "M",
            expected: "dsep",
            notes: "Pearl 1988 alarm network: B and M are d-separated given the chain node A \
                    (A observed blocks the chain B->A->M). Cross-checks the network of \
                    papers/bayesian_network.json (dsep lens; that fixture holds the \
                    posterior query P(B|J,M)). Datalog verdict cross-checked against a \
                    native port of the breed's Bayes-ball state machine. Verifier, not \
                    deriver: the verdict is hand-checked, the engine verifies.",
        },
        DsepCase {
            vector_id: "bayesian_network-dsep-paper-b-e-a",
            nodes: &["A", "B", "E", "J", "M"],
            edges: &[("A", "B"), ("A", "E"), ("J", "A"), ("M", "A")],
            cpts: &[
                ("cpt:A|B,E", "0.001,0.29,0.94,0.95"),
                ("cpt:B", "0.001"),
                ("cpt:E", "0.002"),
                ("cpt:J|A", "0.05,0.90"),
                ("cpt:M|A", "0.01,0.70"),
            ],
            obs: &["A"],
            start: "B",
            end: "E",
            expected: "not_dsep",
            notes: "Pearl 1988 alarm network: B and E are d-connected given A — A is an \
                    OBSERVED COLLIDER on B->A<-E, and observing a collider activates the \
                    path (explaining-away). The classical discriminating case for the \
                    ancestor-of-obs seeding. Differential-checked vs the breed's \
                    Bayes-ball. Verifier, not deriver.",
        },
        // (b) collider case: classically independent a,c given empty;
        // dependent given b.
        DsepCase {
            vector_id: "bayesian_network-dsep-collider-empty",
            nodes: &["a", "b", "c"],
            edges: &[("b", "a"), ("b", "c")],
            cpts: &[("cpt:B|A,C", "0.5,0.5,0.5,0.5")],
            obs: &[],
            start: "a",
            end: "c",
            expected: "dsep",
            notes: "Collider a->b<-c with empty observation set: a and c are classically \
                    independent (marginal), so d-separation HOLDS. The path through b is \
                    blocked because b is unobserved. Verifier, not deriver.",
        },
        DsepCase {
            vector_id: "bayesian_network-dsep-collider-b",
            nodes: &["a", "b", "c"],
            edges: &[("b", "a"), ("b", "c")],
            cpts: &[("cpt:B|A,C", "0.5,0.5,0.5,0.5")],
            obs: &["b"],
            start: "a",
            end: "c",
            expected: "not_dsep",
            notes: "Collider a->b<-c conditioned on the collider b: observing b renders a \
                    and c DEPENDENT (selection bias / explaining-away). D-separation must \
                    FAIL here; a flat moralization-without-ancestor-seeding encoding gets \
                    exactly this case wrong. Verifier, not deriver.",
        },
        // (c) chain case.
        DsepCase {
            vector_id: "bayesian_network-dsep-chain-b",
            nodes: &["a", "b", "c"],
            edges: &[("b", "a"), ("c", "b")],
            cpts: &[("cpt:B|A", "0.5,0.9"), ("cpt:C|B", "0.5,0.9")],
            obs: &["b"],
            start: "a",
            end: "c",
            expected: "dsep",
            notes: "Chain a->b->c with the middle node b observed: the chain is blocked, \
                    a and c are d-separated. Verifier, not deriver.",
        },
        DsepCase {
            vector_id: "bayesian_network-dsep-chain-empty",
            nodes: &["a", "b", "c"],
            edges: &[("b", "a"), ("c", "b")],
            cpts: &[("cpt:B|A", "0.5,0.9"), ("cpt:C|B", "0.5,0.9")],
            obs: &[],
            start: "a",
            end: "c",
            expected: "not_dsep",
            notes: "Chain a->b->c with no observations: influence flows along the chain, \
                    a and c are d-connected. Verifier, not deriver.",
        },
    ]
}

fn pop_cases() -> Vec<PopCase> {
    vec![
        // (a) paper fixture plan — Sussman anomaly (McAllester & Rosenblitt 1991),
        // cross-referenced against tests/fixtures/papers/partial_order_plan.json.
        PopCase {
            vector_id: "partial_order_plan-sussman-paper",
            steps: &["start", "end", "put_c", "put_b", "put_a"],
            ord: &[
                ("start", "put_c"),
                ("start", "put_b"),
                ("start", "put_a"),
                ("put_c", "put_a"),
                ("put_c", "put_b"),
                ("put_b", "put_a"),
                ("put_a", "end"),
                ("put_b", "end"),
            ],
            pre: &[
                ("put_c", "clear_c"),
                ("put_c", "on_c_a"),
                ("put_a", "clear_a"),
                ("put_a", "clear_b"),
                ("put_a", "ontable_a"),
                ("put_b", "clear_b"),
                ("put_b", "clear_c"),
                ("put_b", "ontable_b"),
                ("end", "on_a_b"),
                ("end", "on_b_c"),
            ],
            add: &[
                ("start", "clear_b"),
                ("start", "clear_c"),
                ("start", "ontable_a"),
                ("start", "ontable_b"),
                ("start", "on_c_a"),
                ("put_c", "clear_a"),
                ("put_c", "ontable_c"),
                ("put_b", "on_b_c"),
                ("put_a", "on_a_b"),
            ],
            del: &[
                ("put_c", "on_c_a"),
                ("put_a", "clear_b"),
                ("put_a", "ontable_a"),
                ("put_b", "clear_c"),
                ("put_b", "ontable_b"),
            ],
            links: &[
                ("L1", "start", "put_c", "clear_c"),
                ("L2", "start", "put_c", "on_c_a"),
                ("L3", "start", "put_b", "clear_b"),
                ("L4", "start", "put_b", "ontable_b"),
                ("L5", "start", "put_a", "clear_b"),
                ("L6", "start", "put_a", "ontable_a"),
                ("L7", "put_c", "put_a", "clear_a"),
                ("L8", "start", "put_b", "clear_c"),
                ("L9", "put_a", "end", "on_a_b"),
                ("L10", "put_b", "end", "on_b_c"),
            ],
            native_ops: &[
                ("put_c_from_a_on_table", "pre"),
                ("put_c_from_a_on_table", "add"),
                ("put_c_from_a_on_table", "del"),
                ("put_a_on_b", "pre"),
                ("put_a_on_b", "add"),
                ("put_a_on_b", "del"),
                ("put_b_on_c", "pre"),
                ("put_b_on_c", "add"),
                ("put_b_on_c", "del"),
            ],
            native_rules: &[
                ("put_c_from_a_on_table",
                 "clear_c=true,on_c_a=true",
                 "clear_a=true; ontable_c=true; !on_c_a=true"),
                ("put_a_on_b",
                 "clear_a=true,clear_b=true,ontable_a=true",
                 "on_a_b=true; !clear_b=true; !ontable_a=true"),
                ("put_b_on_c",
                 "clear_b=true,clear_c=true,ontable_b=true",
                 "on_b_c=true; !clear_c=true; !ontable_b=true"),
            ],
            native_state: &[
                ("on_c_a", "true"),
                ("clear_c", "true"),
                ("clear_b", "true"),
                ("ontable_a", "true"),
                ("ontable_b", "true"),
            ],
            native_goals: &[("on_a_b", "true"), ("on_b_c", "true")],
            native_intent: "solve the Sussman anomaly with SNLP causal-link planning",
            expected: "put_c_from_a_on_table;put_b_on_c;put_a_on_b",
            notes: "Sussman anomaly plan (papers/partial_order_plan.json's emitted order \
                    put_c;put_b;put_a) validated as the Chicago court's canonical case: \
                    ordtc closure, acyclic; every precondition carried by a causal link \
                    whose source adds the condition; every deleter threat resolved by \
                    promotion (put_a after put_b protects clear_b for put_b) or demotion \
                    ordering. Verifier, not deriver: the plan is INPUT, the vector \
                    checks it.",
        },
        // (b) valid multi-step plan with a genuinely interleaving deleter.
        PopCase {
            vector_id: "partial_order_plan-valid-multistep",
            steps: &["start", "end", "opA", "opB", "opC"],
            ord: &[
                ("start", "opA"),
                ("opA", "opB"),
                ("opB", "opC"),
                ("opC", "end"),
            ],
            pre: &[
                ("opA", "p"),
                ("opB", "r"),
                ("opC", "s"),
                ("end", "t"),
            ],
            add: &[
                ("start", "p"),
                ("start", "q"),
                ("opA", "r"),
                ("opB", "s"),
                ("opC", "t"),
            ],
            del: &[
                ("opA", "q"),
                ("opC", "r"),
            ],
            links: &[
                ("K1", "start", "opA", "p"),
                ("K2", "opA", "opB", "r"),
                ("K3", "opB", "opC", "s"),
                ("K4", "opC", "end", "t"),
            ],
            native_ops: &[
                ("opA", "pre"), ("opA", "add"), ("opA", "del"),
                ("opB", "pre"), ("opB", "add"),
                ("opC", "pre"), ("opC", "add"), ("opC", "del"),
            ],
            native_rules: &[
                ("opA", "p=true", "r=true; !q=true"),
                ("opB", "r=true", "s=true"),
                ("opC", "s=true", "t=true; !r=true"),
            ],
            native_state: &[("p", "true"), ("q", "true")],
            native_goals: &[("t", "true")],
            native_intent: "validate a valid 3-action chain plan",
            expected: "opA;opB;opC",
            notes: "Valid 3-action chain: opC deletes r (opA's contribution to opB) but is \
                    ordered after opB (promotion), so no unresolved threat; opA deletes q, \
                    which no link needs. All preconditions supported. Verifier, not \
                    deriver.",
        },
        // (c) cyclic-ordering rejection: the Datalog court (provenance
        //     datalog_fixpoint) rejects the cyclic ordering set via
        //     cyclic(S) :- ordtc(S,S); the breed-native surface refuses.
        PopCase {
            vector_id: "partial_order_plan-cyclic-refused",
            steps: &["start", "end", "opA", "opB", "opC"],
            ord: &[
                ("start", "opA"),
                ("opA", "opB"),
                ("opB", "opC"),
                ("opC", "opA"),
                ("opC", "end"),
            ],
            pre: &[
                ("opA", "p"),
                ("opB", "r"),
                ("opC", "s"),
                ("end", "t"),
            ],
            add: &[
                ("start", "p"),
                ("opA", "r"),
                ("opB", "s"),
                ("opC", "t"),
            ],
            del: &[],
            links: &[
                ("K1", "start", "opA", "p"),
                ("K2", "opA", "opB", "r"),
                ("K3", "opB", "opC", "s"),
                ("K4", "opC", "end", "t"),
            ],
            native_ops: &[
                ("opA", "pre"), ("opA", "add"),
                ("opB", "pre"), ("opB", "add"),
                ("opC", "pre"), ("opC", "add"), ("opC", "del"),
            ],
            native_rules: &[
                ("opA", "p=true", "r=true"),
                ("opB", "r=true", "s=true"),
                ("opC", "s=true", "t=true; !r=true"),
            ],
            native_state: &[("p", "true")],
            native_goals: &[("on_x_y", "true")],
            native_intent: "candidate plan with a cyclic ordering (opC->opA back edge) must be rejected",
            expected: "refused",
            notes: "Two-layer vector. Datalog court (provenance datalog_fixpoint): the \
                    ordering set carries the back edge opC->opA, ordtc(S,S) derives for \
                    opA/opB/opC, cyclic fires, the candidate plan is REJECTED. Breed \
                    surface: the candidate goal on_x_y is added by no action, so the \
                    planner refuses with a lawful typed refusal (verdict refused). \
                    Verifier, not deriver.",
        },
    ]
}

// -------------------------------------------------------------------- main

fn vecdir() -> PathBuf {
    std::env::var_os("GL_VECDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw")
        })
}

fn git_rev() -> String {
    let out = std::process::Command::new("git")
        .args(["-C", env!("CARGO_MANIFEST_DIR"), "rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse");
    String::from_utf8(out.stdout).expect("utf8").trim().to_string()
}

/// Build EDB fact keys (Datalog court) + breed-native breed_input for a dsep
/// case. The breed surface is the bayesian_network cpt: encoding; the Datalog
/// program travels in provenance.
fn dsep_edb(c: &DsepCase) -> (Vec<String>, Value) {
    // Datalog court EDB (unchanged).
    let mut facts: Vec<String> = c.nodes.iter().map(|n| format!("node:{n}")).collect();
    facts.extend(diff_keys("ddiff", c.nodes));
    for (child, parent) in c.edges {
        facts.push(format!("par:{child},{parent}"));
    }
    for o in c.obs {
        facts.push(format!("obs:{o}"));
    }
    facts.push(format!("start:{}", c.start));
    facts.push(format!("end:{}", c.end));

    // Breed-native input: the Pearl network as cpt: facts (values are the
    // canonical alarm-network CPTs where applicable; dsep reads only the
    // graph, but values keep the input honest).
    let goal_value = format!(
        "dsep:{},{}|{}",
        c.start,
        c.end,
        c.obs.join(",")
    );
    let cpt_facts: Vec<Value> = c
        .cpts
        .iter()
        .map(|(k, v)| json!({"key": k, "value": v}))
        .collect();
    let breed_input = json!({
        "intent": format!("d-separation {},{} given {{{}}}", c.start, c.end, c.obs.join(",")),
        "candidates": [],
        "facts": cpt_facts,
        "cases": [],
        "rules": [],
        "goals": [{
            "id": "g1",
            "predicate": "query",
            "value": goal_value,
        }],
        "state": [],
    });
    (facts, breed_input)
}

/// The dsep rule set in breed-string form (mirrors dsep_rules()).
fn dsep_rule_specs() -> Vec<Value> {
    let mk = |conclusion: &str, premise: &[&str]| {
        json!({
            "id": format!("r_{}", conclusion.split(':').next().unwrap_or("r")),
            "premise": premise,
            "conclusion": conclusion,
            "certainty": 1.0,
        })
    };
    vec![
        mk("anc:?0", &["obs:?0"]),
        mk("anc:?0,?1", &["obs:?0", "par:?0,?1"]),
        mk("anc:?0,?2", &["anc:?0,?1", "par:?1,?2"]),
        mk("unobs:?0", &["node:?0", "!obs:?0"]),
        mk("marry:?0,?1", &["par:?2,?0", "par:?2,?1", "ddiff:?0,?1"]),
        mk("up:?0", &["start:?0"]),
        mk("down:?1", &["up:?0", "par:?1,?0", "unobs:?0"]),
        mk("up:?1", &["up:?0", "par:?0,?1", "unobs:?0"]),
        mk("down:?1", &["down:?0", "par:?1,?0", "unobs:?0"]),
        mk("up:?1", &["down:?0", "par:?0,?1", "anc:?0"]),
        mk("reach:?0", &["up:?0"]),
        mk("reach:?0", &["down:?0"]),
        mk("dsep_ok:?0", &["end:?0", "!reach:?0"]),
    ]
}

/// Build EDB fact keys (Datalog court) + breed-native breed_input for a POP
/// case. The breed surface is the pop: STRIPS encoding (rules + state +
/// goals), exactly the papers/partial_order_plan.json shape; the Datalog
/// validation court travels in provenance.
fn pop_edb(c: &PopCase) -> (Vec<String>, Value) {
    let mut facts: Vec<String> = c.steps.iter().map(|s| format!("step:{s}")).collect();
    facts.extend(diff_keys("pdiff", c.steps));
    for (a, b) in c.ord {
        facts.push(format!("ord:{a},{b}"));
    }
    for (s, cond) in c.pre {
        facts.push(format!("pre:{s},{cond}"));
    }
    for (s, cond) in c.add {
        facts.push(format!("add:{s},{cond}"));
    }
    for (s, cond) in c.del {
        facts.push(format!("del:{s},{cond}"));
    }
    for (id, from, to, cond) in c.links {
        facts.push(format!("link:{id}"));
        facts.push(format!("linkfrom:{id},{from}"));
        facts.push(format!("linkto:{id},{to}"));
        facts.push(format!("linkcond:{id},{cond}"));
    }
    let _ = &facts;
    // Breed-native input: pop: op facts + rules + state + goals.
    let mut op_facts: Vec<Value> = Vec::new();
    for (op, key) in c.native_ops {
        op_facts.push(json!({"key": format!("pop:op:{op}:{key}"), "value": ""}));
    }
    let rules: Vec<Value> = c
        .native_rules
        .iter()
        .map(|(id, premise, conclusion)| {
            json!({
                "id": id,
                "premise": premise.split(',').filter(|s| !s.is_empty()).collect::<Vec<_>>(),
                "conclusion": conclusion,
                "certainty": 1.0,
            })
        })
        .collect();
    let state: Vec<Value> = c
        .native_state
        .iter()
        .map(|(p, v)| json!({"predicate": p, "value": v}))
        .collect();
    let goals: Vec<Value> = c
        .native_goals
        .iter()
        .enumerate()
        .map(|(i, (p, v))| json!({"id": format!("g{i}"), "predicate": p, "value": v}))
        .collect();
    let breed_input = json!({
        "intent": c.native_intent,
        "candidates": [],
        "facts": op_facts,
        "cases": [],
        "rules": rules,
        "goals": goals,
        "state": state,
    });
    (facts, breed_input)
}

/// The POP rule set in breed-string form (mirrors pop_rules()).
fn pop_rule_specs() -> Vec<Value> {
    let mk = |conclusion: &str, premise: &[&str]| {
        json!({
            "id": format!("r_{}", conclusion.split(':').next().unwrap_or("r")),
            "premise": premise,
            "conclusion": conclusion,
            "certainty": 1.0,
        })
    };
    vec![
        mk("ordtc:?0,?1", &["ord:?0,?1"]),
        mk("ordtc:?0,?2", &["ord:?0,?1", "ordtc:?1,?2"]),
        mk("cyclic:?0", &["ordtc:?0,?0"]),
        mk("supp:?0,?1", &[
            "pre:?0,?1",
            "link:?2",
            "linkfrom:?2,?3",
            "linkto:?2,?0",
            "linkcond:?2,?1",
            "add:?3,?1",
        ]),
        mk("unsupp:?0,?1", &["pre:?0,?1", "!supp:?0,?1"]),
        mk("threat:?0,?1", &[
            "link:?0",
            "linkfrom:?0,?2",
            "linkto:?0,?3",
            "linkcond:?0,?4",
            "del:?1,?4",
            "pdiff:?1,?2",
            "pdiff:?1,?3",
            "!ordtc:?1,?2",
            "!ordtc:?3,?1",
        ]),
    ]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verify = args.iter().any(|a| a == "--verify");
    let rev = git_rev();
    let dir = vecdir();

    let mut failures = 0usize;
    let mut written = 0usize;

    let dsep = dsep_cases();
    for c in &dsep {
        let (facts, breed_input) = dsep_edb(c);
        let fix = eval_fixpoint(dsep_rules(), &facts.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        if std::env::var_os("GL_DEBUG").is_some() {
            eprintln!("--- {} facts:", c.vector_id);
            for f in &fix {
                eprintln!("  {f}");
            }
        }
        let got = dsep_verdict(&fix, c.end);

        // Differential court: native Bayes-ball port must agree.
        let idx = |n: &str| c.nodes.iter().position(|x| *x == n).expect("node");
        let edges_usize: Vec<(usize, usize)> =
            c.edges.iter().map(|(ch, pa)| (idx(pa), idx(ch))).collect();
        let obs_set: BTreeSet<usize> = c.obs.iter().map(|o| idx(o)).collect();
        let native = bayes_ball_native(&edges_usize, c.nodes.len(), idx(c.start), idx(c.end), &obs_set);
        let native_verdict = if native { "dsep" } else { "not_dsep" };
        if native_verdict != c.expected {
            eprintln!(
                "FAIL {}: native Bayes-ball {} != hand-checked {}",
                c.vector_id, native_verdict, c.expected
            );
            failures += 1;
        }
        if got != c.expected {
            eprintln!(
                "FAIL {}: datalog {} != hand-checked {}",
                c.vector_id, got, c.expected
            );
            failures += 1;
        }

        let path = dir.join(format!("{}.json", c.vector_id));
        if verify {
            let disk = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("missing vector {}: {e}", path.display()));
            let v: Value = serde_json::from_str(&disk).expect("vector JSON parses");
            let exp_verdict = v["expected"]["verdicts"]["dsep_query"]
                .as_str()
                .expect("verdict");
            let want = format!(
                "dsep:{},{}|{}={}",
                c.start,
                c.end,
                c.obs.join(","),
                got == "dsep"
            );
            let fix_sorted: Vec<String> = fix.into_iter().collect();
            let disk_fix: Vec<String> = v["provenance"]["datalog_program"]["fixpoint"]
                .as_array()
                .expect("fixpoint array")
                .iter()
                .map(|f| f.as_str().expect("fact").to_string())
                .collect();
            if exp_verdict != want || fix_sorted != disk_fix {
                eprintln!("VERIFY FAIL {c}: {exp_verdict} vs {want}", c = c.vector_id);
                failures += 1;
            }
            let disk_rev = v["provenance"]["graphlaw_rev"].as_str().expect("rev");
            if disk_rev != rev {
                eprintln!("VERIFY FAIL {}: rev {disk_rev} != {rev}", c.vector_id);
                failures += 1;
            }
        } else {
            let verdict_value = format!(
                "dsep:{},{}|{}={}",
                c.start,
                c.end,
                c.obs.join(","),
                got == "dsep"
            );
            let vector = json!({
                "vector_id": c.vector_id,
                "breed": "bayesian_network",
                "provenance": {
                    "graphlaw_rev": rev,
                    "purrdf": PURRDF,
                    "eyeron": EYERON,
                    "generator": "bayes_pop_vectors",
                    "stratified": true,
                    "datalog_program": {
                        "rules": dsep_rule_specs(),
                        "fixpoint": fix.iter().collect::<Vec<_>>(),
                    },
                    "differential": "native Bayes-ball port of breeds/bayesian_network.rs dsep must agree with the Datalog court verdict and the hand-checked expectation at generation time",
                },
                "breed_input": breed_input,
                "expected": {
                    "derived_facts_sorted": [],
                    "verdicts": {"dsep_query": verdict_value},
                    "notes": c.notes,
                },
            });
            fs::write(&path, serde_json::to_string_pretty(&vector).unwrap() + "\n")
                .expect("write vector");
            println!("wrote {} (verdict {got})", path.display());
            written += 1;
        }
    }

    let pop = pop_cases();
    for c in &pop {
        let (facts, breed_input) = pop_edb(c);
        let fix = eval_fixpoint(pop_rules(), &facts.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        if std::env::var_os("GL_DEBUG").is_some() {
            eprintln!("--- {} offending facts:", c.vector_id);
            for f in &fix {
                let (pr, _) = f.split_once(':').unwrap();
                if matches!(pr, "cyclic" | "unsupp" | "threat") {
                    eprintln!("  {f}");
                }
            }
        }
        let got = plan_verdict(&fix);
        let expected_valid = c.expected != "refused";
        if (got == "valid") != expected_valid {
            eprintln!(
                "FAIL {}: datalog {} != hand-checked {}",
                c.vector_id, got, c.expected
            );
            failures += 1;
        }
        let path = dir.join(format!("{}.json", c.vector_id));
        if verify {
            let disk = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("missing vector {}: {e}", path.display()));
            let v: Value = serde_json::from_str(&disk).expect("vector JSON parses");
            let exp_verdict = v["expected"]["verdicts"]["plan"].as_str().expect("verdict");
            let fix_sorted: Vec<String> = fix.into_iter().collect();
            let disk_fix: Vec<String> = v["provenance"]["datalog_program"]["fixpoint"]
                .as_array()
                .expect("fixpoint array")
                .iter()
                .map(|f| f.as_str().expect("fact").to_string())
                .collect();
            if exp_verdict != c.expected || fix_sorted != disk_fix {
                eprintln!("VERIFY FAIL {}: {} vs {}", c.vector_id, exp_verdict, c.expected);
                failures += 1;
            }
            let disk_rev = v["provenance"]["graphlaw_rev"].as_str().expect("rev");
            if disk_rev != rev {
                eprintln!("VERIFY FAIL {}: rev {disk_rev} != {rev}", c.vector_id);
                failures += 1;
            }
} else {
            let vector = json!({
                "vector_id": c.vector_id,
                "breed": "partial_order_plan",
                "provenance": {
                    "graphlaw_rev": rev,
                    "purrdf": PURRDF,
                    "eyeron": EYERON,
                    "generator": "bayes_pop_vectors",
                    "stratified": true,
                    "datalog_program": {
                        "rules": pop_rule_specs(),
                        "fixpoint": fix.iter().collect::<Vec<_>>(),
                    },
                    "differential": "the Datalog validation court (ordtc closure, acyclicity, support, threats) must reproduce the hand-checked verdict at generation time",
                },
                "breed_input": breed_input,
                "expected": {
                    "derived_facts_sorted": [],
                    "verdicts": {"plan": c.expected},
                    "notes": c.notes,
                },
            });
            fs::write(&path, serde_json::to_string_pretty(&vector).unwrap() + "\n")
                .expect("write vector");
            println!("wrote {} (verdict {got})", path.display());
            written += 1;
        }
    }

    if failures > 0 {
        panic!("{failures} failure(s)");
    }
    if verify {
        println!(
            "verify: {} vector(s) re-evaluated and identical; rev {rev}",
            dsep.len() + pop.len()
        );
    } else {
        println!("generated {written} vector(s); rev {rev}; now run with --verify");
    }
}
