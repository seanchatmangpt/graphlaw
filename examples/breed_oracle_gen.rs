//! breed_oracle_gen — vector GENERATOR for the PROLOG breed from the native
//! graphlaw toolchain (JSON ABI `datalog` op → seminaive compile/evaluate over
//! RelationStore).
//!
//! Translation (PROLOG → Datalog):
//!   fact/key or rule string "pred:arg1,...,argN"   (BreedInput predicate-encoding)
//!     → Datalog atom pred(arg1, ..., argN) → triple [arg1, "pred", argN] (arity 2),
//!       [arg, "pred", "()"] (arity 1), ["pred", "-", "-"] (arity 0).
//!   rule premises/conclusion use ?N positional variables, translated verbatim
//!     (graphlaw clause terms use the same `?`-prefix variable syntax).
//!
//! Modes:
//!   (no args)        generate vectors into
//!                    $GL_VECDIR (default: wasm4pm cognition fixtures/graphlaw/)
//!   --verify         re-evaluate each emitted vector's `breed_input` with the
//!                    native engine and byte-compare against `expected`
//!                    (idempotence / second independent evaluation run).
//!
//! Run: cargo run --example breed_oracle_gen --features abi [-- --verify]
//! (from /Users/sac/graphlaw)

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use graphlaw::abi::call_json;
use serde_json::{Value, json};

const PURRDF: &str = "2.0.2";
const EYERON: &str = "0.7.8";

// ---------------------------------------------------------------- translation

/// "pred:a,b" → triple ["a", "pred", "b"]; arity-1 → ["a","pred","()"];
/// arity-0 → ["pred","-","-"].
fn atom_to_triple(key: &str) -> Vec<String> {
    let (pred, args) = match key.split_once(':') {
        Some((p, rest)) => (p, rest.split(',').collect::<Vec<_>>()),
        None => (key, vec![]),
    };
    match args.len() {
        0 => vec![pred.to_string(), "-".into(), "-".into()],
        1 => vec![args[0].to_string(), pred.to_string(), "()".into()],
        2 => vec![args[0].to_string(), pred.to_string(), args[1].to_string()],
        _ => panic!("breed_oracle_gen: arity > 2 not supported: {key}"),
    }
}

