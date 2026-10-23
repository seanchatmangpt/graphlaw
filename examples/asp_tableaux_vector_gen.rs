//! asp_tableaux_vector_gen — vector GENERATOR for the ASP and TABLEAUX breeds,
//! driving purrdf-datalog DIRECTLY (not through graphlaw's JSON ABI, whose
//! datalog shim is positive-only) so that NAF premises are expressible via
//! [`graphlaw::datalog::clause::ClauseAtom::negated`].
//!
//! ASP ground truth: only the STRATIFIED subset is oracle-admissible
//! (unique stable model == stratified least model). The generator runs
//! [`graphlaw::datalog::plan::stratify`] over every candidate program and
//! REFUSES non-stratifiable programs — the refusal is documented generator
//! behavior, printed and skipped, never emitted as a vector.
//!
//! TABLEAUX ground truth: for each propositional formula the generator
//! enumerates ALL valuations (brute force, independent of both graphlaw and
//! wasm4pm), derives valid/invalid, and for an invalid formula records a
//! falsifying witness valuation. The emitted vector asserts the breed's
//! `tableaux:verdict` matches and that the frozen countermodel assignment
//! falsifies the formula (re-checked in `--verify` by independent evaluation).
//!
//! Encoding (propositional ASP atom p → datalog quad):
//!   p  ↦ (subject=p, predicate="prop", object="true") in the default graph
//!   rule h :- b1, not b2 ↦ DlClause::datalog(head(h), [pos(b1), neg(b2)])
//!   fact p ↦ EDB row (p, "prop", "true")
//!
//! Modes:
//!   (no args)  generate vectors into $GL_VECDIR (default: wasm4pm cognition
//!              fixtures/graphlaw/), printing any stratifiability refusals
//!   --verify   re-evaluate every emitted ASP vector with the native engine
//!              and every tableau vector against brute-force ground truth,
//!              byte-comparing expectations (independent second run)
//!
//! Run: cargo run --example asp_tableaux_vector_gen [-- --verify]
//! (from /Users/sac/graphlaw)
//! Output: generation log ending `generated N vectors; rev <sha>; now run with
//! --verify` (exit 0); `--verify` re-evaluates every emitted vector and exits 0
//! on byte-agreement.
//! Court: tests/corpus_conformance.rs::vecgen_asp_tableaux_generate_then_verify.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use graphlaw::datalog::clause::{ClauseAtom, ClauseTerm, DlClause};
use graphlaw::datalog::plan::stratify;
use graphlaw::datalog::seminaive;
use graphlaw::datalog::store::RelationStore;
use serde_json::json;

const PURRDF: &str = "2.0.2";
const EYERON: &str = "0.7.8";
const SUBJ: &str = "t";
const TRUE_OBJ: &str = "true";

// ------------------------------------------------------------------ ASP encoding

/// A propositional ASP program: facts + rules with `not` NAF premises.
struct AspCase {
    vector_id: &'static str,
    intent: &'static str,
    facts: &'static [&'static str],
    /// (conclusion, premises); a premise starting with "not " is NAF.
    rules: &'static [(&'static str, &'static [&'static str])],
    notes: &'static str,
}

/// A propositional atom p is the quad (subject="t", predicate=p, object="true").
/// The ATOM NAME lives in the PREDICATE position: stratify's dependency graph
/// keys on predicate symbols, so a subject-side encoding would collapse every
/// atom onto one symbol and mis-report stratifiability.
fn pos_atom(name: &str) -> ClauseAtom {
    ClauseAtom::quad(
        ClauseTerm::literal(SUBJ),
        ClauseTerm::literal(name),
        ClauseTerm::literal(TRUE_OBJ),
        ClauseTerm::default_graph(),
    )
}

fn neg_atom(name: &str) -> ClauseAtom {
    ClauseAtom::negated_quad(
        ClauseTerm::literal(SUBJ),
        ClauseTerm::literal(name),
        ClauseTerm::literal(TRUE_OBJ),
        ClauseTerm::default_graph(),
    )
}

fn head_atom(name: &str) -> ClauseAtom {
    pos_atom(name)
}

