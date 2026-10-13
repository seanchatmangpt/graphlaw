//! Naive-physics cross-engine fixture generator (wasm4pm `naive_physics`
//! breed -> GraphLaw Datalog fixpoint).
//!
//! Translates the breed's np:* scene facts into a positive-Datalog program
//! (support transitivity, falls/spill consequence rules, removal as an event
//! fact), runs the least fixpoint through GraphLaw's admitted Datalog
//! authority (PurRDF via the JSON ABI `datalog` op), and emits conformance
//! vectors to /Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw/.
//!
//! Run from /Users/sac/graphlaw:
//!   cargo run --example naive_physics_vectors --features abi --release
//! Idempotent: re-running rewrites the same bytes.

#![cfg(feature = "abi")]

use graphlaw::abi::call_json;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const OUT_DIR: &str = "/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw";
const NS: &str = "https://np.graphlaw.dev/";

/// np:* fact contract (mirrors the breed's parse_scene):
/// np:on:a = b, np:in:a = c, np:liquid:l = c, np:ground:x = true, np:remove:x = true.
struct Scene {
    id: &'static str,
    facts: Vec<(&'static str, &'static str)>,
    /// Objects that must NOT be derived as falling (stability claim).
    not_falls: &'static [&'static str],
    note: &'static str,
}

fn scenes() -> Vec<Scene> {
    vec![
        // (a) Hayes 1979/1985 cup-of-water paper fixture.
        Scene {
            id: "naive-physics-paper",
            facts: vec![
                ("np:ground:floor", "true"),
                ("np:on:table", "floor"),
                ("np:on:cup", "table"),
                ("np:liquid:water", "cup"),
                ("np:remove:table", "true"),
            ],
            not_falls: &["floor", "table"],
            note: "Hayes cup-of-water: removing the table drops the cup; water spills; floor and table never fall.",
        },
        // (b) Support-chain cascade: removing the base drops everything above.
        Scene {
            id: "naive-physics-support-chain",
            facts: vec![
                ("np:ground:floor", "true"),
                ("np:on:b2", "floor"),
                ("np:on:b1", "b2"),
                ("np:on:b0", "b1"),
                ("np:remove:b2", "true"),
            ],
            not_falls: &["b2", "floor"],
            note: "Removing base block b2 cascades: b1 then b0 fall through the ax-unsupported-falls fixpoint.",
        },
        // (c) Spill case: liquid in a falling container.
        Scene {
            id: "naive-physics-spill",
            facts: vec![
                ("np:ground:floor", "true"),
                ("np:on:table", "floor"),
                ("np:on:jug", "table"),
                ("np:in:ink", "jug"),
                ("np:liquid:ink", "jug"),
                ("np:remove:table", "true"),
            ],
            not_falls: &["floor", "table"],
            note: "Removing the table drops the jug; the contained liquid spills (ax-containment-transport + ax-liquid-spill).",
        },
        // (d) No-fall case: every chain bottoms out at ground, no removal.
        Scene {
            id: "naive-physics-no-fall",
            facts: vec![
                ("np:ground:floor", "true"),
                ("np:on:table", "floor"),
                ("np:on:cup", "table"),
                ("np:in:water", "cup"),
                ("np:liquid:water", "cup"),
            ],
            not_falls: &["floor", "table"],
            note: "Fully supported scene: no falls and no spills are derived (negative case).",
        },
    ]
}

