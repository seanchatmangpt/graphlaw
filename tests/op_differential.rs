//! Cross-op differential properties over the real wasm module and the native
//! dispatcher. Every property is computed through the public JSON ABI only;
//! each request is run on both runtimes and must be byte-identical before any
//! cross-op property is asserted.
//!
//! Set `GRAPHLAW_WASM` to test a prebuilt module (see `tests/common`).
#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::collections::BTreeSet;

use serde_json::{Value, json};

const NS: &str = "https://example.org/";

/// Successful response (harness asserts `ok: true` and native == wasm bytes).
fn run_ok(req: Value) -> Value {
    let body = common::ok(&req);
    assert!(body.is_object(), "ok body is an object for {req}");
    body
}

/// The `error` object of a refusal (harness asserts `ok: false` and identity).
fn run_refused(req: Value) -> Value {
    common::refused(&req)["error"].clone()
}

const RDF_DIALECTS: [&str; 9] = [
    "turtle",
    "trig",
    "ntriples",
    "nquads",
    "rdfxml",
    "jsonld",
    "yamlld",
    "trix",
    "hextuples",
];

fn base_turtle() -> &'static str {
    "@prefix ex: <https://example.org/> .\n\
     ex:a ex:knows ex:b , ex:c ;\n\
          ex:name \"Alice\" ;\n\
          ex:age 42 .\n\
     ex:b ex:knows ex:c ;\n\
          ex:label \"Bob\"@en .\n"
}