/// Translate a propositional ASP program into purrdf-datalog clauses.
fn asp_to_clauses(facts: &[&str], rules: &[(&str, &[&str])]) -> Vec<DlClause> {
    let mut clauses = Vec::new();
    // A fact is a zero-body rule.
    for f in facts {
        clauses.push(DlClause::datalog(head_atom(f), vec![]));
    }
    for (conclusion, premises) in rules {
        let body = premises
            .iter()
            .map(|p| match p.strip_prefix("not ") {
                Some(atom) => neg_atom(atom.trim()),
                None => pos_atom(p),
            })
            .collect();
        clauses.push(DlClause::datalog(head_atom(conclusion), body));
    }
    clauses
}

/// Stratifiability via purrdf's relaxation (`None` = a negative edge sits in a
/// cycle: an odd OR even NAF cycle — both are outside the oracle-admissible
/// stratified subset).
fn stratified(rules: &[DlClause]) -> bool {
    stratify(rules).is_some()
}

/// Stratified least model via purrdf-datalog (compile → evaluate → facts).
/// Returns the set of TRUE atom names.
fn least_model(facts: &[&str], rules: &[DlClause]) -> BTreeSet<String> {
    let exe = seminaive::compile(rules.to_vec()).expect("program compiles");
    let mut edb = RelationStore::new();
    for f in facts {
        edb.insert(SUBJ, f, TRUE_OBJ, RelationStore::DEFAULT_GRAPH);
    }
    let eval = seminaive::evaluate(&exe, edb).expect("evaluation succeeds");
    let store = eval.facts();
    let mut model = BTreeSet::new();
    for fact in store.facts_sorted() {
        // Encoding: (subject="t", predicate=atom, object="true", graph="").
        if fact.subject == SUBJ && fact.object == TRUE_OBJ && fact.graph.is_empty() {
            model.insert(fact.predicate.clone());
        }
    }
    model
}

/// Emit an ASP vector. The `expected.derived_facts_sorted` entries are
/// `key=value` strings evidenced in the breed output haystack:
/// `asp:answer_set_count`, `stable_models_count`, `model_0_i`.
fn emit_asp(case: &AspCase, rev: &str, dir: &std::path::Path) {
    let clauses = asp_to_clauses(case.facts, case.rules);
    if !stratified(&clauses) {
        // Documented generator refusal: outside the oracle-admissible subset.
        println!(
            "REFUSED (non-stratifiable): {} — negative edge in a dependency cycle; \
             no vector emitted",
            case.vector_id
        );
        return;
    }
    let model = least_model(case.facts, &clauses);
    // Stratified ⇒ unique stable model; the breed must report count 1 and its
    // first (only) model must equal the stratified least model.
    let mut expected = vec![
        "asp:answer_set_count=1".to_string(),
        "stable_models_count=1".to_string(),
    ];
    for (i, atom) in model.iter().enumerate() {
        expected.push(format!("model_0_{i}={atom}"));
    }
    expected.sort();

    let breed_input = json!({
        "intent": case.intent,
        "facts": case.facts.iter().map(|k| json!({"key": k, "value": ""})).collect::<Vec<_>>(),
        "rules": case.rules.iter().enumerate().map(|(i, (concl, prem))| json!({
            "id": format!("r{}", i + 1),
            "premise": prem.iter().map(|p| json!(p)).collect::<Vec<_>>(),
            "conclusion": concl,
            "certainty": 1.0,
        })).collect::<Vec<_>>(),
        "goals": [],
        "candidates": [],
        "cases": [],
        "state": [],
    });

    let vector = json!({
        "vector_id": case.vector_id,
        "breed": "asp",
        "provenance": {
            "graphlaw_rev": rev,
            "purrdf": PURRDF,
            "eyeron": EYERON,
            "generator": "asp_tableaux_vector_gen (purrdf-datalog stratified least model; \
                          stratifiability enforced via graphlaw::datalog::plan::stratify)",
            "stratified": true,
        },
        "breed_input": breed_input,
        "expected": {
            "derived_facts_sorted": expected,
            "verdicts": {},
            "notes": case.notes,
        },
    });
    let path = dir.join(format!("{}.json", case.vector_id));
    fs::write(&path, serde_json::to_string_pretty(&vector).unwrap() + "\n").expect("write vector");
    println!("wrote {} (|model|={})", path.display(), model.len());
}

