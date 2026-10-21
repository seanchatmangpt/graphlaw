//! DL -> DATALOG / OWL-RL dual-route vector generator.
//!
//! Translates the wasm4pm `description_logic` breed's Surface-3 grammar
//! (`dl:subclass:` / `dl:conj:` / `dl:exists_rhs:` / `dl:exists_lhs:` axioms;
//! goals `dl:subsumes` "A:B") into two independent derivations:
//!
//! * Route A (DATALOG): per-axiom N3 rules over `dl#sub` / `dl#edge_<r>` —
//!   CR1 transitivity rule, CR2 per-conj-axiom rule, CR3 exists_rhs -> edge
//!   rules, CR4 exists_lhs rules. Run to fixpoint by Eyeron `Step::DeriveN3`.
//! * Route B (OWL-RL): `dl:subclass` -> `rdfs:subClassOf`, exists ->
//!   `owl:someValuesFrom` restrictions, conj -> `owl:intersectionOf`. Run by
//!   the purrdf OWL-RL chase via `Step::EntailOwlRl`. OWL-RL classifies at
//!   the instance level, so each named concept carries a witness individual
//!   `dl:w_<C>` and verdicts are read from the derived type closure of the
//!   witnesses (mirroring the breed's S(A) sets exactly).
//!
//! A vector is emitted ONLY when the two routes agree on every verdict for a
//! case; on disagreement the generator refuses for that case and reports the
//! semantic gap instead of a vector. Exit 0 iff all cases agreed and the
//! paper-fixture cross-check passed.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawState, Step};
use graphlaw::tier::validate_n3_rules;
use graphlaw::rdf::{SparqlEngine, SparqlRequest, SparqlResult, TermValue};
use graphlaw::sparql::NativeSparqlEngine;
use std::collections::BTreeSet;
use std::fmt::Write as _;

const DL: &str = "https://graphlaw.dev/dl#";

struct Case {
    slug: &'static str,
    facts: Vec<(String, String)>,
    goals: Vec<&'static str>,
    notes: &'static str,
}

fn f(key: &str, value: &str) -> (String, String) {
    (key.to_string(), value.to_string())
}

#[derive(Default)]
struct Parsed {
    subclass: Vec<(String, String)>,
    conj: Vec<(String, String, String)>,
    exists_rhs: Vec<(String, String, String)>, // (A, r, B)
    exists_lhs: Vec<(String, String, String)>, // (r, A, B)
    concepts: BTreeSet<String>,
}

fn parse(facts: &[(String, String)]) -> Result<Parsed, String> {
    let mut p = Parsed::default();
    for (key, value) in facts {
        if let Some(a) = key.strip_prefix("dl:subclass:") {
            p.concepts.insert(a.to_string());
            p.concepts.insert(value.clone());
            p.subclass.push((a.to_string(), value.clone()));
        } else if let Some(rest) = key.strip_prefix("dl:conj:") {
            let (a1, a2) = rest
                .split_once('+')
                .ok_or_else(|| format!("malformed dl:conj key '{key}'"))?;
            p.concepts.insert(a1.to_string());
            p.concepts.insert(a2.to_string());
            p.concepts.insert(value.clone());
            p.conj.push((a1.to_string(), a2.to_string(), value.clone()));
        } else if let Some(a) = key.strip_prefix("dl:exists_rhs:") {
            let (r, b) = value
                .split_once('.')
                .ok_or_else(|| format!("malformed dl:exists_rhs value '{value}'"))?;
            p.concepts.insert(a.to_string());
            p.concepts.insert(b.to_string());
            p.exists_rhs.push((a.to_string(), r.to_string(), b.to_string()));
        } else if let Some(rest) = key.strip_prefix("dl:exists_lhs:") {
            let (r, a) = rest
                .split_once('.')
                .ok_or_else(|| format!("malformed dl:exists_lhs key '{key}'"))?;
            p.concepts.insert(a.to_string());
            p.concepts.insert(value.clone());
            p.exists_lhs.push((r.to_string(), a.to_string(), value.clone()));
        }
    }
    if p.subclass.len() + p.conj.len() + p.exists_rhs.len() + p.exists_lhs.len() == 0 {
        return Err("no dl:* axioms".into());
    }
    Ok(p)
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            slug: "a-paper-baader-el",
            facts: vec![
                f("dl:subclass:Pericarditis", "Inflammation"),
                f("dl:subclass:Inflammation", "Disease"),
                f("dl:exists_rhs:Pericarditis", "has_location.Heart"),
                f("dl:exists_lhs:has_location.Heart", "HasLocationHeart"),
                f("dl:conj:Disease+HasLocationHeart", "HeartDisease"),
            ],
            goals: vec!["Pericarditis:HeartDisease", "HeartDisease:Pericarditis"],
            notes: "the paper fixture (Baader/Brandt/Lutz EL example); all four CRs fire",
        },
        Case {
            slug: "b-chain3-conj",
            facts: vec![
                f("dl:subclass:Puppy", "Dog"),
                f("dl:subclass:Dog", "Mammal"),
                f("dl:subclass:Mammal", "Animal"),
                f("dl:subclass:Dog", "Pet"),
                f("dl:conj:Mammal+Pet", "HousePet"),
            ],
            goals: vec!["Puppy:HousePet", "Dog:HousePet", "HousePet:Dog", "Mammal:HousePet"],
            notes: "3-level chain + CR2 conjunction",
        },
        Case {
            slug: "c-exists-pair",
            facts: vec![
                f("dl:subclass:Transfer", "Event"),
                f("dl:exists_rhs:Transfer", "involves.Account"),
                f("dl:exists_lhs:involves.Account", "LedgerEntry"),
            ],
            goals: vec!["Transfer:LedgerEntry", "LedgerEntry:Transfer", "Transfer:Event"],
            notes: "exists_rhs + exists_lhs (CR3 then CR4), plus asserted and negative verdicts",
        },
        Case {
            slug: "d-exists-conj-interlock",
            facts: vec![
                f("dl:subclass:x_X", "x_W"),
                f("dl:exists_rhs:x_X", "r.Y"),
                f("dl:exists_lhs:r.Y", "x_Z"),
                f("dl:conj:x_Z+x_W", "x_V"),
                f("dl:exists_rhs:x_Z", "s.Y2"),
                f("dl:exists_lhs:s.Y2", "x_V2"),
                f("dl:conj:x_V2+x_W", "x_V"),
            ],
            goals: vec!["x_X:x_V", "x_V:x_X", "x_Z:x_V", "x_Z:x_V2", "x_V2:x_Z", "x_V2:x_W"],
            notes: "adversarial: CR3->CR4->CR2 interlock run twice over two roles",
        },
    ]
}