fn lines(s: &str) -> BTreeSet<String> {
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

fn s(v: &Value, k: &str) -> String {
    v[k].as_str()
        .unwrap_or_else(|| panic!("`{k}` is a string in {v}"))
        .to_string()
}

// ---------------------------------------------------------------- positive control

#[test]
fn a_control_parse_and_canonical_succeed_on_the_base_fixture() {
    let p = run_ok(json!({"op": "parse", "text": base_turtle(), "dialect": "turtle"}));
    assert_eq!(p["quads"], 6);
    assert!(s(&p, "id").starts_with("sha256:"), "id: {}", s(&p, "id"));
    let c =
        run_ok(json!({"op": "canonical", "data": {"text": base_turtle(), "dialect": "turtle"}}));
    assert_eq!(c["quads"], 6);
    assert_eq!(p["id"], c["id"]);
}

#[test]
fn b_control_a_changed_graph_changes_the_id() {
    let a = run_ok(json!({"op": "parse", "text": base_turtle(), "dialect": "turtle"}));
    let other = format!("{}<{NS}z> <{NS}p> <{NS}q> .\n", "");
    let b = run_ok(json!({"op": "parse", "text": other, "dialect": "ntriples"}));
    assert_ne!(a["id"], b["id"], "the id must discriminate distinct graphs");
}

// ---------------------------------------------------------------- id agreement

#[test]
fn parse_canonical_convert_ids_agree_across_all_nine_rdf_dialects() {
    let base = run_ok(json!({"op": "parse", "text": base_turtle(), "dialect": "turtle"}));
    let want_id = s(&base, "id");
    let want_quads = base["quads"].clone();
    for d in RDF_DIALECTS {
        let conv = run_ok(json!({
            "op": "convert", "text": base_turtle(), "dialect": "turtle", "to": d}));
        assert_eq!(s(&conv, "id"), want_id, "convert.id for {d}");
        let text = s(&conv, "text");

        let p = run_ok(json!({"op": "parse", "text": text, "dialect": d}));
        assert_eq!(s(&p, "id"), want_id, "parse.id of {d} round trip");
        assert_eq!(p["quads"], want_quads, "parse.quads of {d} round trip");

        let c = run_ok(json!({"op": "canonical", "data": {"text": text, "dialect": d}}));
        assert_eq!(s(&c, "id"), want_id, "canonical.id of {d} round trip");
        assert_eq!(c["quads"], want_quads, "canonical.quads of {d} round trip");

        let again = run_ok(json!({
            "op": "convert", "text": text, "dialect": d, "to": "turtle"}));
        assert_eq!(
            s(&again, "id"),
            want_id,
            "convert.id back to turtle from {d}"
        );
    }
}

#[test]
fn canonical_nquads_is_identical_whatever_dialect_the_data_arrived_in() {
    let base = run_ok(json!({
        "op": "canonical", "data": {"text": base_turtle(), "dialect": "turtle"}}));
    for d in RDF_DIALECTS {
        let conv = run_ok(json!({
            "op": "convert", "text": base_turtle(), "dialect": "turtle", "to": d}));
        let c = run_ok(json!({
            "op": "canonical", "data": {"text": s(&conv, "text"), "dialect": d}}));
        assert_eq!(c["nquads"], base["nquads"], "canonical nquads via {d}");
    }
}

// ---------------------------------------------------------------- sparql vs canonical

#[test]
fn sparql_construct_quads_equal_canonical_quads_of_the_same_data() {
    let data = json!({"text": base_turtle(), "dialect": "turtle"});
    let canon = run_ok(json!({"op": "canonical", "data": data}));
    let g = run_ok(json!({
        "op": "sparql", "data": data,
        "query": "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }"}));
    assert_eq!(g["kind"], "graph");
    assert_eq!(
        g["quads"], canon["quads"],
        "CONSTRUCT quads == canonical quads"
    );
    assert_eq!(
        lines(&s(&g, "nquads")),
        lines(&s(&canon, "nquads")),
        "CONSTRUCT statements == canonical statements"
    );
}

#[test]
fn sparql_select_row_count_matches_canonical_quads() {
    let data = json!({"text": base_turtle(), "dialect": "turtle"});
    let canon = run_ok(json!({"op": "canonical", "data": data}));
    let sel = run_ok(json!({
        "op": "sparql", "data": data, "query": "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"}));
    assert_eq!(sel["kind"], "solutions");
    assert_eq!(
        sel["rows"].as_array().unwrap().len() as u64,
        canon["quads"].as_u64().unwrap()
    );
    let ask = run_ok(json!({
        "op": "sparql", "data": data,
        "query": format!("ASK {{ <{NS}a> <{NS}knows> <{NS}b> }}")}));
    assert_eq!(ask["kind"], "boolean");
    assert_eq!(ask["value"], true);
}

// ---------------------------------------------------------------- sniff vs parse

/// Whenever `parse` accepts undeclared text, the dialect it reports is the
/// dialect `sniff` reports; `parse` never succeeds where `sniff` refuses.
#[test]
fn sniff_dialect_agrees_with_parse_dialect_for_every_serialization() {
    let mut agreed = 0;
    for d in RDF_DIALECTS {
        let conv = run_ok(json!({
            "op": "convert", "text": base_turtle(), "dialect": "turtle", "to": d}));
        let text = s(&conv, "text");
        let sn = common::native(&json!({"op": "sniff", "text": text}));
        let pa = common::native(&json!({"op": "parse", "text": text}));
        assert_eq!(
            common::wasm(&json!({"op": "sniff", "text": text})),
            sn,
            "sniff native == wasm for {d}"
        );
        if pa["ok"] == true {
            assert_eq!(
                sn["ok"], true,
                "parse succeeded where sniff refused for {d}"
            );
            assert_eq!(sn["dialect"], pa["dialect"], "sniff vs parse for {d}");
            agreed += 1;
        }
    }
    assert!(
        agreed >= 1,
        "at least one serialization is sniffable and parses"
    );
    let t = run_ok(json!({"op": "sniff", "text": base_turtle()}));
    let p = run_ok(json!({"op": "parse", "text": base_turtle()}));
    assert_eq!(t["dialect"], p["dialect"]);
    assert_eq!(t["dialect"], "Turtle");
}

/// Regression court for a defect observed on the v26.9.29 tree: `sniff` reported NTriples for
/// Turtle that has `@prefix` lines but only absolute-IRI triples (the shape
/// `convert ... to turtle` emits), and undeclared `parse` then refused it.
#[test]
fn sniff_success_implies_parse_success_for_converted_turtle() {
    let conv = run_ok(json!({
        "op": "convert", "text": base_turtle(), "dialect": "turtle", "to": "turtle"}));
    let text = s(&conv, "text");
    let sn = run_ok(json!({"op": "sniff", "text": text}));
    assert_eq!(sn["dialect"], "Turtle", "sniff of converted turtle");
    let pa = run_ok(json!({"op": "parse", "text": text}));
    assert_eq!(pa["dialect"], sn["dialect"]);
}

#[test]
fn sniff_hint_agrees_with_parse_hint() {
    let sn = run_ok(json!({"op": "sniff", "text": base_turtle(), "hint": "data.ttl"}));
    let pa = run_ok(json!({"op": "parse", "text": base_turtle(), "hint": "data.ttl"}));
    assert_eq!(sn["dialect"], pa["dialect"]);
}

// ---------------------------------------------------------------- entail delta

#[test]
fn entail_rdfs_added_equals_the_quad_delta_against_canonical() {
    let text = "@prefix ex: <https://example.org/> .\n\
                @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                ex:Cat rdfs:subClassOf ex:Animal .\n\
                ex:Animal rdfs:subClassOf ex:Thing .\n\
                ex:tom a ex:Cat .\n";
    let data = json!({"text": text, "dialect": "turtle"});
    let canon = run_ok(json!({"op": "canonical", "data": data}));
    let q0 = canon["quads"].as_u64().unwrap();
    for regime in ["rdfs", "owl-rl"] {
        let e = run_ok(json!({"op": "entail", "data": data, "regime": regime}));
        let added = e["added"].as_u64().unwrap();
        assert!(
            added > 0,
            "{regime} derives something from a subclass chain"
        );
        let closure = run_ok(json!({
            "op": "parse", "text": s(&e, "nquads"), "dialect": "nquads"}));
        let q1 = closure["quads"].as_u64().unwrap();
        assert_eq!(
            q1 - q0,
            added,
            "{regime}: added == parsed closure - canonical"
        );
        assert!(
            e["nquads"].as_str().unwrap().contains(&format!(
                "<{NS}tom> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{NS}Thing>"
            )),
            "{regime} closes the subclass chain"
        );
    }
}

#[test]
fn entail_closure_is_a_fixpoint() {
    let text = format!(
        "<{NS}Cat> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <{NS}Animal> .\n\
         <{NS}tom> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{NS}Cat> .\n"
    );
    let first = run_ok(json!({
        "op": "entail", "data": {"text": text, "dialect": "ntriples"}, "regime": "rdfs"}));
    let second = run_ok(json!({
        "op": "entail", "data": {"text": s(&first, "nquads"), "dialect": "nquads"},
        "regime": "rdfs"}));
    assert_eq!(second["added"], 0, "materializing a closure adds nothing");
}

// ---------------------------------------------------------------- closure differential

/// Deterministic LCG edge set (cycles and self-loops included).
fn edges(seed: u64, nodes: usize, count: usize) -> BTreeSet<(usize, usize)> {
    let step = |x: u64| {
        x.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
    };
    let mut x = step(seed);
    let mut out = BTreeSet::new();
    for _ in 0..count {
        x = step(x);
        let a = (x >> 33) as usize % nodes;
        x = step(x);
        let b = (x >> 33) as usize % nodes;
        out.insert((a, b));
    }
    out
}

fn iri(n: usize) -> String {
    format!("{NS}n{n}")
}

fn nt_edges(es: &BTreeSet<(usize, usize)>) -> String {
    es.iter()
        .map(|(a, b)| format!("<{}> <{NS}parent> <{}> .\n", iri(*a), iri(*b)))
        .collect()
}

fn warshall(es: &BTreeSet<(usize, usize)>, nodes: usize) -> BTreeSet<(String, String)> {
    let mut r = vec![vec![false; nodes]; nodes];
    for (a, b) in es {
        r[*a][*b] = true;
    }
    for k in 0..nodes {
        for i in 0..nodes {
            for j in 0..nodes {
                if r[i][k] && r[k][j] {
                    r[i][j] = true;
                }
            }
        }
    }
    r.iter()
        .enumerate()
        .flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, reach)| **reach)
                .map(move |(j, _)| (iri(i), iri(j)))
        })
        .collect()
}