/// The deliberate NON-stratifiable inputs. The generator must refuse these;
/// the printed refusal is the witnessed falsifier for the stratifiability guard.
fn refusal_cases() -> Vec<(&'static str, Vec<DlClause>)> {
    vec![
        (
            "odd NAF cycle (p :- not q. q :- p.)",
            vec![
                DlClause::datalog(head_atom("p"), vec![neg_atom("q")]),
                DlClause::datalog(head_atom("q"), vec![pos_atom("p")]),
            ],
        ),
        (
            "even NAF cycle (p :- not q. q :- not p.)",
            vec![
                DlClause::datalog(head_atom("p"), vec![neg_atom("q")]),
                DlClause::datalog(head_atom("q"), vec![neg_atom("p")]),
            ],
        ),
    ]
}

// -------------------------------------------------------------- TABLEAUX encoding

/// Independent propositional formula AST (graphlaw-free, wasm4pm-free).
#[derive(Clone, Debug)]
enum F {
    Atom(String),
    Not(Box<F>),
    And(Box<F>, Box<F>),
    Or(Box<F>, Box<F>),
    Implies(Box<F>, Box<F>),
}

/// Fully-parenthesized recursive-descent parser over atoms, `!`, `&`, `|`, `->`,
/// parentheses. Ground truth is only ever computed over fully parenthesized
/// formulas, so no precedence/associativity convention is load-bearing.
struct P<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> P<'a> {
    fn parse(s: &'a str) -> F {
        let mut p = P {
            s: s.as_bytes(),
            i: 0,
        };
        let f = p.formula();
        assert!(p.i == p.s.len(), "trailing input at byte {}", p.i);
        f
    }
    fn skip(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    /// formula := '(' formula OP formula ')' | '!' formula | atom
    fn formula(&mut self) -> F {
        self.skip();
        match self.s.get(self.i) {
            Some(b'(') => {
                self.i += 1;
                let a = self.formula();
                let op = self.op();
                self.i += op.len();
                let b = self.formula();
                self.skip();
                assert!(self.s.get(self.i) == Some(&b')'), "expected ')'");
                self.i += 1;
                match op {
                    "&" => F::And(Box::new(a), Box::new(b)),
                    "|" => F::Or(Box::new(a), Box::new(b)),
                    "->" => F::Implies(Box::new(a), Box::new(b)),
                    other => panic!("unknown operator {other:?}"),
                }
            }
            Some(b'!') => {
                self.i += 1;
                F::Not(Box::new(self.formula()))
            }
            Some(c) if c.is_ascii_alphanumeric() || *c == b'_' => {
                let start = self.i;
                while self.i < self.s.len()
                    && (self.s[self.i].is_ascii_alphanumeric() || self.s[self.i] == b'_')
                {
                    self.i += 1;
                }
                F::Atom(String::from_utf8(self.s[start..self.i].to_vec()).expect("utf8 atom"))
            }
            other => panic!("unexpected {:?} at byte {}", other, self.i),
        }
    }
    fn op(&mut self) -> &'static str {
        self.skip();
        for candidate in ["->", "&", "|"] {
            if self.s[self.i..].starts_with(candidate.as_bytes()) {
                return candidate;
            }
        }
        panic!("expected operator at byte {}", self.i);
    }
}

/// Parse via the grammar above (no precedence/associativity convention is
/// load-bearing — every compound formula is parenthesized).
fn parse_formula(src: &str) -> F {
    P::parse(src)
}

fn atoms_of(f: &F, out: &mut BTreeSet<String>) {
    match f {
        F::Atom(a) => {
            out.insert(a.clone());
        }
        F::Not(a) => atoms_of(a, out),
        F::And(a, b) | F::Or(a, b) | F::Implies(a, b) => {
            atoms_of(a, out);
            atoms_of(b, out);
        }
    }
}

/// Brute-force evaluation of `f` under valuation `v` (missing = false).
fn eval(f: &F, v: &BTreeMap<String, bool>) -> bool {
    match f {
        F::Atom(a) => v.get(a).copied().unwrap_or(false),
        F::Not(a) => !eval(a, v),
        F::And(a, b) => eval(a, v) && eval(b, v),
        F::Or(a, b) => eval(a, v) || eval(b, v),
        F::Implies(a, b) => !eval(a, v) || eval(b, v),
    }
}

