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
        serde_json::from_slice(&self.request_bytes(req)).expect("response is JSON")
    }

    /// `gl_call` on an arbitrary (ptr, len) pair; reads and frees the response.
    fn raw_call(&mut self, ptr: u32, len: u32) -> Value {
        let packed = self.call.call(&mut self.store, (ptr, len)).unwrap();
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .unwrap();
        self.free.call(&mut self.store, (out_ptr, out_len)).unwrap();
        serde_json::from_slice(&out).expect("response is JSON")
    }

    fn request_bytes(&mut self, req: &Value) -> Vec<u8> {
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
        out
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
fn plan_admission_runs_in_wasm() {
    let at = |o: &str| format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n");
    let mv =
        |n: &str, f: &str, t: &str| json!({"name": n, "pre": at(f), "add": at(t), "del": at(f)});
    let steps = |mid_pre: &str| {
        json!([{"step": "plan", "plan": {
            "actions": [mv("a-b", "a", "b"),
                        {"name": "b-c", "pre": at(mid_pre), "add": at("c"), "del": at("b")}],
            "goal": at("c")}}])
    };
    let r = ok(json!({"op": "law",
        "data": {"text": at("a"), "dialect": "ntriples"}, "steps": steps("b")}));
    let receipts = r["receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 2);
    assert_eq!(receipts[0]["step"], "plan-action");
    assert_eq!(receipts[0]["child"], receipts[1]["parent"]);
    assert_eq!(r["states"].as_array().unwrap().len(), 3);

    let refused = call(json!({"op": "law",
        "data": {"text": at("a"), "dialect": "ntriples"}, "steps": steps("z")}));
    assert_eq!(refused["ok"], false);
    let msg = refused["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("plan refused at step 1") && msg.contains("urn:p:z"),
        "{msg}"
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

#[cfg(feature = "abi")]
#[test]
fn native_and_wasm_law_plan_responses_are_byte_identical() {
    let at = |o: &str| format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n");
    let req = json!({"op": "law", "data": {"text": at("a"), "dialect": "ntriples"},
        "steps": [
            {"step": "plan", "plan": {
                "actions": [{"name": "a-b", "pre": at("a"), "add": at("b"), "del": at("a")}],
                "goal": at("b")}},
            {"step": "record-receipts"},
            {"step": "require-receipt", "step_name": "plan-action"}]});
    let native = graphlaw::abi::call(req.to_string().as_bytes());
    let wasm = host()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .request_bytes(&req);
    assert_eq!(native, wasm, "native and wasm responses diverge");
    let v: Value = serde_json::from_slice(&native).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    assert!(v["receipts"][0]["plan_sha256"].is_string());
    assert_eq!(ok(json!({"op": "capabilities"}))["abi_version"], 1);
}

fn lease_req(ceiling: &str, scope: Value, expires: u64, now: u64) -> Value {
    json!({"op": "law",
        "data": {"text": "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n<urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n", "dialect": "ntriples"},
        "lease": {"id": "L1", "holder": "h", "ceiling": ceiling, "scope": scope, "expires_unix": expires},
        "now_unix": now,
        "unverified_lease": true,
        "steps": [{"step": "rdfs"}]})
}

#[test]
fn wasm_leased_law_steps_carry_lease_id_and_refuse_typed() {
    let r = ok(lease_req("construct", json!(["derive:rdfs"]), 100, 50));
    assert_eq!(r["receipts"][0]["lease_id"], "L1");
    for (req, reason) in [
        (
            lease_req("construct", json!(["derive:rdfs"]), 100, 100),
            "expired",
        ),
        (
            lease_req("construct", json!(["derive:n3"]), 100, 1),
            "out_of_scope",
        ),
        (
            lease_req("select", json!(["derive:rdfs"]), 100, 1),
            "ceiling",
        ),
    ] {
        let e = call(req);
        assert_eq!(e["ok"], false);
        assert!(
            e["error"]["message"].as_str().unwrap().contains(reason),
            "{e}"
        );
    }
    // unleased receipts carry no lease_id key
    let plain = ok(json!({"op": "law",
        "data": {"text": "<urn:a:x> <urn:a:p> <urn:a:y> .\n", "dialect": "ntriples"},
        "steps": [{"step": "rdfs"}]}));
    assert!(plain["receipts"][0].get("lease_id").is_none());
}

#[test]
fn fond_policy_admission_runs_in_wasm() {
    let problem = json!({
        "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"],
        "goal": {"facts": ["done"]},
        "transitions": [
            {"action": "flip", "from": "s0", "to": "g", "probability_ppm": 500000},
            {"action": "flip", "from": "s0", "to": "s0", "probability_ppm": 500000}]
    });
    let policy = json!({"policy": [{"state": "s0", "action": "flip", "outcomes": [
        {"state": "g", "probability_ppm": 500000},
        {"state": "s0", "probability_ppm": 500000}]}]});
    let r = ok(json!({"op": "policy", "problem": problem, "policy": policy}));
    assert_eq!(r["goal_states"], json!(["g"]));
    assert_eq!(r["entries"], json!([["s0", "flip"]]));

    let mut skewed = policy.clone();
    skewed["policy"][0]["outcomes"][0]["probability_ppm"] = json!(400000);
    let refused = call(json!({"op": "policy", "problem": problem, "policy": skewed}));
    assert_eq!(refused["ok"], false);
    assert!(
        refused["error"]["message"]
            .as_str()
            .unwrap()
            .contains("policy refused (BadMass)")
    );

    let empty = call(json!({"op": "policy", "problem": problem, "policy": {"policy": []}}));
    assert_eq!(empty["ok"], false);
}

#[test]
fn wasm_lease_boundaries_and_refusal_precedence() {
    // expiry boundary: now = expires-1 admitted, now = expires refused
    ok(lease_req("construct", json!(["derive:rdfs"]), 100, 99));
    let e = call(lease_req("construct", json!(["derive:rdfs"]), 100, 100));
    assert_eq!(e["ok"], false);
    let msg = e["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("expired") && msg.contains("L1") && msg.contains("derive:rdfs"),
        "{e}"
    );
    // far past expiry still expired
    let e = call(lease_req("construct", json!(["derive:rdfs"]), 100, 5000));
    assert!(
        e["error"]["message"].as_str().unwrap().contains("expired"),
        "{e}"
    );
    // precedence: expired outranks out_of_scope and ceiling
    let e = call(lease_req("observe", json!(["derive:n3"]), 10, 10));
    let m = e["error"]["message"].as_str().unwrap();
    assert!(
        m.contains("expired") && !m.contains("out_of_scope") && !m.contains("ceiling"),
        "{e}"
    );
    // precedence: out_of_scope outranks ceiling
    let e = call(lease_req("observe", json!(["derive:n3"]), 100, 1));
    let m = e["error"]["message"].as_str().unwrap();
    assert!(m.contains("out_of_scope") && !m.contains("ceiling"), "{e}");
    // empty scope is out_of_scope, not admitted
    let e = call(lease_req("construct", json!([]), 100, 1));
    assert!(
        e["error"]["message"]
            .as_str()
            .unwrap()
            .contains("out_of_scope"),
        "{e}"
    );
    // ceiling: observe is below construct-required step
    let e = call(lease_req("observe", json!(["derive:rdfs"]), 100, 1));
    let m = e["error"]["message"].as_str().unwrap();
    assert!(m.contains("ceiling") && m.contains("L1"), "{e}");
    // refused steps produce no receipts / state
    assert!(
        e.get("receipts").is_none() || e["receipts"].is_null(),
        "{e}"
    );
}

#[cfg(feature = "abi")]
#[test]
fn wasm_refusal_details_round_trip_and_equal_native() {
    let at = |o: &str| format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n");
    let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> . @prefix ex: <https://e/> .\n\
        ex:S a sh:NodeShape ; sh:targetClass ex:T ; sh:property [ sh:path ex:name ; sh:minCount 1 ] .\n";
    let two_bad = "<https://e/a> a <https://e/T> .\n<https://e/b> a <https://e/T> .\n";
    let problem = json!({
        "states": [{"id": "s0"}, {"id": "g", "facts": ["done"]}],
        "initial_states": ["s0"], "goal": {"facts": ["done"]},
        "transitions": [
            {"action": "flip", "from": "s0", "to": "g", "probability_ppm": 500000},
            {"action": "flip", "from": "s0", "to": "s0", "probability_ppm": 500000}]});
    let skewed = json!({"policy": [{"state": "s0", "action": "flip", "outcomes": [
        {"state": "g", "probability_ppm": 400000}, {"state": "s0", "probability_ppm": 500000}]}]});
    let cases = [
        (
            "NotAdmitted",
            json!({"op": "law",
            "data": {"text": two_bad, "dialect": "turtle"},
            "steps": [{"step": "shacl", "shapes": shapes}]}),
        ),
        (
            "PlanRefused",
            json!({"op": "law",
            "data": {"text": at("a"), "dialect": "ntriples"},
            "steps": [{"step": "plan", "plan": {"actions": [
                {"name": "a-b", "pre": at("z"), "add": at("b"), "del": at("a")}], "goal": at("b")}}]}),
        ),
        (
            "PolicyRefused",
            json!({"op": "policy", "problem": problem, "policy": skewed}),
        ),
        (
            "LeaseRefused",
            lease_req("construct", json!(["derive:n3"]), 100, 1),
        ),
        (
            "ReceiptRequired",
            json!({"op": "law",
            "data": {"text": at("a"), "dialect": "ntriples"},
            "steps": [{"step": "require-receipt", "step_name": "derive:rdfs"}]}),
        ),
    ];
    for (code, req) in cases {
        let wasm = call(req.clone());
        let native = graphlaw::abi::call_json(&req);
        assert_eq!(wasm["ok"], false, "{code}: {wasm}");
        assert_eq!(wasm["error"]["details"]["code"], code, "{wasm}");
        assert_eq!(wasm, native, "{code}: wasm and native responses differ");
    }
    let nad = call(
        json!({"op": "law", "data": {"text": two_bad, "dialect": "turtle"},
        "steps": [{"step": "shacl", "shapes": shapes}]}),
    );
    let vs = nad["error"]["details"]["violations"].as_array().unwrap();
    assert_eq!(vs.len(), 2, "{nad}");
    assert!(
        vs.iter()
            .all(|v| v["focus"].is_string() && v["path"].is_string())
    );
}

#[cfg(feature = "abi")]
#[test]
fn wasm_resource_limits_refuse_typed_and_never_trap() {
    use graphlaw::abi::{MAX_JSON_DEPTH, MAX_PLAN_ACTIONS, MAX_REQUEST_BYTES};
    let limit = |r: &Value, name: &str| {
        assert_eq!(r["ok"], false, "{r}");
        assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
        assert_eq!(r["error"]["details"]["code"], "ResourceLimit", "{r}");
        assert_eq!(r["error"]["details"]["limit"], name, "{r}");
    };
    let mut guard = host().lock().unwrap_or_else(|e| e.into_inner());
    let h = &mut *guard;

    // gl_alloc above the cap returns null (0), including the 4 GiB extreme.
    assert_eq!(h.alloc.call(&mut h.store, u32::MAX).unwrap(), 0);
    let over = (MAX_REQUEST_BYTES + 1) as u32; // a 16 MiB + 1 request body
    assert_eq!(h.alloc.call(&mut h.store, over).unwrap(), 0);
    // gl_call on that missing buffer is a typed error, not a trap.
    let r = h.raw_call(0, over);
    assert_eq!(r["ok"], false, "{r}");
    assert!(
        r["error"]["message"]
            .as_str()
            .unwrap()
            .contains("buffer missing")
    );
    // a live buffer with an over-cap declared length is refused without being read
    let live = h.alloc.call(&mut h.store, 8).unwrap();
    assert_ne!(live, 0);
    limit(&h.raw_call(live, over), "request_bytes");

    // depth-100 JSON is refused before parsing
    let mut deep = String::from(r#"{"op":"capabilities","x":"#);
    deep.push_str(&"[".repeat(99));
    deep.push_str(&"]".repeat(99));
    deep.push('}');
    let ptr = h.alloc.call(&mut h.store, deep.len() as u32).unwrap();
    h.memory
        .write(&mut h.store, ptr as usize, deep.as_bytes())
        .unwrap();
    let r = h.raw_call(ptr, deep.len() as u32);
    limit(&r, "json_depth");
    assert_eq!(r["error"]["details"]["max"], MAX_JSON_DEPTH);

    // plan with MAX_PLAN_ACTIONS + 1 actions
    let actions: Vec<Value> = (0..=MAX_PLAN_ACTIONS)
        .map(|i| json!({"name": format!("a{i}"), "pre": "", "add": "", "del": ""}))
        .collect();
    let r = h.request(&json!({"op": "law",
        "data": {"text": "<urn:p:r> <urn:p:at> <urn:p:a> .\n", "dialect": "ntriples"},
        "steps": [{"step": "plan", "plan": {"actions": actions, "goal": ""}}]}));
    limit(&r, "plan_actions");
    assert_eq!(r["error"]["details"]["observed"], MAX_PLAN_ACTIONS + 1);

    // just under the request cap still works (1 MiB body)
    let pad = "x".repeat(1 << 20);
    let r = h.request(&json!({"op": "capabilities", "pad": pad}));
    assert_eq!(r["ok"], true);
}

// ---- signed leases and receipts through the compiled module ----------------

fn issuer() -> graphlaw::attest::SigningKey {
    graphlaw::attest::SigningKey::from_seed([3; 32])
}

fn signed_lease_json(
    key: &graphlaw::attest::SigningKey,
    expires: u64,
    tamper_expires: Option<u64>,
) -> Value {
    use graphlaw::law::{Ceiling, Lease};
    let signed = graphlaw::attest::sign_lease(
        key,
        Lease {
            id: "L1".into(),
            holder: "h".into(),
            ceiling: Ceiling::Construct,
            scope: vec!["derive:rdfs".into()],
            expires_unix: expires,
            issued_unix: 0,
        },
    );
    json!({
        "lease": {"id": "L1", "holder": "h", "ceiling": "construct", "scope": ["derive:rdfs"],
                  "expires_unix": tamper_expires.unwrap_or(expires), "issued_unix": 0},
        "attestation": {"key_id": signed.attestation.key_id,
                        "payload_sha256": signed.attestation.payload_sha256,
                        "signature": signed.attestation.signature},
    })
}

fn signed_req(signed_lease: Value, trusted: &graphlaw::attest::SigningKey, now: u64) -> Value {
    json!({"op": "law",
        "data": {"text": "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n<urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n", "dialect": "ntriples"},
        "signed_lease": signed_lease,
        "trusted_keys": [trusted.verifying_key().to_hex()],
        "now_unix": now,
        "steps": [{"step": "rdfs"}]})
}

fn refusal_of(r: &Value) -> (&str, &str) {
    assert_eq!(r["ok"], false, "{r}");
    (
        r["error"]["details"]["code"].as_str().unwrap_or(""),
        r["error"]["details"]["reason"].as_str().unwrap_or(""),
    )
}

#[test]
fn f_wasm_signed_lease_with_trusted_keys_works_and_expiry_uses_the_module_clock() {
    let k = issuer();
    // far-future expiry, request `now_unix` deliberately absurd: ignored
    let good = signed_req(signed_lease_json(&k, 4_000_000_000, None), &k, u64::MAX);
    let r = ok(good.clone());
    assert_eq!(r["receipts"][0]["lease_id"], "L1");
    assert_eq!(r, graphlaw::abi::call_json(&good));

    // expired per the module's own clock; `now_unix: 0` cannot revive it
    let expired = signed_req(signed_lease_json(&k, 100, None), &k, 0);
    assert_eq!(refusal_of(&call(expired)), ("LeaseRefused", "expired"));

    // untrusted signer
    let other = graphlaw::attest::SigningKey::from_seed([4; 32]);
    let forged = signed_req(signed_lease_json(&other, 4_000_000_000, None), &k, 0);
    assert_eq!(refusal_of(&call(forged)), ("LeaseRefused", "untrusted_key"));

    // lease edited after signing (expiry stretched)
    let stretched = signed_req(signed_lease_json(&k, 100, Some(4_000_000_000)), &k, 0);
    assert_eq!(
        refusal_of(&call(stretched)),
        ("LeaseRefused", "bad_signature")
    );
}

#[test]
fn f_wasm_unsigned_lease_is_refused_unless_explicitly_unverified() {
    let mut req = lease_req("construct", json!(["derive:rdfs"]), 100, 50);
    req.as_object_mut().unwrap().remove("unverified_lease");
    let e = call(req.clone());
    assert_eq!(refusal_of(&e).0, "UnverifiedLeaseRefused", "{e}");
    assert_eq!(e, graphlaw::abi::call_json(&req));
    req["unverified_lease"] = json!(true);
    assert_eq!(ok(req)["receipts"][0]["lease_id"], "L1");
}

#[test]
fn f_wasm_require_signed_receipt_step() {
    use graphlaw::attest::{SigningKey, sign_receipt};
    use graphlaw::dialect::Dialect;
    use graphlaw::law::{LawState, Step};
    let base = LawState::parse(
        b"<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap();
    let (child, r) = base.transition(&Step::EntailRdfs).unwrap();
    let k = SigningKey::from_seed([1; 32]);
    let nq = |s: &LawState| {
        String::from_utf8(
            graphlaw::purrdf::serialize_dataset(
                s.dataset().as_ref(),
                "application/n-quads",
                graphlaw::purrdf::SerializeGraph::Dataset,
            )
            .unwrap(),
        )
        .unwrap()
    };
    let signed = nq(&graphlaw::receipt::record_signed(&child, &r, &sign_receipt(&k, &r)).unwrap());
    let unsigned = nq(&graphlaw::receipt::record(&child, &r).unwrap());
    let req = |data: &str, key: &SigningKey| {
        json!({"op": "law", "data": {"text": data, "dialect": "nquads"},
            "steps": [{"step": "require-signed-receipt", "step_name": "derive:rdfs",
                       "trusted_keys": [key.verifying_key().to_hex()]}]})
    };
    ok(req(&signed, &k));
    assert_eq!(
        refusal_of(&call(req(&signed, &SigningKey::from_seed([2; 32])))),
        ("ReceiptRefused", "untrusted_key")
    );
    assert_eq!(
        refusal_of(&call(req(&unsigned, &k))),
        ("ReceiptRefused", "unattested")
    );
}

#[test]
fn plan_negation_round_trips_in_wasm_and_matches_native() {
    let at = |o: &str| format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n");
    let occ = "<urn:c:b> <urn:p:occupied> <urn:v:yes> .\n";
    let req = |start: &str| {
        json!({"op": "law", "data": {"text": start, "dialect": "ntriples"},
            "steps": [{"step": "plan", "plan": {
                "actions": [{"name": "a-b", "pre": at("a"), "pre_not": occ,
                             "add": at("b"), "del": at("a")}],
                "goal": at("b"), "goal_not": at("a")}}]})
    };
    let blocked = req(&format!("{}{occ}", at("a")));
    let r = call(blocked.clone());
    assert_eq!(r["ok"], false, "{r}");
    let d = &r["error"]["details"];
    assert_eq!(d["code"], "PlanRefused");
    assert_eq!(d["index"], 0);
    assert!(
        d["violated_absent"][0]
            .as_str()
            .unwrap()
            .contains("urn:c:b")
    );
    assert_eq!(r, graphlaw::abi::call_json(&blocked));

    let free = req(&at("a"));
    let r = ok(free.clone());
    assert_eq!(r["receipts"].as_array().unwrap().len(), 1);
    assert_eq!(r, graphlaw::abi::call_json(&free));

    // goal_not unmet in wasm
    let mut g = free.clone();
    g["steps"][0]["plan"]["goal_not"] = json!(at("b"));
    let r = call(g.clone());
    assert_eq!(r["ok"], false);
    assert_eq!(r["error"]["details"]["action"], "<goal>");
    assert_eq!(r, graphlaw::abi::call_json(&g));
}

#[test]
fn sa2a_run12_portability_corpus_executes_through_real_wasi() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("conformance/sa2a/portable/run12");
    let mut files: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
        .collect();
    files.sort();
    assert_eq!(
        files.len(),
        50,
        "run12 must stay a 50-vector portable court"
    );
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let parsed = ok(json!({"op": "parse", "text": text, "dialect": "turtle"}));
        assert!(
            parsed["quads"].as_u64().unwrap_or(0) > 0,
            "{} parsed empty",
            path.display()
        );
        let first = ok(json!({"op": "canonical", "data": {"text": text, "dialect": "turtle"}}));
        let second = ok(json!({"op": "canonical", "data": {"text": text, "dialect": "turtle"}}));
        assert_eq!(
            first["id"],
            second["id"],
            "{} canonical identity drifted",
            path.display()
        );
        if text.contains("sa2a:expected \"ADMIT\"")
            || text.contains("sa2a:expectedDecision \"ADMIT\"")
        {
            assert!(
                text.contains("sa2a:authority \"NONE\""),
                "{} authority widened",
                path.display()
            );
            assert!(
                text.contains("sa2a:consequence \"EVIDENCE_ONLY\""),
                "{} consequence widened",
                path.display()
            );
            assert!(
                text.contains("sa2a:canonicalization \"RDFC-1.0\""),
                "{} canonicalization widened",
                path.display()
            );
        } else {
            assert!(
                text.contains("sa2a:expected \"REFUSE\""),
                "{} missing disposition",
                path.display()
            );
            assert!(
                text.contains("sa2a:violation"),
                "{} refusal lacks typed violation",
                path.display()
            );
        }
    }
}