fn iri(name: &str) -> String {
    format!("{DL}{name}")
}

// ---------------------------------------------------------------------------
// Route A: DL -> DATALOG (N3) over dl#sub / dl#edge_<r>
// ---------------------------------------------------------------------------

fn route_a(p: &Parsed, goals: &[&str]) -> Result<Vec<bool>, String> {
    let mut base = String::new();
    let mut rules = String::from("@prefix dl: <https://graphlaw.dev/dl#> .\n\n");
    let _ = writeln!(
        rules,
        "# CR1 transitivity\n{{ ?a dl:sub ?b . ?b dl:sub ?c }} => {{ ?a dl:sub ?c }} .\n"
    );
    for c in &p.concepts {
        let _ = writeln!(base, "<{}> <{DL}sub> <{}> .", iri(c), iri(c));
    }
    for (a, b) in &p.subclass {
        let _ = writeln!(base, "<{}> <{DL}sub> <{}> .", iri(a), iri(b));
    }
    for (i, (a1, a2, b)) in p.conj.iter().enumerate() {
        let _ = writeln!(
            rules,
            "# CR2 conj[{i}] {a1} n{a2} sub {b}\n{{ ?x dl:sub <{}> . ?x dl:sub <{}> }} => {{ ?x dl:sub <{}> }} .\n",
            iri(a1), iri(a2), iri(b)
        );
    }
    for (i, (a, r, b)) in p.exists_rhs.iter().enumerate() {
        let _ = writeln!(
            rules,
            "# CR3 exists_rhs[{i}] {a} sub exists {r}.{b}\n{{ ?x dl:sub <{}> }} => {{ ?x dl:edge_{r} <{}> }} .\n",
            iri(a), iri(b)
        );
    }
    for (i, (r, a, b)) in p.exists_lhs.iter().enumerate() {
        let _ = writeln!(
            rules,
            "# CR4 exists_lhs[{i}] exists {r}.{a} sub {b}\n{{ ?x dl:edge_{r} <{}> }} => {{ ?x dl:sub <{}> }} .\n",
            iri(a), iri(b)
        );
    }

    let s0 = LawState::parse(base.as_bytes(), Dialect::Turtle, None)
        .map_err(|e| format!("route A parse: {e:?}"))?;
    let ruleset = validate_n3_rules(rules.as_bytes(), None)
        .map_err(|e| format!("route A rules: {e:?}"))?;
    let (s1, _r) = s0
        .transition(&Step::DeriveN3 { rules: &rules, ruleset: &ruleset })
        .map_err(|e| format!("route A derive: {e:?}"))?;

    // Closure: all sub pairs.
    let mut closure: BTreeSet<(String, String)> = BTreeSet::new();
    for row in select(&s1, "SELECT ?a ?b WHERE { ?a <https://graphlaw.dev/dl#sub> ?b }")? {
        let (Some(a), Some(b)) = (row.first().cloned(), row.get(1).cloned())
        else {
            continue;
        };
        closure.insert((a, b));
    }
    Ok(goals.iter().map(|g| eval_goal(g, &closure, &p.concepts)).collect())
}

