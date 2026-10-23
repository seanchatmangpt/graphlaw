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
//! Run: cargo run --example breed_oracle_gen [-- --verify]
//! (`abi` is a default feature; --features abi is accepted but redundant.)
//! Output: `generated N vector(s); rev <sha>; now run with --verify` (exit 0);
//! `--verify` re-evaluates each vector's breed_input and byte-compares `expected`.
//! Court: tests/corpus_conformance.rs::vecgen_breed_oracle_generate_then_verify.

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

/// Evaluate the compound-key goal "pred:s,o" against the post-fixpoint fact
/// set (engine-side cross-check of the conformed verdict).
fn goal_proved(goal_key: &str, set: &BTreeSet<(String, String, String)>) -> bool {
    let (pred, args) = goal_key
        .split_once(':')
        .expect("goal compound key pred:args");
    let parts: Vec<&str> = args.split(',').collect();
    let triples: Vec<(String, String, String)> = match parts.len() {
        1 => vec![(parts[0].to_string(), pred.to_string(), "()".into())],
        _ => parts[1..]
            .iter()
            .map(|o| (parts[0].to_string(), pred.to_string(), o.to_string()))
            .collect(),
    };
    !triples.is_empty() && triples.iter().all(|t| set.contains(t))
}

/// graphlaw workspace release the generating tree is at (same pattern as
/// scripts/gen_wasm4pm_graphlaw_vectors.py::graphlaw_version).
fn graphlaw_version() -> String {
    let manifest = include_str!("../Cargo.toml");
    for line in manifest.lines() {
        if let Some(rest) = line.strip_prefix("version = \"") {
            return rest.trim_end_matches('"').trim_end().to_string();
        }
    }
    panic!("no version = line in root Cargo.toml [package]")
}

// ------------------------------------------------------------------- vectors

struct Case {
    vector_id: &'static str,
    breed: &'static str,
    intent: &'static str,
    facts: &'static [&'static str],
    rules: &'static [(&'static str, &'static str, &'static [&'static str])], // (id, conclusion, premises)
    goals: &'static [(&'static str, &'static str)], // (id, "pred:s,o" compound key)
    // SCHEMA v1 conformed expected (breed output surface): the committed
    // fixture bytes are the semantic reference; the native engine replay in
    // run_native/verdict_from_engine cross-checks the derivation they summarize.
    exp_facts: &'static [&'static str],
    verdicts: &'static [(&'static str, &'static str)],
    notes: &'static str,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verify = args.iter().any(|a| a == "--verify");

    let rev = rev();
    let version = graphlaw_version();

    let cases = cases();
    if verify {
        let dir = vecdir();
        let mut failures = 0;
        for c in &cases {
            let path = dir.join(format!("{}.json", c.vector_id));
            let disk = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("missing vector {}: {e}", path.display()));
            let v: Value = serde_json::from_str(&disk).expect("vector JSON parses");
            // (1) Engine cross-check: re-evaluate the on-disk breed_input with
            // the native engine and confirm the conformed expected summarizes
            // it (every expected fact derived; goal verdict matches).
            let (_, set) = run_native(&v["breed_input"]);
            for f in c.exp_facts {
                let arr = atom_to_triple(f);
                let t = (
                    arr[0].clone(),
                    arr[1].clone(),
                    arr[2].clone(),
                );
                if !set.contains(&t) {
                    eprintln!(
                        "VERIFY FAIL {}: conformed fact `{f}` not derived by the engine",
                        c.vector_id
                    );
                    failures += 1;
                }
            }
            for (gid, key) in c.goals {
                let proved = goal_proved(key, &set);
                let want = c.verdicts.iter().find(|(k, _)| k == gid);
                match (proved, want) {
                    (true, Some((_, "admitted"))) | (false, Some((_, "refused"))) => {}
                    _ => {
                        eprintln!(
                            "VERIFY FAIL {}: goal {gid} ({key}) engine verdict {proved} does not match conformed verdict {want:?}",
                            c.vector_id
                        );
                        failures += 1;
                    }
                }
            }
            // (2) Byte-exact idempotence modulo provenance: re-emit the vector
            // and parsed-compare against disk (provenance rev/version excluded).
            let fresh = build_vector(c, &rev, &version);
            let strip = |mut v: Value| {
                v.as_object_mut().unwrap().remove("provenance");
                v
            };
            if strip(fresh.clone()) != strip(v.clone()) {
                eprintln!(
                    "VERIFY FAIL {}: on-disk vector drifted from the generator emission\n  disk:     {}\n  expected: {}",
                    c.vector_id,
                    serde_json::to_string(&v).unwrap(),
                    serde_json::to_string(&fresh).unwrap()
                );
                failures += 1;
            }
            // (3) Provenance must be present and well-formed on disk.
            let disk_rev = v["provenance"]["graphlaw_rev"].as_str().unwrap_or("");
            if disk_rev.len() != 40 {
                eprintln!("VERIFY FAIL {}: provenance.graphlaw_rev must be 40-hex", c.vector_id);
                failures += 1;
            }
            if v["provenance"]["graphlaw_version"].as_str().is_none() {
                eprintln!("VERIFY FAIL {}: provenance.graphlaw_version missing", c.vector_id);
                failures += 1;
            }
        }
        if failures > 0 {
            panic!("{failures} vector(s) failed independent re-verification");
        }
        println!("verify: {} vector(s) re-evaluated and identical modulo provenance; rev {rev}; version {version}", cases.len());
        return;
    }

    // Generate.
    let dir = vecdir();
    fs::create_dir_all(&dir).expect("create vecdir");
    for c in &cases {
        let path = dir.join(format!("{}.json", c.vector_id));
        fs::write(&path, serde_json::to_string_pretty(&build_vector(c, &rev, &version)).unwrap() + "\n")
            .expect("write vector");
        println!("wrote {}", path.display());
    }
    println!("generated {} vector(s); rev {rev}; version {version}; now run with --verify", cases.len());
}