/// `ex:ancestor` pairs (bare IRIs) from N-Triples / N-Quads text.
fn ancestor_pairs(text: &str) -> BTreeSet<(String, String)> {
    let want = format!("<{NS}ancestor>");
    let bare = |t: &str| t.trim_start_matches('<').trim_end_matches('>').to_string();
    text.lines()
        .filter_map(|l| {
            let mut t = l.split_whitespace();
            let (a, p, o) = (t.next()?, t.next()?, t.next()?);
            (p == want).then(|| (bare(a), bare(o)))
        })
        .collect()
}

fn datalog_pairs(es: &BTreeSet<(usize, usize)>) -> BTreeSet<(String, String)> {
    let anc = format!("{NS}ancestor");
    let par = format!("{NS}parent");
    let facts: Vec<Value> = es
        .iter()
        .map(|(a, b)| json!([iri(*a), par, iri(*b)]))
        .collect();
    let r = run_ok(json!({"op": "datalog",
        "rules": [
            {"head": ["?x", anc, "?y"], "body": [["?x", par, "?y"]]},
            {"head": ["?x", anc, "?z"], "body": [["?x", anc, "?y"], ["?y", par, "?z"]]}],
        "facts": facts}));
    assert_eq!(
        r["count"].as_u64().unwrap() as usize,
        r["facts"].as_array().unwrap().len()
    );
    r["facts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f[1] == anc.as_str())
        .map(|f| (s_of(&f[0]), s_of(&f[2])))
        .collect()
}

fn s_of(v: &Value) -> String {
    v.as_str().expect("string term").to_string()
}

fn n3_pairs(es: &BTreeSet<(usize, usize)>) -> BTreeSet<(String, String)> {
    let doc = format!(
        "{}\n{{ ?x <{NS}parent> ?y }} => {{ ?x <{NS}ancestor> ?y }} .\n\
         {{ ?x <{NS}ancestor> ?y . ?y <{NS}parent> ?z }} => {{ ?x <{NS}ancestor> ?z }} .\n",
        nt_edges(es)
    );
    let r = run_ok(json!({"op": "n3", "text": doc}));
    let derived = r["derived"]
        .as_str()
        .expect("n3 derived is text")
        .to_string();
    let nt = run_ok(json!({
        "op": "convert", "text": derived, "dialect": "turtle", "to": "ntriples"}));
    ancestor_pairs(&s(&nt, "text"))
}

fn owlrl_pairs(es: &BTreeSet<(usize, usize)>) -> BTreeSet<(String, String)> {
    let doc = format!(
        "{}<{NS}parent> <http://www.w3.org/2000/01/rdf-schema#subPropertyOf> <{NS}ancestor> .\n\
         <{NS}ancestor> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#TransitiveProperty> .\n",
        nt_edges(es)
    );
    let r = run_ok(json!({
        "op": "entail", "data": {"text": doc, "dialect": "ntriples"}, "regime": "owl-rl"}));
    ancestor_pairs(&s(&r, "nquads"))
}

#[test]
fn control_closure_fixture_is_non_trivial() {
    let es = edges(7, 6, 8);
    let truth = warshall(&es, 6);
    assert!(
        truth.len() > es.len(),
        "closure strictly larger than the edges"
    );
    assert!(!datalog_pairs(&es).is_empty(), "datalog derives ancestors");
}

#[test]
fn datalog_n3_and_owl_rl_agree_with_warshall_on_seeded_graphs() {
    for (seed, nodes, count) in [(7u64, 6usize, 8usize), (11, 5, 6), (2026, 7, 10)] {
        let es = edges(seed, nodes, count);
        let truth = warshall(&es, nodes);
        let dl = datalog_pairs(&es);
        let n3 = n3_pairs(&es);
        let owl = owlrl_pairs(&es);
        assert_eq!(dl, truth, "datalog vs warshall (seed {seed})");
        assert_eq!(n3, dl, "n3 vs datalog (seed {seed})");
        assert_eq!(owl, dl, "owl-rl vs datalog (seed {seed})");
    }
}

// ---------------------------------------------------------------- hooks idempotence

fn read(rel: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn hooks_rerun_on_its_own_output_fires_nothing_and_keeps_the_id() {
    let pack = json!({"text": read("packs/self-monitoring-pack/hook.ttl"), "dialect": "turtle"});
    let data = json!({
        "text": read("packs/self-monitoring-pack/fixtures/session-real-broad-topic.ttl"),
        "dialect": "turtle"});
    let first = run_ok(json!({"op": "hooks", "pack": pack, "data": data}));
    assert!(
        !first["firings"].as_array().unwrap().is_empty(),
        "positive control: the first run fires"
    );
    let second = run_ok(json!({
        "op": "hooks", "pack": pack,
        "data": {"text": first["nquads"], "dialect": "nquads"}}));
    assert!(second["firings"].as_array().unwrap().is_empty());
    assert_eq!(second["id"], first["id"]);
    assert_eq!(second["quads"], first["quads"]);
    let third = run_ok(json!({
        "op": "hooks", "pack": pack,
        "data": {"text": second["nquads"], "dialect": "nquads"}}));
    assert!(third["firings"].as_array().unwrap().is_empty());
    assert_eq!(third["id"], first["id"]);
}

// ---------------------------------------------------------------- law vs parse

#[test]
fn law_shacl_only_first_state_is_the_parse_id() {
    let data = "@prefix ex: <https://example.org/> .\nex:a a ex:T ; ex:name \"x\" .\n";
    let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
                  @prefix ex: <https://example.org/> .\n\
                  ex:S a sh:NodeShape ; sh:targetClass ex:T ;\n\
                    sh:property [ sh:path ex:name ; sh:minCount 1 ] .\n";
    let p = run_ok(json!({"op": "parse", "text": data, "dialect": "turtle"}));
    let l = run_ok(json!({"op": "law",
        "data": {"text": data, "dialect": "turtle"},
        "steps": [{"step": "shacl", "shapes": shapes}]}));
    let states = l["states"].as_array().unwrap();
    assert!(!states.is_empty());
    assert_eq!(states[0], p["id"], "law states[0] == parse id");
    assert_eq!(l["receipts"].as_array().unwrap().len(), 1);
    let re = run_ok(json!({"op": "parse", "text": s(&l, "nquads"), "dialect": "nquads"}));
    assert_eq!(
        states[states.len() - 1],
        re["id"],
        "final state id == id of returned nquads"
    );
}

#[test]
fn law_shacl_refusal_is_typed_and_identical_on_both_runtimes() {
    let data = "@prefix ex: <https://example.org/> .\nex:a a ex:T .\n";
    let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
                  @prefix ex: <https://example.org/> .\n\
                  ex:S a sh:NodeShape ; sh:targetClass ex:T ;\n\
                    sh:property [ sh:path ex:name ; sh:minCount 1 ] .\n";
    let e = run_refused(json!({"op": "law",
        "data": {"text": data, "dialect": "turtle"},
        "steps": [{"step": "shacl", "shapes": shapes}]}));
    assert_eq!(e["details"]["code"], "NotAdmitted");
}