/// Ground truth: enumerate ALL valuations over the formula's atoms.
/// Returns (valid, first falsifying valuation as atom→bool, if invalid).
fn ground_truth(src: &str) -> (bool, Option<BTreeMap<String, bool>>) {
    let f = parse_formula(src);
    let mut atoms = BTreeSet::new();
    atoms_of(&f, &mut atoms);
    let atoms: Vec<String> = atoms.into_iter().collect();
    let n = atoms.len();
    for mask in 0..(1usize << n) {
        let v: BTreeMap<String, bool> = atoms
            .iter()
            .enumerate()
            .map(|(i, a)| (a.clone(), (mask >> i) & 1 == 1))
            .collect();
        if !eval(&f, &v) {
            return (false, Some(v));
        }
    }
    (true, None)
}

struct TableauxCase {
    vector_id: &'static str,
    /// Fully parenthesized formula in the wasm4pm tableaux syntax
    /// (atoms, `!`, `&`, `|`, `->`).
    formula: &'static str,
    notes: &'static str,
}

/// Countermodel fact string as the wasm4pm tableaux breed emits it:
/// `tableaux:countermodel:<atom>=<true|false>`.
fn cm_fact(atom: &str, v: bool) -> String {
    format!("tableaux:countermodel:{atom}={v}")
}

fn emit_tableaux(case: &TableauxCase, rev: &str, dir: &std::path::Path) {
    let (valid, witness) = ground_truth(case.formula);
    let mut expected = vec![format!(
        "tableaux:verdict={}",
        if valid { "valid" } else { "invalid" }
    )];
    if let Some(w) = &witness {
        for (a, v) in w {
            expected.push(cm_fact(a, *v));
        }
    }
    expected.sort();

    let breed_input = json!({
        "intent": format!("propositional validity: {}", case.formula),
        "facts": [{"key": "tableaux:formula", "value": case.formula}],
        "rules": [],
        "goals": [],
        "candidates": [],
        "cases": [],
        "state": [],
    });

    let vector = json!({
        "vector_id": case.vector_id,
        "breed": "tableaux",
        "provenance": {
            "graphlaw_rev": rev,
            "purrdf": PURRDF,
            "eyeron": EYERON,
            "generator": "asp_tableaux_vector_gen (brute-force valuation enumeration over all \
                          atoms; independent of both graphlaw and wasm4pm formula machinery)",
            "stratified": false,
        },
        "breed_input": breed_input,
        "expected": {
            "derived_facts_sorted": expected,
            "verdicts": {},
            "notes": case.notes,
        },
    });
    let path = dir.join(format!("{}.json", case.vector_id));
    fs::write(&path, serde_json::to_string_pretty(&vector).unwrap() + "\n").expect("write vector");
    println!(
        "wrote {} ({}; witness {})",
        path.display(),
        if valid { "VALID" } else { "INVALID" },
        witness
            .as_ref()
            .map(|w| w
                .iter()
                .map(|(a, v)| format!("{a}={v}"))
                .collect::<Vec<_>>()
                .join(","))
            .as_deref()
            .unwrap_or("-")
    );
}

// --------------------------------------------------------------------- verify