fn main() {
    // GL_VECDIR overrides the output dir (same convention as the other vector
    // generators); the wasm4pm fixture path remains the default.
    let out_dir = std::env::var_os("GL_VECDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(OUT_DIR));
    let shown = out_dir.display();
    fs::create_dir_all(&out_dir).expect("create fixture dir");
    for s in scenes() {
        let vector = build_vector(&s);
        let path = out_dir.join(format!("{}.json", s.id));
        fs::write(&path, vector).expect("write vector");
        println!("wrote {}", path.display());
    }
    println!("wrote {shown}/SCHEMA.md");
    fs::write(out_dir.join("SCHEMA.md"), schema_doc()).expect("write SCHEMA.md");
    println!("wrote {shown}/ENCODERS.md");
    fs::write(out_dir.join("ENCODERS.md"), encoders_doc()).expect("write ENCODERS.md");
}

fn iri(name: &str) -> String {
    format!("{NS}{name}")
}

fn iri_json(name: &str) -> Value {
    json!(iri(name))
}

fn true_obj() -> Value {
    iri_json("true")
}

/// Scene facts -> Datalog EDB triples under the np: vocabulary.
fn edb_facts(scene: &Scene) -> Vec<Value> {
    let mut facts = Vec::new();
    for (key, value) in &scene.facts {
        if let Some(a) = key.strip_prefix("np:on:") {
            facts.push(json!([iri_json(a), iri_json("on"), iri_json(value)]));
        } else if let Some(a) = key.strip_prefix("np:in:") {
            facts.push(json!([iri_json(a), iri_json("in"), iri_json(value)]));
        } else if let Some(l) = key.strip_prefix("np:liquid:") {
            facts.push(json!([iri_json(l), iri_json("liquid"), iri_json(value)]));
        } else if let Some(x) = key.strip_prefix("np:ground:") {
            facts.push(json!([iri_json(x), iri_json("ground"), true_obj()]));
        } else if let Some(x) = key.strip_prefix("np:remove:") {
            facts.push(json!([iri_json(x), iri_json("remove"), true_obj()]));
        } else {
            panic!("unrecognized fact key {key}");
        }
    }
    facts.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    facts
}

/// The four-axiom program as positive Horn rules (all over np: predicates).
fn axioms() -> Vec<Value> {
    let p = |n: &str| iri_json(n);
    vec![
        // ax-support, base: on/in are support.
        json!({"head": ["?x", p("sup"), "?y"], "body": [["?x", p("on"), "?y"]]}),
        json!({"head": ["?x", p("sup"), "?y"], "body": [["?x", p("in"), "?y"]]}),
        // ax-support, transitivity.
        json!({"head": ["?x", p("sup"), "?z"],
               "body": [["?x", p("sup"), "?y"], ["?y", p("sup"), "?z"]]}),
        // ax-unsupported-falls (seed: direct support removed).
        json!({"head": ["?x", p("falls"), "?y"],
               "body": [["?x", p("sup"), "?y"], ["?y", p("remove"), "?w"]]}),
        // falls fixpoint (cascade + ax-containment-transport).
        json!({"head": ["?x", p("falls"), "?y"],
               "body": [["?x", p("sup"), "?y"], ["?y", p("falls"), "?w"]]}),
        // ax-liquid-spill: container fell or was removed.
        json!({"head": ["?l", p("spills"), "?c"],
               "body": [["?l", p("liquid"), "?c"], ["?c", p("falls"), "?w"]]}),
        json!({"head": ["?l", p("spills"), "?c"],
               "body": [["?l", p("liquid"), "?c"], ["?c", p("remove"), "?w"]]}),
    ]
}

/// Run the program through GraphLaw's Datalog authority; return falls/spills sets.
fn run_datalog(scene: &Scene) -> (BTreeSet<String>, BTreeSet<String>) {
    let req = json!({"op": "datalog", "rules": axioms(), "facts": edb_facts(scene)});
    let out = call_json(&req);
    assert_eq!(out["ok"], json!(true), "datalog refused: {out}");
    let mut falls = BTreeSet::new();
    let mut spills = BTreeSet::new();
    for fact in out["facts"].as_array().expect("facts array") {
        let f = fact.as_array().expect("fact triple");
        let pred = f[1].as_str().unwrap();
        let subj = f[0].as_str().unwrap().strip_prefix(NS).unwrap();
        if pred == iri("falls") {
            falls.insert(subj.to_string());
        } else if pred == iri("spills") {
            spills.insert(subj.to_string());
        }
    }
    (falls, spills)
}

/// Independent least-fixpoint reference (breed semantics) for cross-checking.
fn reference_fixpoint(scene: &Scene) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut sup: Vec<(String, String)> = Vec::new();
    let mut liquids: Vec<(String, String)> = Vec::new();
    let mut grounds = BTreeSet::new();
    let mut removed = BTreeSet::new();
    let mut objects = BTreeSet::new();
    for (key, value) in &scene.facts {
        if let Some(a) = key.strip_prefix("np:on:") {
            sup.push((a.to_string(), value.to_string()));
            objects.insert(a.to_string());
            objects.insert(value.to_string());
        } else if let Some(a) = key.strip_prefix("np:in:") {
            sup.push((a.to_string(), value.to_string()));
            objects.insert(a.to_string());
            objects.insert(value.to_string());
        } else if let Some(l) = key.strip_prefix("np:liquid:") {
            liquids.push((l.to_string(), value.to_string()));
            objects.insert(value.to_string());
        } else if let Some(x) = key.strip_prefix("np:ground:") {
            grounds.insert(x.to_string());
            objects.insert(x.to_string());
        } else if let Some(x) = key.strip_prefix("np:remove:") {
            removed.insert(x.to_string());
        }
    }
    let mut falls = BTreeSet::new();
    loop {
        let mut changed = false;
        for (o, s) in &sup {
            if removed.contains(o) || grounds.contains(o) {
                continue;
            }
            if (removed.contains(s) || falls.contains(s)) && falls.insert(o.clone()) {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut spills = BTreeSet::new();
    for (l, c) in &liquids {
        if falls.contains(c) || removed.contains(c) {
            spills.insert(l.clone());
        }
    }
    let _ = objects;
    (falls, spills)
}

fn check(scene: &Scene) -> Value {
    let (falls, spills) = run_datalog(scene);
    let (ref_falls, ref_spills) = reference_fixpoint(scene);
    assert_eq!(falls, ref_falls, "{}: datalog vs reference falls", scene.id);
    assert_eq!(spills, ref_spills, "{}: datalog vs reference spills", scene.id);
    for o in scene.not_falls {
        assert!(!falls.contains(*o), "{}: {} must not fall", scene.id, o);
    }
    let not_falls: Vec<String> = scene.not_falls.iter().map(|s| s.to_string()).collect();
    let value = "verified";
    json!({
        "vector": scene.id,
        "breed": "naive_physics",
        "engine": "graphlaw::abi op=datalog (PurRDF least fixpoint)",
        "provenance": {
            "paper": "Hayes 1979 (Naive Physics Manifesto); Hayes 1985 (Naive physics I: ontology for liquids)",
            "axioms": [
                "ax-support (on/in are support; support transitivity)",
                "ax-unsupported-falls (support removed -> falls)",
                "ax-containment-transport (contents of a falling container fall)",
                "ax-liquid-spill (liquid in a falling/removed container spills)"
            ],
            "note": scene.note
        },
        "input": {"facts": scene.facts.iter().map(|(k, v)| json!({"key": k, "value": v})).collect::<Vec<_>>()},
        "datalog": {"rules": axioms(), "facts": edb_facts(scene)},
        "expected": {
            "falls": falls.iter().cloned().collect::<Vec<_>>(),
            "spills": spills.iter().cloned().collect::<Vec<_>>(),
            "not_falls": not_falls,
            "value": value
        }
    })
}

fn build_vector(scene: &Scene) -> String {
    let v = check(scene);
    serde_json::to_string_pretty(&v).expect("serialize vector") + "\n"
}

fn schema_doc() -> String {
    r#"# GraphLaw cross-engine fixture vectors — naive_physics

Generated (do not hand-edit) by `/Users/sac/graphlaw/examples/naive_physics_vectors.rs`:

    cd /Users/sac/graphlaw
    cargo run --example naive_physics_vectors --features abi --release

Each vector translates the wasm4pm `naive_physics` breed's scene (np:* facts)
into a positive-Datalog program and runs GraphLaw's admitted Datalog authority
(PurRDF least fixpoint) over it. The generator cross-checks the Datalog result
against an independent reference fixpoint before writing.

## Vector fields

- `vector` — case id (`naive-physics-{paper,support-chain,spill,no-fall}`).
- `breed` — always `naive_physics`.
- `engine` — the executing authority string.
- `provenance` — Hayes 1979/1985 sources, the four named axioms, case note.
- `input.facts` — the breed-contract np:* facts (same shape as
  `tests/fixtures/papers/naive_physics.json`).
- `datalog.rules` — head/body triple patterns with `?var` variables
  (GraphLaw ABI `datalog` op shape).
- `datalog.facts` — EDB triples under `https://np.graphlaw.dev/`.
- `expected.falls` / `expected.spills` — sorted derived atoms.
- `expected.not_falls` — objects asserted stable (deriving a fall for one is a
  defect; over-derivation gate).
- `expected.value` — `verified` only after generator-side checks pass.

## Axiom -> rule encoding (see ENCODERS.md)

All four Hayes axioms are positive Horn clauses; removal is an event fact
(`np:remove`), never negation.
"#.to_string()
}

fn encoders_doc() -> String {
    r#"# Encoders — naive_physics breed -> GraphLaw Datalog

Namespace: `https://np.graphlaw.dev/`.

## Fact encoders (np:* contract -> EDB triples [s p o])

| breed fact | Datalog triple |
|---|---|
| `np:on:a = b` | `a np:on b` |
| `np:in:a = c` | `a np:in c` |
| `np:liquid:l = c` | `l np:liquid c` |
| `np:ground:x = true` | `x np:ground np:true` |
| `np:remove:x = true` | `x np:remove np:true` |

## Rules (the four axioms, positive Horn)

1. `sup(x,y) :- on(x,y)` and `sup(x,y) :- in(x,y)` — ax-support base.
2. `sup(x,z) :- sup(x,y), sup(y,z)` — ax-support transitivity.
3. `falls(x) :- sup(x,y), removed(y)` — ax-unsupported-falls seed.
4. `falls(x) :- sup(x,y), falls(y)` — falls fixpoint cascade; doubles as
   ax-containment-transport because `in` feeds `sup`.
5. `spills(l) :- liquid(l,c), falls(c)` and
   `spills(l) :- liquid(l,c), removed(c)` — ax-liquid-spill.

## Removal-as-event

`np:remove:x` is a positive fact. Removed objects never enter `falls` (no rule
derives falls from `removed(x)` with x in the head), matching the breed, which
skips removed objects in its fixpoint. Cascades arise purely through rules 3+4.

## Determinism

Sorted EDB facts, order-independent rules; re-running the generator is
byte-idempotent.
"#.to_string()
}
