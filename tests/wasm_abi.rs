//! Runs the *compiled* WebAssembly module in a real wasm runtime (wasmi) and
//! drives every dialect and capability through the JSON ABI, exactly as a host
//! such as Elixir/Wasmex would. Set `GRAPHLAW_WASM` to test a prebuilt module;
//! otherwise the module is built once into `target/wasm-abi`.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use serde_json::{Value, json};
use wasmi::{Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmi_wasi::{WasiCtx, WasiCtxBuilder};

fn wasm_path() -> PathBuf {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        return p.into();
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target = root.join("target/wasm-abi");
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .current_dir(&root)
        .args([
            "build",
            "-p",
            "graphlaw-wasm",
            "--target",
            "wasm32-wasip1",
            "--profile",
            "wasm",
            "--target-dir",
        ])
        .arg(&target)
        .status()
        .expect("cargo runs");
    assert!(
        status.success(),
        "wasm build failed (is the wasm32-wasip1 target installed?)"
    );
    target.join("wasm32-wasip1/wasm/graphlaw_wasm.wasm")
}

struct Host {
    store: Store<WasiCtx>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), u64>,
}

impl Host {
    fn new() -> Self {
        let bytes = std::fs::read(wasm_path()).expect("wasm artifact");
        let engine = Engine::default();
        let module = Module::new(&engine, &bytes[..]).expect("valid wasm");
        // The only imports allowed are WASI's (clock, random, stdio): no JavaScript glue.
        for i in module.imports() {
            assert_eq!(
                i.module(),
                "wasi_snapshot_preview1",
                "unexpected host import {}::{}",
                i.module(),
                i.name()
            );
        }
        let mut store = Store::new(&engine, WasiCtxBuilder::new().build());
        let mut linker = <Linker<WasiCtx>>::new(&engine);
        wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx).expect("links WASI");
        let instance: Instance = linker
            .instantiate_and_start(&mut store, &module)
            .expect("instantiates");
        // Reactor-style modules expose `_initialize`; hosts must call it once.
        if let Ok(init) = instance.get_typed_func::<(), ()>(&store, "_initialize") {
            init.call(&mut store, ()).expect("_initialize");
        }
        let memory = instance
            .get_memory(&store, "memory")
            .expect("exports memory");
        Host {
            alloc: instance.get_typed_func(&store, "gl_alloc").unwrap(),
            free: instance.get_typed_func(&store, "gl_free").unwrap(),
            call: instance.get_typed_func(&store, "gl_call").unwrap(),
            store,
            memory,
        }
    }

    fn request(&mut self, req: &Value) -> Value {
        let body = req.to_string().into_bytes();
        let ptr = self.alloc.call(&mut self.store, body.len() as u32).unwrap();
        self.memory
            .write(&mut self.store, ptr as usize, &body)
            .unwrap();
        let packed = self
            .call
            .call(&mut self.store, (ptr, body.len() as u32))
            .unwrap();
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .unwrap();
        self.free.call(&mut self.store, (out_ptr, out_len)).unwrap();
        serde_json::from_slice(&out).expect("response is JSON")
    }
}

/// One shared instance: instantiation is the expensive part.
fn host() -> &'static Mutex<Host> {
    static H: OnceLock<Mutex<Host>> = OnceLock::new();
    H.get_or_init(|| Mutex::new(Host::new()))
}

fn call(req: Value) -> Value {
    host()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .request(&req)
}

fn ok(req: Value) -> Value {
    let r = call(req.clone());
    assert_eq!(r["ok"], json!(true), "request {req} failed: {r}");
    r
}