fn select(state: &LawState, q: &str) -> Result<Vec<Vec<String>>, String> {
    match NativeSparqlEngine::new().query(
        state.dataset(),
        SparqlRequest { query: q, base_iri: None, substitutions: &[] },
    ) {
        Ok(SparqlResult::Solutions { rows, .. }) => Ok(rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|t| match t {
                        Some(TermValue::Iri(v)) => v.clone(),
                        _ => String::new(),
                    })
                    .collect()
            })
            .collect()),
        Ok(other) => Err(format!("expected solutions, got {other:?}")),
        Err(e) => Err(format!("sparql: {e:?}")),
    }
}

/// Reflexive verdict: goal A:B is true iff sub(A,B) in the closure.
fn eval_goal(goal: &str, closure: &BTreeSet<(String, String)>, _c: &BTreeSet<String>) -> bool {
    let (a, b) = goal.split_once(':').expect("goal A:B");
    closure.contains(&(iri(a), iri(b)))
}

// ---------------------------------------------------------------------------
// Route B: DL -> OWL-RL
// ---------------------------------------------------------------------------

fn route_b(p: &Parsed, goals: &[&str]) -> Result<Vec<bool>, String> {
    let mut g = String::from(
        "@prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix dl: <https://graphlaw.dev/dl#> .\n\n",
    );
    // Witnesses + reflexive typing surface (rdfs10 fires on typed classes).
    for c in &p.concepts {
        let _ = writeln!(g, "<{}> a owl:Class .", iri(c));
        let _ = writeln!(g, "dl:w_{c} a <{}> .", iri(c));
    }
    // subclass -> rdfs:subClassOf
    for (a, b) in &p.subclass {
        let _ = writeln!(g, "<{}> rdfs:subClassOf <{}> .", iri(a), iri(b));
    }
    // conj -> intersectionOf superclass
    for (a1, a2, b) in &p.conj {
        let _ = writeln!(
            g,
            "[] a owl:Class ; owl:intersectionOf ( <{}> <{}> ) ; rdfs:subClassOf <{}> .",
            iri(a1), iri(a2), iri(b)
        );
    }
    // exists_rhs A sub exists r.B -> A subClassOf restriction(r, someValuesFrom B)
    for (a, r, b) in &p.exists_rhs {
        let _ = writeln!(
            g,
            "<{}> rdfs:subClassOf _:res_{a}_{r}_{b} .\n_:res_{a}_{r}_{b} a owl:Restriction ; owl:onProperty <{}> ; owl:someValuesFrom <{}> .",
            iri(a), iri(r), iri(b)
        );
    }
    // exists_lhs exists r.A sub B -> restriction(r, someValuesFrom A) subClassOf B
    for (r, a, b) in &p.exists_lhs {
        let _ = writeln!(
            g,
            "_:res_{r}_{a}_{b} a owl:Restriction ; owl:onProperty <{}> ; owl:someValuesFrom <{}> ; rdfs:subClassOf <{}> .",
            iri(r), iri(a), iri(b)
        );
    }

    let s0 = LawState::parse(g.as_bytes(), Dialect::Turtle, None)
        .map_err(|e| format!("route B parse: {e:?}"))?;
    let (s1, _r) = s0
        .transition(&Step::EntailOwlRl)
        .map_err(|e| format!("route B entail: {e:?}"))?;

    // Verdict surface: witness type closure. w_A rdf:type B  <=>  A sub B.
    let mut types: BTreeSet<(String, String)> = BTreeSet::new();
    for row in select(&s1, "SELECT ?s ?o WHERE { ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?o }")? {
        let (Some(s), Some(o)) = (row.first().cloned(), row.get(1).cloned()) else {
            continue;
        };
        types.insert((s, o));
    }
    Ok(goals.iter().map(|goal| {
        let (a, b) = goal.split_once(':').expect("goal A:B");
        types.contains(&(format!("{}w_{}", DL, a), iri(b)))
    }).collect())
}

// ---------------------------------------------------------------------------
// Vector emission (tests/fixtures/graphlaw schema v1)
// ---------------------------------------------------------------------------