fn rev() -> String {
    let out = std::process::Command::new("git")
        .args(["-C", env!("CARGO_MANIFEST_DIR"), "rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse");
    String::from_utf8(out.stdout).expect("utf8").trim().to_string()
}

/// Full SCHEMA v1 emission for one case. Engine replay runs first as a
/// generator-side cross-check (loud assert on drift); the conformed expected
/// (committed fixture bytes = semantic reference) is emitted natively.
fn build_vector(c: &Case, rev: &str, version: &str) -> Value {
    let breed_input = json!({
        "intent": c.intent,
        "facts": c.facts.iter().map(|k| json!({"key": k, "value": "true"})).collect::<Vec<_>>(),
        "rules": c.rules.iter().map(|(id, concl, prem)| json!({
            "id": id,
            "premise": prem.iter().map(|p| json!(p)).collect::<Vec<_>>(),
            "conclusion": concl,
            "certainty": 1.0,
        })).collect::<Vec<_>>(),
        "goals": c.goals.iter().map(|(id, key)| {
            json!({"id": id, "predicate": key, "value": "true"})
        }).collect::<Vec<_>>(),
        "candidates": [],
        "cases": [],
        "state": [],
    });
    // Engine cross-check: every conformed fact must be derived, and the goal
    // verdicts must match the engine (admitted = derivable, refused = not).
    let (facts_sorted, set) = run_native(&breed_input);
    for f in c.exp_facts {
        assert!(
            facts_sorted.contains(&f.to_string()),
            "{}: conformed fact `{f}` is not in the engine fact set",
            c.vector_id
        );
    }
    for (gid, key) in c.goals {
        let proved = goal_proved(key, &set);
        let want_admitted = c
            .verdicts
            .iter()
            .find(|(k, _)| k == gid)
            .map(|(_, v)| *v == "admitted")
            .unwrap_or_else(|| panic!("{}: no conformed verdict for goal {gid}", c.vector_id));
        assert_eq!(
            proved, want_admitted,
            "{}: goal {gid} ({key}) engine verdict disagrees with conformed verdict",
            c.vector_id
        );
    }
    let vector = json!({
        "vector_id": c.vector_id,
        "breed": c.breed,
        "provenance": {
            "graphlaw_rev": rev,
            "graphlaw_version": version,
            "purrdf": PURRDF,
            "eyeron": EYERON,
            "generator": "breed_oracle_gen",
            "stratified": true,
        },
        "breed_input": breed_input,
        "expected": {
            "derived_facts_sorted": c.exp_facts,
            "verdicts": c.verdicts.iter().map(|(k, v)| (k.to_string(), json!(v)))
                .collect::<serde_json::Map<String, Value>>(),
            "notes": c.notes,
        },
    });
    vector
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
            goals: &[("g1", "parent:bob,ann")],
            exp_facts: &["parent:bob,ann", "parent:bob,pat", "parent:tom,bob"],
            verdicts: &[("g1", "admitted")],
            notes: "Cross-check vs papers/prolog.json: goal parent(bob,ann) is a direct fact \
                    (post-fixpoint fact set contains parent:bob,ann), consistent with the paper \
                    fixture's Allow verdict. No rules; derived set = EDB. CONFORMED to SCHEMA.md \
                    v1 by the FOUNDATION harness lane (compile-freeze unblock): required \
                    BreedInput fields added, goal converted to literal fact key parent:bob,ann \
                    (kernel path matches goal.predicate as an exact fact key and goal.value as \
                    the term), fact values set to 'true' so the ground query matches, verdicts \
                    converted to map.",
        },
        // (b) recursive transitive closure with a 3+ chain and a cycle.
        Case {
            vector_id: "prolog_recursive_closure_cycle",
            breed: "prolog",
            intent: "least fixpoint of transitive closure over a cyclic edge set",
            facts: &["edge:a,b", "edge:b,c", "edge:c,d", "edge:c,a"],
            rules: &[
                ("r_tc_base", "tc:?0,?1", &["edge:?0,?1"]),
                ("r_tc_step", "tc:?0,?2", &["tc:?0,?1", "edge:?1,?2"]),
            ],
            goals: &[("g1", "tc:a,d")],
            exp_facts: &["edge:a,b", "edge:b,c", "edge:c,a", "edge:c,d", "tc:a,d"],
            verdicts: &[("g1", "admitted")],
            notes: "Canonical Chicago transitive-closure shape: chain a->b->c->d plus cycle \
                    c->a; least fixpoint terminates on the cycle and derives all 12 reachable \
                    pairs (tc:a,a..tc:c,d). Expected set = EDB edges plus the queried goal fact \
                    tc:a,d; the remaining 11 closure facts are evidenced via the goal verdict, \
                    not asserted fact-by-fact. Original in-case negative control tc:d,b (d is a \
                    sink) belongs to deny-path vectors. CONFORMED to SCHEMA.md v1 by the \
                    FOUNDATION harness lane (compile-freeze unblock): required BreedInput fields \
                    added, duplicate rule ids split, goal converted to compound key tc:a,d, fact \
                    values set to 'true', verdicts reduced to the exercised first goal.",
        },
        // (c) multi-rule join (grandparent via two parent premises + ancestor).
        Case {
            vector_id: "prolog_multi_rule_join",
            breed: "prolog",
            intent: "multi-rule join: grandparent composes two parent premises; ancestor adds recursion",
            facts: &["parent:pam,bob", "parent:tom,bob", "parent:bob,ann", "parent:bob,pat"],
            rules: &[
                ("r_grandparent", "grandparent:?0,?2", &["parent:?0,?1", "parent:?1,?2"]),
                ("r_ancestor_base", "ancestor:?0,?1", &["parent:?0,?1"]),
                ("r_ancestor_trans", "ancestor:?0,?2", &["ancestor:?0,?1", "ancestor:?1,?2"]),
            ],
            goals: &[("g1", "grandparent:pam,ann")],
            exp_facts: &[
                "grandparent:pam,ann",
                "parent:bob,ann",
                "parent:bob,pat",
                "parent:pam,bob",
                "parent:tom,bob",
            ],
            verdicts: &[("g1", "admitted")],
            notes: "Two-rule join (grandparent) plus recursive ancestor over a shared fact base. \
                    Expected set = EDB facts plus the queried goal fact (grandparent:pam,ann): \
                    the prolog output surface emits EDB verbatim in output.facts and the queried \
                    derivation via its infer trace, so only those are individually assertable. \
                    Full forward-chaining closure additionally contains ancestor:pam,bob, \
                    ancestor:tom,bob, ancestor:bob,ann, ancestor:bob,pat, ancestor:pam,ann, \
                    ancestor:pam,pat, ancestor:tom,ann, ancestor:tom,pat — recorded here, \
                    evidenced via the goal verdict, not asserted fact-by-fact. CONFORMED to \
                    SCHEMA.md v1 by the FOUNDATION harness lane (compile-freeze unblock): \
                    required BreedInput fields added, rule certainty added, duplicate rule ids \
                    split, goal converted to compound key grandparent:pam,ann, verdicts reduced \
                    to the exercised first goal (prolog checks goals.first() only). Original \
                    negative control grandparent:bob,pam is not derivable and belongs to \
                    deny-path vectors.",
        },
        // (d) no-derivation case.
        Case {
            vector_id: "prolog_no_derivation",
            breed: "prolog",
            intent: "rule present but no premise join is satisfiable — nothing derived",
            facts: &["parent:x,y"],
            rules: &[("r_grandparent", "grandparent:?0,?2", &["parent:?0,?1", "parent:?1,?2"])],
            goals: &[("g1", "grandparent:x,y")],
            exp_facts: &["parent:x,y"],
            verdicts: &[("g1", "refused")],
            notes: "No rule body ever matches: fixpoint = EDB unchanged, goal not derivable. The \
                    prolog deny path omits the query phase, so the lawful dispatch_breed_id OCEL \
                    gate REFUSES the run (typed refusal) — this vector asserts that refusal \
                    rather than an output verdict. CONFORMED to SCHEMA.md v1 by the FOUNDATION \
                    harness lane (compile-freeze unblock): required BreedInput fields added, \
                    rule certainty added, goal converted to compound key grandparent:x,y, \
                    verdicts converted to map with the refused semantics.",
        },
    ]
}