fn verify(rev: &str, dir: &std::path::Path) {
    let mut failures = 0usize;

    // ASP: re-run the native stratified least model and byte-compare.
    for case in asp_cases() {
        let path = dir.join(format!("{}.json", case.vector_id));
        let disk = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("VERIFY FAIL {}: unreadable: {e}", path.display());
                failures += 1;
                continue;
            }
        };
        let v: serde_json::Value = serde_json::from_str(&disk).expect("vector JSON parses");
        let clauses = asp_to_clauses(case.facts, case.rules);
        if !stratified(&clauses) {
            eprintln!("VERIFY FAIL {}: program must be stratified", case.vector_id);
            failures += 1;
            continue;
        }
        let model = least_model(case.facts, &clauses);
        let mut exp: Vec<String> = vec![
            "asp:answer_set_count=1".into(),
            "stable_models_count=1".into(),
        ];
        for (i, atom) in model.iter().enumerate() {
            exp.push(format!("model_0_{i}={atom}"));
        }
        exp.sort();
        let disk_exp: Vec<String> = v["expected"]["derived_facts_sorted"]
            .as_array()
            .expect("derived_facts_sorted")
            .iter()
            .map(|x| x.as_str().expect("string").to_string())
            .collect();
        if disk_exp != exp {
            eprintln!(
                "VERIFY FAIL {}: expectations differ\n  disk: {:?}\n  now:  {:?}",
                case.vector_id, disk_exp, exp
            );
            failures += 1;
        }
        let disk_rev = v["provenance"]["graphlaw_rev"].as_str().expect("rev");
        if disk_rev != rev {
            eprintln!(
                "VERIFY FAIL {}: provenance rev {disk_rev} != current {rev}",
                case.vector_id
            );
            failures += 1;
        }
    }

    // Tableaux: verdict matches brute force; frozen countermodel (if invalid)
    // is a real falsifying valuation; valid verdict implies none exists.
    for case in tableaux_cases() {
        let path = dir.join(format!("{}.json", case.vector_id));
        let disk = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("VERIFY FAIL {}: unreadable: {e}", path.display());
                failures += 1;
                continue;
            }
        };
        let v: serde_json::Value = serde_json::from_str(&disk).expect("vector JSON parses");
        let (valid, witness) = ground_truth(case.formula);
        let mut exp: Vec<String> = vec![format!(
            "tableaux:verdict={}",
            if valid { "valid" } else { "invalid" }
        )];
        if let Some(w) = &witness {
            for (a, b) in w {
                exp.push(cm_fact(a, *b));
            }
        }
        exp.sort();
        let disk_exp: Vec<String> = v["expected"]["derived_facts_sorted"]
            .as_array()
            .expect("derived_facts_sorted")
            .iter()
            .map(|x| x.as_str().expect("string").to_string())
            .collect();
        if disk_exp != exp {
            eprintln!(
                "VERIFY FAIL {}: expectations differ\n  disk: {:?}\n  brute force: {:?}",
                case.vector_id, disk_exp, exp
            );
            failures += 1;
            continue;
        }
        // Extra court: a frozen countermodel must actually falsify the formula.
        if !valid {
            let f = parse_formula(case.formula);
            let mut valuation = BTreeMap::new();
            for entry in &disk_exp {
                if let Some(rest) = entry.strip_prefix("tableaux:countermodel:") {
                    let (atom, val) = rest.rsplit_once('=').expect("countermodel k=v");
                    valuation.insert(
                        atom.to_string(),
                        match val {
                            "true" => true,
                            "false" => false,
                            other => panic!("bad countermodel value {other:?}"),
                        },
                    );
                }
            }
            if eval(&f, &valuation) {
                eprintln!(
                    "VERIFY FAIL {}: frozen countermodel does NOT falsify the formula",
                    case.vector_id
                );
                failures += 1;
            }
        }
        let disk_rev = v["provenance"]["graphlaw_rev"].as_str().expect("rev");
        if disk_rev != rev {
            eprintln!(
                "VERIFY FAIL {}: provenance rev {disk_rev} != current {rev}",
                case.vector_id
            );
            failures += 1;
        }
    }

    if failures > 0 {
        panic!("{failures} vector(s) failed independent re-verification");
    }
    println!("verify: all ASP + tableaux vectors re-derived and byte-identical; rev {rev}");
}

// ---------------------------------------------------------------------- cases