/// Build the graphlaw ABI datalog request for a breed input.
fn datalog_request(breed_input: &Value) -> Value {
    let facts: Vec<Value> = breed_input["facts"]
        .as_array()
        .expect("breed facts array")
        .iter()
        .map(|f| {
            let arr = atom_to_triple(f["key"].as_str().expect("fact key"));
            json!(arr)
        })
        .collect();
    let rules: Vec<Value> = breed_input["rules"]
        .as_array()
        .expect("breed rules array")
        .iter()
        .map(|r| {
            json!({
                "head": atom_to_triple(r["conclusion"].as_str().expect("rule conclusion")),
                "body": r["premise"]
                    .as_array()
                    .expect("rule premise array")
                    .iter()
                    .map(|p| json!(atom_to_triple(p.as_str().expect("premise string"))))
                    .collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({"op": "datalog", "rules": rules, "facts": facts})
}

/// Run the native engine over a breed input; returns (facts_sorted as
/// "pred:s,o" strings, BTreeSet of fact tuples for verdict checks).
fn run_native(breed_input: &Value) -> (Vec<String>, BTreeSet<(String, String, String)>) {
    let out = call_json(&datalog_request(breed_input));
    assert_eq!(out["ok"], true, "graphlaw datalog refused: {out}");
    let mut sorted = Vec::new();
    let mut set = BTreeSet::new();
    for f in out["facts"].as_array().expect("facts array") {
        let arr = f.as_array().expect("triple");
        let (s, p, o) = (
            arr[0].as_str().expect("s"),
            arr[1].as_str().expect("p"),
            arr[2].as_str().expect("o"),
        );
        sorted.push(triple_to_key(s, p, o));
        set.insert((s.to_string(), p.to_string(), o.to_string()));
    }
    sorted.sort();
    sorted.dedup();
    (sorted, set)
}

fn triple_to_key(s: &str, p: &str, o: &str) -> String {
    match o {
        "-" => p.to_string(),
        "()" => format!("{p}:{s}"),
        _ => format!("{p}:{s},{o}"),
    }
}

/// Evaluate goals against the post-fixpoint fact set.
fn verdicts(breed_input: &Value, set: &BTreeSet<(String, String, String)>) -> Vec<Value> {
    breed_input["goals"]
        .as_array()
        .expect("goals array")
        .iter()
        .map(|g| {
            // goal value is dash- or comma-joined args; predicate is the pred name.
            let pred = g["predicate"].as_str().unwrap_or_default();
            let raw = g["value"].as_str().unwrap_or_default();
            let args: Vec<&str> = if raw.contains(',') {
                raw.split(',').collect()
            } else if let Some((a, b)) = raw.split_once('-') {
                vec![a, b]
            } else if raw.is_empty() {
                vec![]
            } else {
                vec![raw]
            };
            let key = format!("{}:{}", pred, args.join(","));
            let triples: Vec<(String, String, String)> = match args.len() {
                0 => vec![(pred.to_string(), "-".into(), "-".into())],
                1 => vec![(args[0].to_string(), pred.to_string(), "()".into())],
                _ => args[1..]
                    .iter()
                    .map(|o| (args[0].to_string(), pred.to_string(), o.to_string()))
                    .collect(),
            };
            let proved = !triples.is_empty() && triples.iter().all(|t| set.contains(t));
            json!({"goal": key, "verdict": if proved { "proved" } else { "not_proved" }})
        })
        .collect()
}

// ------------------------------------------------------------------- vectors

struct Case {
    vector_id: &'static str,
    breed: &'static str,
    intent: &'static str,
    facts: &'static [&'static str],
    rules: &'static [(&'static str, &'static [&'static str])], // (conclusion, premises)
    goals: &'static [(&'static str, &'static str)],            // (id, "pred" + value)
    notes: &'static str,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verify = args.iter().any(|a| a == "--verify");

    let rev = {
        let out = std::process::Command::new("git")
            .args(["-C", env!("CARGO_MANIFEST_DIR"), "rev-parse", "HEAD"])
            .output()
            .expect("git rev-parse");
        String::from_utf8(out.stdout).expect("utf8").trim().to_string()
    };

    let cases = cases();
    if verify {
        let dir = vecdir();
        let mut failures = 0;
        for c in &cases {
            let path = dir.join(format!("{}.json", c.vector_id));
            let disk = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("missing vector {}: {e}", path.display()));
            let v: Value = serde_json::from_str(&disk).expect("vector JSON parses");
            let (facts, set) = run_native(&v["breed_input"]);
            let got_verdicts = verdicts(&v["breed_input"], &set);
            let exp_facts: &Vec<Value> = v["expected"]["derived_facts_sorted"]
                .as_array()
                .expect("derived_facts_sorted array");
            let exp_facts: Vec<String> = exp_facts
                .iter()
                .map(|f| f.as_str().expect("fact string").to_string())
            .collect();
            if facts != exp_facts {
                eprintln!(
                    "VERIFY FAIL {}: derived facts differ\n  expected: {:?}\n  got:      {:?}",
                    c.vector_id, exp_facts, facts
                );
                failures += 1;
                continue;
            }
            if got_verdicts != v["expected"]["verdicts"].as_array().expect("verdicts").as_slice() {
                eprintln!("VERIFY FAIL {}: verdicts differ\n  expected: {}\n  got:      {}",
                    c.vector_id, v["expected"]["verdicts"], serde_json::to_string(&got_verdicts).unwrap());
                failures += 1;
                continue;
            }
            // provenance rev must match the running tree's rev
            let disk_rev = v["provenance"]["graphlaw_rev"].as_str().expect("rev");
            if disk_rev != rev {
                eprintln!("VERIFY FAIL {}: provenance rev {disk_rev} != current {rev}", c.vector_id);
                failures += 1;
            }
        }
        if failures > 0 {
            panic!("{failures} vector(s) failed independent re-verification");
        }
        println!("verify: {} vector(s) re-evaluated and byte-identical; rev {rev}", cases.len());
        return;
    }

    // Generate.
    let dir = vecdir();
    fs::create_dir_all(&dir).expect("create vecdir");
    for c in &cases {
        let breed_input = json!({
            "intent": c.intent,
            "facts": c.facts.iter().map(|k| json!({"key": k, "value": ""})).collect::<Vec<_>>(),
            "rules": c.rules.iter().map(|(concl, prem)| json!({
                "id": format!("r_{}", concl.split(':').next().unwrap_or("r")),
                "premise": prem.iter().map(|p| json!(p)).collect::<Vec<_>>(),
                "conclusion": concl,
            })).collect::<Vec<_>>(),
            "goals": c.goals.iter().map(|(id, val)| {
                // val is "pred|value" — predicate name and joined args
                let (pred, value) = val.split_once('|').expect("goal pred|value");
                json!({"id": id, "predicate": pred, "value": value})
            }).collect::<Vec<_>>(),
        });
        let (facts_sorted, set) = run_native(&breed_input);
        let verd = verdicts(&breed_input, &set);
        let vector = json!({
            "vector_id": c.vector_id,
            "breed": c.breed,
            "provenance": {
                "graphlaw_rev": rev,
                "purrdf": PURRDF,
                "eyeron": EYERON,
                "generator": "breed_oracle_gen",
                "stratified": true,
            },
            "breed_input": breed_input,
            "expected": {
                "derived_facts_sorted": facts_sorted,
                "verdicts": verd,
                "notes": c.notes,
            },
        });
        let path = dir.join(format!("{}.json", c.vector_id));
        fs::write(&path, serde_json::to_string_pretty(&vector).unwrap() + "\n")
            .expect("write vector");
        println!("wrote {}", path.display());
    }
    println!("generated {} vector(s); rev {rev}; now run with --verify", cases.len());
}