fn read(p: &str) -> String {
    std::fs::read_to_string(format!("{}/{p}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

const SAMPLE: &str = "@prefix ex: <https://e/> . ex:s ex:p ex:o, \"caf\\u00e9\"@fr, 42 ; a ex:T .";

#[test]
fn module_is_self_contained_and_reports_capabilities() {
    let r = ok(json!({"op": "capabilities"}));
    assert_eq!(r["abi"], 1);
    let caps: Vec<_> = r["authorities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["capability"].as_str().unwrap())
        .collect();
    for c in [
        "SPARQL 1.1/1.2",
        "SHACL",
        "ShEx 2.1",
        "Datalog",
        "Notation3",
    ] {
        assert!(caps.contains(&c), "{c} missing from {caps:?}");
    }
    assert!(caps.iter().any(|c| c.starts_with("Knowledge hooks")));
}

#[test]
fn every_rdf_dialect_round_trips_through_wasm() {
    let base = ok(json!({"op": "parse", "text": SAMPLE, "dialect": "turtle"}));
    let id = base["id"].clone();
    assert_eq!(base["quads"], 4);
    for dialect in [
        "turtle",
        "trig",
        "ntriples",
        "nquads",
        "rdfxml",
        "jsonld",
        "yamlld",
        "trix",
        "hextuples",
    ] {
        let out = ok(json!({"op": "convert", "text": SAMPLE, "dialect": "turtle", "to": dialect}));
        assert_eq!(out["id"], id);
        let text = out["text"].as_str().unwrap();
        assert!(!text.is_empty(), "{dialect} produced no output");
        let back = ok(json!({"op": "parse", "text": text, "dialect": dialect}));
        assert_eq!(back["id"], id, "{dialect} does not round-trip");
        assert_eq!(back["quads"], 4, "{dialect}");
    }
}

#[test]
fn content_routing_works_inside_wasm() {
    for (text, expect) in [
        ("<https://e/s> <https://e/p> <https://e/o> .\n", "NTriples"),
        ("@prefix e: <https://e/> . e:s e:p e:o .", "Turtle"),
        ("{ ?x a <https://e/A> } => { ?x a <https://e/B> } .", "N3"),
        ("SELECT * WHERE { ?s ?p ?o }", "Sparql"),
        (
            "<?xml version=\"1.0\"?><rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"/>",
            "RdfXml",
        ),
    ] {
        assert_eq!(
            ok(json!({"op": "sniff", "text": text}))["dialect"],
            expect,
            "{text}"
        );
    }
    let refused = call(json!({"op": "sniff", "text": "<!DOCTYPE html><html></html>"}));
    assert_eq!(refused["ok"], false);
    assert_eq!(refused["error"]["kind"], "NotSemanticContent");
}

#[test]
fn sparql_select_construct_ask() {
    let data = json!({"text": SAMPLE, "dialect": "turtle"});
    let s = ok(
        json!({"op": "sparql", "data": data, "query": "SELECT ?o WHERE { <https://e/s> <https://e/p> ?o } ORDER BY ?o"}),
    );
    assert_eq!(s["kind"], "solutions");
    assert_eq!(s["rows"].as_array().unwrap().len(), 3);
    let c = ok(
        json!({"op": "sparql", "data": data, "query": "CONSTRUCT { ?s a <https://e/U> } WHERE { ?s a <https://e/T> }"}),
    );
    assert_eq!(
        (c["kind"].as_str(), c["quads"].as_u64()),
        (Some("graph"), Some(1))
    );
    let a =
        ok(json!({"op": "sparql", "data": data, "query": "ASK { <https://e/s> a <https://e/T> }"}));
    assert_eq!(a["value"], true);
}

#[test]
fn shacl_validates() {
    let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> . @prefix ex: <https://e/> . @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
                  ex:S a sh:NodeShape ; sh:targetClass ex:T ; sh:property [ sh:path ex:p ; sh:datatype xsd:integer ] .";
    let r =
        ok(json!({"op": "shacl", "data": {"text": SAMPLE, "dialect": "turtle"}, "shapes": shapes}));
    assert_eq!(r["conforms"], false);
    assert!(!r["results"].as_array().unwrap().is_empty());
    let good = "@prefix ex: <https://e/> . ex:s a ex:T ; ex:p 1 .";
    assert_eq!(
        ok(json!({"op": "shacl", "data": {"text": good}, "shapes": shapes}))["conforms"],
        true
    );
}

#[test]
fn shex_validates_shexc_and_shexj() {
    let data = json!({"text": "@prefix ex: <https://e/> . ex:cat ex:says \"meow\" . ex:rock ex:weight 5 ."});
    let shexc = "PREFIX ex: <https://e/>\nex:Cat { ex:says . }";
    let r = ok(
        json!({"op": "shex", "data": data, "schema": shexc, "map": "<https://e/cat>@<https://e/Cat>, <https://e/rock>@<https://e/Cat>"}),
    );
    let statuses: Vec<_> = r["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, ["Conformant", "Nonconformant"]);

    let shexj = read("innovation/candidate.shex.json");
    let j = ok(
        json!({"op": "shex", "schema_dialect": "shexj", "schema": shexj,
        "data": {"text": "@prefix i: <https://praxis.chatman.io/innovation#> . <https://e/c> i:replaces <https://e/a> ."},
        "map": "<https://e/c>@<https://praxis.chatman.io/innovation#CandidateFutureShape>"}),
    );
    assert_eq!(
        j["conforms"], false,
        "incomplete candidate must not conform: {j}"
    );
}

#[test]
fn n3_reasons() {
    let r = ok(
        json!({"op": "n3", "text": "@prefix : <https://e/> . :s a :Human . { ?x a :Human } => { ?x a :Mortal } ."}),
    );
    assert!(r["derived"].as_str().unwrap().contains("Mortal"));
    let refused = call(json!({"op": "n3", "text": "{ ?x a } =>"}));
    assert_eq!(
        (refused["ok"].clone(), refused["error"]["engine"].clone()),
        (json!(false), json!("Eyeron"))
    );
}

#[test]
fn every_entailment_regime_executes() {
    let data = json!({"text": "@prefix ex: <https://e/> . @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> . ex:Cat rdfs:subClassOf ex:Animal . ex:tom a ex:Cat ."});
    for regime in ["simple", "rdf", "rdfs", "owl-rl", "d"] {
        let r = ok(json!({"op": "entail", "data": data, "regime": regime}));
        assert!(r["nquads"].as_str().unwrap().contains("Cat"), "{regime}");
        if matches!(regime, "rdfs" | "owl-rl") {
            assert!(r["nquads"].as_str().unwrap().contains("<https://e/tom> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Animal>"), "{regime}");
        }
    }
}

#[test]
fn datalog_computes_a_least_fixpoint() {
    let r = ok(json!({"op": "datalog",
        "rules": [
            {"head": ["?x", "https://e/anc", "?y"], "body": [["?x", "https://e/par", "?y"]]},
            {"head": ["?x", "https://e/anc", "?z"], "body": [["?x", "https://e/anc", "?y"], ["?y", "https://e/par", "?z"]]}],
        "facts": [["https://e/a", "https://e/par", "https://e/b"], ["https://e/b", "https://e/par", "https://e/c"]]}));
    let facts = r["facts"].as_array().unwrap();
    assert!(facts.contains(&json!(["https://e/a", "https://e/anc", "https://e/c"])));
    assert_eq!(facts.len(), 5);
}

#[test]
fn knowledge_hooks_execute_in_wasm() {
    let r = ok(json!({"op": "hooks",
        "pack": {"text": read("packs/self-monitoring-pack/hook.ttl"), "dialect": "turtle"},
        "data": {"text": read("packs/self-monitoring-pack/fixtures/session-real-broad-topic.ttl"), "dialect": "turtle"}}));
    assert_eq!(r["firings"].as_array().unwrap().len(), 3);
    let again = ok(json!({"op": "hooks",
        "pack": {"text": read("packs/self-monitoring-pack/hook.ttl"), "dialect": "turtle"},
        "data": {"text": r["nquads"], "dialect": "nquads"}}));
    assert_eq!(
        again["firings"].as_array().unwrap().len(),
        0,
        "saturated state fires nothing"
    );
    assert_eq!(again["id"], r["id"]);
}

#[test]
fn law_pipeline_runs_end_to_end_in_wasm() {
    let r = ok(json!({"op": "law",
        "data": {"text": read("innovation/industry-fixture.n3"), "dialect": "turtle"},
        "steps": [
            {"step": "n3", "rules": read("innovation/blue-ocean-triz.n3")},
            {"step": "shacl", "shapes": read("innovation/candidate.shacl.ttl")}]}));
    let receipts = r["receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 2);
    assert_eq!(receipts[0]["authority"], "eyeron");
    assert_eq!(receipts[1]["authority"], "purrdf::shapes");
    let ids = r["states"].as_array().unwrap();
    assert_ne!(ids[0], ids[1]);
    assert_eq!(
        ids[1], ids[2],
        "an admission gate does not change the state"
    );

    let refused = call(json!({"op": "law",
        "data": {"text": "<https://e/c> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://praxis.chatman.io/innovation#CandidateFuture> .", "dialect": "ntriples"},
        "steps": [{"step": "shacl", "shapes": read("innovation/candidate.shacl.ttl")}]}));
    assert_eq!(refused["ok"], false);
    assert!(
        refused["error"]["message"]
            .as_str()
            .unwrap()
            .contains("admission refused")
    );
}

#[test]
fn protocol_errors_are_json_not_traps() {
    let r = host()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .request(&json!("not an object"));
    assert_eq!(r["ok"], false);
    assert_eq!(call(json!({"op": "nope"}))["ok"], false);
    assert_eq!(
        call(json!({"op": "parse", "text": "<a> <b> .", "dialect": "turtle"}))["error"]["engine"],
        "PurRdf"
    );
}