fn asp_cases() -> Vec<AspCase> {
    vec![
        // (a) paper fixture (Kowalski 1974) as a propositional ASP program:
        // facts only — no NAF anywhere, so trivially stratifiable. The check
        // ran; the program is in the admissible subset, so the vector is emitted.
        AspCase {
            vector_id: "asp-paper-fixture",
            intent: "stable model of the paper family fact base (facts-only program)",
            facts: &["parent:tom,bob", "parent:bob,ann", "parent:bob,pat"],
            rules: &[],
            notes: "Paper fixture (Kowalski 1974 Fig. 2) as a facts-only propositional ASP \
                    program. Stratifiability CHECKED (no NAF — trivially stratified). The unique \
                    stable model is the fact set itself; asp:answer_set_count=1.",
        },
        // (b) NAF-as-guard chain: q :- p. p fact. r :- not p.
        AspCase {
            vector_id: "asp-naf-guard-chain",
            intent: "NAF as a guard: r fires only if p is absent",
            facts: &["p"],
            rules: &[("q", &["p"]), ("r", &["not p"])],
            notes: "Stratified NAF guard chain (q :- p, p fact, r :- not p). Stratified least \
                    model = {p, q}: p derives q; r's NAF guard blocks because p holds. Unique \
                    stable model.",
        },
        // (c) recursion with NAF: reachable :- edge, not blocked + positive recursion.
        AspCase {
            vector_id: "asp-recursive-naf-guard",
            intent: "recursion guarded by NAF: reachable :- edge, not blocked",
            facts: &["edge"],
            rules: &[
                ("reachable", &["edge", "not blocked"]),
                ("reach_link", &["edge"]),
                ("reachable", &["reach_link"]),
            ],
            notes: "Recursion with NAF (reachable :- edge, not blocked; reachable :- reach_link \
                    :- edge). blocked is EDB-only and absent, so the guard passes and reachable \
                    is derived; the recursion on reachable is positive-only, so the program is \
                    stratified with a unique stable model {edge, reach_link, reachable}.",
        },
    ]
}

fn tableaux_cases() -> Vec<TableauxCase> {
    vec![
        // modus-ponens chain, valid (alpha-only tableau, zero beta expansions).
        TableauxCase {
            vector_id: "tableaux-mp-chain",
            formula: "((a & (a -> b)) -> b)",
            notes: "Modus ponens chain, VALID by brute force over {a,b}: no valuation falsifies \
                    (a & (a -> b)) -> b. Tableau closes on every branch (alpha-only).",
        },
        // the paper fixture, propositional MP instance over the family facts.
        TableauxCase {
            vector_id: "tableaux-paper-fixture",
            formula: "((parent_tom_bob & (parent_tom_bob -> parent_bob_ann)) -> parent_bob_ann)",
            notes: "Paper fixture (Kowalski 1974) as a propositional modus-ponens instance over \
                    the family facts: VALID by brute force over \
                    {parent_tom_bob, parent_bob_ann}.",
        },
        // pigeonhole-flavoured INVALID with a falsifying witness valuation.
        TableauxCase {
            vector_id: "tableaux-pigeonhole-invalid",
            formula: "(((p1h1 | p1h2) & (p2h1 | p2h2)) -> (p1h1 & p2h1))",
            notes: "Pigeonhole-flavoured INVALID claim: 'if each of two pigeons takes one of two \
                    holes then some hole holds both pigeons' is falsified by the witness \
                    valuation p1h1=false, p1h2=true, p2h1=true, p2h2=false. The tableau must \
                    report invalid and its countermodel assignment must falsify the formula.",
        },
    ]
}

// ----------------------------------------------------------------------- main

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verify_mode = args.iter().any(|a| a == "--verify");

    let rev = {
        let out = std::process::Command::new("git")
            .args(["-C", env!("CARGO_MANIFEST_DIR"), "rev-parse", "HEAD"])
            .output()
            .expect("git rev-parse");
        String::from_utf8(out.stdout)
            .expect("utf8")
            .trim()
            .to_string()
    };

    let dir = vecdir();
    if verify_mode {
        verify(&rev, &dir);
        return;
    }

    fs::create_dir_all(&dir).expect("create vecdir");

    // The stratifiability guard must FIRE on the deliberate non-stratifiable
    // inputs — witnessed here, on the real engine, every generation run.
    for (label, clauses) in refusal_cases() {
        assert!(
            !stratified(&clauses),
            "stratifiability guard did NOT fire on: {label}"
        );
        println!("REFUSED (non-stratifiable): {label} — no vector emitted");
    }

    for case in asp_cases() {
        emit_asp(&case, &rev, &dir);
    }
    for case in tableaux_cases() {
        emit_tableaux(&case, &rev, &dir);
    }
    println!("generated vectors; rev {rev}; now run with --verify");
}

fn vecdir() -> PathBuf {
    std::env::var_os("GL_VECDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw")
        })
}