fn vecdir() -> PathBuf {
    std::env::var_os("GL_VECDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw")
        })
}

fn cases() -> Vec<Case> {
    vec![
        // (a) paper fixture input (Kowalski 1974 Fig. 2), facts only.
        Case {
            vector_id: "prolog_paper_fixture",
            breed: "prolog",
            intent: "prove parent(bob, ann) from the family fact base (Kowalski 1974)",
            facts: &["parent:tom,bob", "parent:bob,ann", "parent:bob,pat"],
            rules: &[],
            goals: &[("g1", "parent|bob-ann")],
            notes: "Cross-check vs papers/prolog.json: goal parent(bob,ann) is a direct fact \
                    (post-fixpoint fact set contains parent:bob,ann), consistent with the paper \
                    fixture's Allow verdict and resolved binding 'ann'. No rules; derived set = EDB.",
        },
        // (b) recursive transitive closure with a 3+ chain and a cycle.
        Case {
            vector_id: "prolog_recursive_closure_cycle",
            breed: "prolog",
            intent: "least fixpoint of transitive closure over a cyclic edge set",
            facts: &["edge:a,b", "edge:b,c", "edge:c,d", "edge:c,a"],
            rules: &[
                ("tc:?0,?1", &["edge:?0,?1"]),
                ("tc:?0,?2", &["tc:?0,?1", "edge:?1,?2"]),
            ],
            goals: &[
                ("g1", "tc|a-d"),
                ("g2", "tc|d-b"),
                ("g3", "tc|b-a"),
            ],
            notes: "Canonical Chicago transitive-closure shape: 3+ chain a→b→c→d plus cycle \
                    c→a; least fixpoint terminates on the cycle and derives all 12 reachable \
                    pairs (a,b,c each reach 4 nodes; d is a sink, so g2 tc(d,b) is the \
                    in-case negative control).",
        },
        // (c) multi-rule join (grandparent via two parent premises + ancestor).
        Case {
            vector_id: "prolog_multi_rule_join",
            breed: "prolog",
            intent: "multi-rule join: grandparent composes two parent premises; ancestor adds recursion",
            facts: &["parent:pam,bob", "parent:tom,bob", "parent:bob,ann", "parent:bob,pat"],
            rules: &[
                ("grandparent:?0,?2", &["parent:?0,?1", "parent:?1,?2"]),
                ("ancestor:?0,?1", &["parent:?0,?1"]),
                ("ancestor:?0,?2", &["ancestor:?0,?1", "ancestor:?1,?2"]),
            ],
            goals: &[
                ("g1", "grandparent|pam-ann"),
                ("g2", "ancestor|pam-ann"),
                ("g3", "grandparent|bob-pam"),
            ],
            notes: "Two-rule join (grandparent) plus recursive ancestor over a shared fact base; \
                    g3 is an in-case negative control (not derivable).",
        },
        // (d) no-derivation case.
        Case {
            vector_id: "prolog_no_derivation",
            breed: "prolog",
            intent: "rule present but no premise join is satisfiable — nothing derived",
            facts: &["parent:x,y"],
            rules: &[("grandparent:?0,?2", &["parent:?0,?1", "parent:?1,?2"])],
            goals: &[("g1", "grandparent|x-y")],
            notes: "No rule body ever matches: fixpoint = EDB unchanged, goal not_proved.",
        },
    ]
}