fn emit_vector(case: &Case, _parsed: &Parsed, verdicts: &[bool]) -> Result<String, String> {
    let rev = std::process::Command::new("git")
        .args(["-C", "/Users/sac/graphlaw", "rev-parse", "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;
    let rev = String::from_utf8_lossy(&rev.stdout).trim().to_string();

    let mut facts_json = String::new();
    for (i, (k, v)) in case.facts.iter().enumerate() {
        if i > 0 { facts_json.push_str(",\n        "); }
        facts_json.push_str(&format!("{{ \"key\": \"{k}\", \"value\": \"{v}\" }}"));
    }
    let mut goals_json = String::new();
    let mut verdicts_json = String::new();
    let mut derived = Vec::new();
    for (i, (g, v)) in case.goals.iter().zip(verdicts).enumerate() {
        let (a, b) = g.split_once(':').unwrap();
        let gid = format!("g{}", i + 1);
        if i > 0 { goals_json.push_str(", "); }
        goals_json.push_str(&format!(
            "{{ \"id\": \"{gid}\", \"predicate\": \"dl:subsumes\", \"value\": \"{g}\" }}"
        ));
        if i > 0 { verdicts_json.push_str(", "); }
        verdicts_json.push_str(&format!("\"{gid}\": \"{a}\u{2291}{b}={v}\""));
        derived.push(format!("dl:verdict:{a}:{b}={v}"));
    }
    derived.sort();
    let mut derived_json = String::new();
    for (i, d) in derived.iter().enumerate() {
        if i > 0 { derived_json.push_str(",\n      "); }
        derived_json.push_str(&format!("\"{d}\""));
    }

    Ok(format!(
        r#"{{
  "vector_id": "description_logic-{slug}",
  "breed": "description_logic",
  "provenance": {{
    "graphlaw_rev": "{rev}",
    "purrdf": "2.0.2",
    "eyeron": "0.7.8",
    "generator": "graphlaw src/bin/dl_dualroute.rs (dual-route: Eyeron N3 DATALOG CR1-CR4 + purrdf OWL-RL; agreement required before emission)",
    "stratified": false
  }},
  "breed_input": {{
    "intent": "classify",
    "facts": [
        {facts_json}
    ],
    "rules": [],
    "goals": [{goals_json}],
    "candidates": [],
    "cases": [],
    "state": []
  }},
  "expected": {{
    "derived_facts_sorted": [
      {derived_json}
    ],
    "verdicts": {{ {verdicts_json} }},
    "notes": "{notes} — dual-route agreed (Eyeron DATALOG fixpoint == purrdf OWL-RL closure) on {n} goals before emission."
  }}
}}
"#,
        slug = case.slug,
        facts_json = facts_json,
        goals_json = goals_json,
        derived_json = derived_json,
        verdicts_json = verdicts_json,
        notes = case.notes,
        n = case.goals.len(),
    ))
}

fn main() {
    let out_dir = "/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw";
    let paper = [
        ("Pericarditis:HeartDisease", true),
        ("HeartDisease:Pericarditis", false),
    ];
    let mut failed = false;

    for case in cases() {
        let parsed = match parse(&case.facts) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("FAIL[{}]: parse: {e}", case.slug);
                failed = true;
                continue;
            }
        };
        let a = match route_a(&parsed, &case.goals) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("FAIL[{}]: route A: {e}", case.slug);
                failed = true;
                continue;
            }
        };
        let b = match route_b(&parsed, &case.goals) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("FAIL[{}]: route B: {e}", case.slug);
                failed = true;
                continue;
            }
        };
        let mut agree = true;
        for (i, g) in case.goals.iter().enumerate() {
            if a[i] != b[i] {
                agree = false;
                eprintln!(
                    "SEMANTIC-GAP[{}]: goal {g}: route A (DATALOG) = {}, route B (OWL-RL) = {}",
                    case.slug, a[i], b[i]
                );
            }
        }
        if !agree {
            failed = true;
            continue;
        }
        // Paper-fixture cross-check on case (a).
        if case.slug == "a-paper-baader-el" {
            for (g, expected) in paper {
                let i = case.goals.iter().position(|x| *x == g).unwrap();
                if a[i] != expected {
                    eprintln!("FAIL: paper-fixture cross-check {g}: got {}, want {expected}", a[i]);
                    failed = true;
                }
            }
        }
        let json = match emit_vector(&case, &parsed, &a) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("FAIL[{}]: emit: {e}", case.slug);
                failed = true;
                continue;
            }
        };
        let path = format!("{out_dir}/description_logic-{}.json", case.slug);
        std::fs::write(&path, json).expect("write vector");
        println!("OK[{}]: dual-route agreement, vector -> {path}", case.slug);
    }

    if failed {
        std::process::exit(1);
    }
}
