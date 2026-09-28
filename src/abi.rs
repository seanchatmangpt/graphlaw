//! JSON request/response ABI over every GraphLaw capability.
//!
//! This is the safe, natively testable core of the WebAssembly module in
//! `./wasm`: hosts (for example Elixir through Wasmex) exchange UTF-8 JSON and
//! never touch Rust types. A request is `{"op": "...", ...}`; a response is
//! `{"ok": true, ...}` or `{"ok": false, "error": {kind, engine, dialect,
//! message}}`. Every op delegates to PurRDF or Eyeron; nothing here parses RDF.
//!
//! Ops: `capabilities`, `sniff`, `parse`, `convert`, `canonical`, `sparql`,
//! `shacl`, `shex`, `n3`, `entail`, `datalog`, `hooks`, `law`.
//! A *data spec* is `{"text": "...", "dialect"?: "turtle", "hint"?: "ttl", "base"?: "..."}`;
//! without `dialect` the router sniffs the content.


use purrdf::{RdfDataset, SparqlEngine, SparqlRequest, SparqlResult, TermValue};
use serde_json::{Value, json};

use crate::dialect::{Dialect, Engine, Refusal, RefusalKind, check, sniff};
use crate::hooks::HookPack;
use crate::law::{LawError, LawState, Step};

/// ABI revision; bumped on any incompatible request/response change.
pub const ABI_VERSION: u32 = 1;

type Res<T> = Result<T, Refusal>;

fn bad(message: impl Into<String>) -> Refusal {
    Refusal { kind: RefusalKind::Unsupported, dialect: None, engine: None, message: message.into() }
}

/// Handle one request; always returns a JSON document.
pub fn call(request: &[u8]) -> Vec<u8> {
    let out = match serde_json::from_slice::<Value>(request) {
        Err(e) => Err(bad(format!("request is not JSON: {e}"))),
        Ok(v) => dispatch(&v),
    };
    let v = match out {
        Ok(mut v) => {
            if let Some(o) = v.as_object_mut() {
                o.insert("ok".into(), json!(true));
            }
            v
        }
        Err(r) => json!({"ok": false, "error": refusal_json(&r)}),
    };
    serde_json::to_vec(&v).expect("json serializes")
}

fn refusal_json(r: &Refusal) -> Value {
    json!({
        "kind": format!("{:?}", r.kind),
        "engine": r.engine.map(|e| format!("{e:?}")),
        "dialect": r.dialect.map(|d| format!("{d:?}")),
        "message": r.message,
    })
}

fn law_err(e: LawError) -> Refusal {
    match e {
        LawError::Refused(r) => r,
        LawError::NotAdmitted { violations } => Refusal {
            kind: RefusalKind::EngineRejected,
            dialect: Some(Dialect::Turtle),
            engine: Some(Engine::PurRdf),
            message: format!("SHACL admission refused: {violations} violation(s)"),
        },
    }
}

fn str_field<'a>(v: &'a Value, k: &str) -> Res<&'a str> {
    v.get(k).and_then(Value::as_str).ok_or_else(|| bad(format!("missing string field `{k}`")))
}

fn opt_str<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str)
}

/// Dialect names accepted on the wire.
pub fn dialect_by_name(name: &str) -> Res<Dialect> {
    Ok(match name.to_ascii_lowercase().as_str() {
        "turtle" | "ttl" => Dialect::Turtle,
        "trig" => Dialect::TriG,
        "ntriples" | "nt" => Dialect::NTriples,
        "nquads" | "nq" => Dialect::NQuads,
        "rdfxml" | "rdf" | "owl" => Dialect::RdfXml,
        "jsonld" => Dialect::JsonLd,
        "yamlld" => Dialect::YamlLd,
        "trix" => Dialect::TriX,
        "hextuples" | "hext" => Dialect::HexTuples,
        "n3" => Dialect::N3,
        "shexc" | "shex" => Dialect::ShExC,
        "shexj" => Dialect::ShExJ,
        "sparql" | "rq" => Dialect::Sparql,
        other => return Err(bad(format!("unknown dialect `{other}`"))),
    })
}

fn spec_dialect(spec: &Value) -> Res<Dialect> {
    match opt_str(spec, "dialect") {
        Some(name) => dialect_by_name(name),
        None => sniff(str_field(spec, "text")?.as_bytes(), opt_str(spec, "hint")),
    }
}

fn spec_of<'a>(v: &'a Value, k: &str) -> Res<&'a Value> {
    match v.get(k) {
        Some(s) if s.is_object() => Ok(s),
        Some(Value::String(_)) => Err(bad(format!("`{k}` must be a data spec object {{\"text\": ...}}"))),
        _ => Err(bad(format!("missing data spec `{k}`"))),
    }
}

fn state_of(spec: &Value) -> Res<LawState> {
    let dialect = spec_dialect(spec)?;
    LawState::parse(str_field(spec, "text")?.as_bytes(), dialect, opt_str(spec, "base"))
}

fn state_field(v: &Value, k: &str) -> Res<LawState> {
    state_of(spec_of(v, k)?)
}

fn term_json(t: &TermValue) -> Value {
    match t {
        TermValue::Iri(i) => json!({"type": "uri", "value": i}),
        TermValue::Blank { label, .. } => json!({"type": "bnode", "value": label}),
        TermValue::Literal { lexical_form, datatype, language, .. } => {
            let mut o = json!({"type": "literal", "value": lexical_form, "datatype": datatype});
            if let Some(l) = language {
                o["xml:lang"] = json!(l);
            }
            o
        }
        other => json!({"type": "triple", "value": format!("{other:?}")}),
    }
}

fn nquads_of(ds: &RdfDataset) -> Res<String> {
    let bytes = purrdf::serialize_dataset(ds, "application/n-quads", purrdf::SerializeGraph::Dataset)
        .map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
    String::from_utf8(bytes).map_err(|e| Refusal::engine(Dialect::NQuads, e))
}

fn dispatch(v: &Value) -> Res<Value> {
    match str_field(v, "op")? {
        "capabilities" => Ok(capabilities()),
        "sniff" => {
            let d = sniff(str_field(v, "text")?.as_bytes(), opt_str(v, "hint"))?;
            Ok(json!({"dialect": format!("{d:?}"), "engine": format!("{:?}", d.engine())}))
        }
        "parse" => op_parse(v),
        "convert" => op_convert(v),
        "canonical" => {
            let s = state_field(v, "data")?;
            let c = purrdf::try_canonicalize(s.dataset()).map_err(|e| bad(format!("{e:?}")))?;
            Ok(json!({"id": s.id(), "nquads": c.nquads, "quads": s.quad_count()}))
        }
        "sparql" => op_sparql(v),
        "shacl" => op_shacl(v),
        "shex" => op_shex(v),
        "n3" => {
            let out = eyeron::reason(str_field(v, "text")?).map_err(|e| Refusal::engine(Dialect::N3, e))?;
            Ok(json!({"derived": out}))
        }
        "entail" => op_entail(v),
        "datalog" => op_datalog(v),
        "hooks" => op_hooks(v),
        "law" => op_law(v),
        other => Err(bad(format!("unknown op `{other}`"))),
    }
}

fn capabilities() -> Value {
    json!({
        "abi": ABI_VERSION,
        "crate": env!("CARGO_PKG_VERSION"),
        "authorities": crate::BACKEND_AUTHORITIES.iter().map(|a| json!({
            "capability": a.capability, "authority": a.authority, "revision": a.revision,
        })).collect::<Vec<_>>(),
        "rdf_dialects": ["turtle", "trig", "ntriples", "nquads", "rdfxml", "jsonld", "yamlld", "trix", "hextuples"],
        "other_dialects": ["n3", "sparql", "shexc", "shexj"],
        "ops": ["capabilities", "sniff", "parse", "convert", "canonical", "sparql", "shacl", "shex",
                "n3", "entail", "datalog", "hooks", "law"],
    })
}

fn op_parse(v: &Value) -> Res<Value> {
    let text = str_field(v, "text")?;
    let dialect = spec_dialect(v)?;
    if dialect.media_type().is_some() {
        let s = LawState::parse(text.as_bytes(), dialect, opt_str(v, "base"))?;
        return Ok(json!({"dialect": format!("{dialect:?}"), "quads": s.quad_count(), "id": s.id()}));
    }
    check(text.as_bytes(), dialect, opt_str(v, "base"))?;
    Ok(json!({"dialect": format!("{dialect:?}"), "valid": true}))
}

fn op_convert(v: &Value) -> Res<Value> {
    let s = LawState::parse(
        str_field(v, "text")?.as_bytes(),
        spec_dialect(v)?,
        opt_str(v, "base"),
    )?;
    let to = dialect_by_name(str_field(v, "to")?)?;
    let media = to.media_type().ok_or_else(|| bad("`to` must be an RDF dialect"))?;
    let bytes = purrdf::serialize_dataset(s.dataset(), media, purrdf::SerializeGraph::Dataset)
        .map_err(|e| Refusal::engine(to, e))?;
    Ok(json!({"text": String::from_utf8(bytes).map_err(|e| Refusal::engine(to, e))?, "id": s.id()}))
}

fn op_sparql(v: &Value) -> Res<Value> {
    let data = state_field(v, "data")?;
    let query = str_field(v, "query")?;
    let result = crate::sparql::NativeSparqlEngine::new()
        .query(data.dataset(), SparqlRequest { query, base_iri: opt_str(v, "base"), substitutions: &[] })
        .map_err(|e| Refusal::engine(Dialect::Sparql, e))?;
    Ok(match result {
        SparqlResult::Solutions { variables, rows, .. } => json!({
            "kind": "solutions",
            "variables": variables,
            "rows": rows.iter().map(|r| r.iter().map(|c| c.as_ref().map_or(Value::Null, term_json)).collect::<Vec<_>>()).collect::<Vec<_>>(),
        }),
        SparqlResult::Graph(g) => json!({"kind": "graph", "nquads": nquads_of(&g)?, "quads": g.quad_count()}),
        SparqlResult::Boolean(b) => json!({"kind": "boolean", "value": b}),
    })
}

fn op_shacl(v: &Value) -> Res<Value> {
    let data = state_field(v, "data")?;
    let shapes = crate::shacl::engine::parse_shapes(str_field(v, "shapes")?, opt_str(v, "base"))
        .map_err(|e| Refusal::engine(Dialect::Turtle, e))?;
    let report = crate::shacl::engine::validate_dataset(data.dataset(), &shapes)
        .map_err(|e| Refusal::engine(Dialect::Turtle, e))?;
    Ok(json!({
        "conforms": report.conforms,
        "results": report.results.iter().map(|r| json!({
            "focus": r.focus_node.to_string(),
            "path": r.result_path.as_ref().map(ToString::to_string),
            "value": r.value.as_ref().map(ToString::to_string),
            "severity": format!("{:?}", r.severity),
            "component": r.source_constraint_component.to_string(),
            "shape": r.source_shape.to_string(),
            "message": r.message,
        })).collect::<Vec<_>>(),
    }))
}

fn op_shex(v: &Value) -> Res<Value> {
    let data = state_field(v, "data")?;
    let text = str_field(v, "schema")?;
    let base = opt_str(v, "base");
    let (dialect, schema) = match opt_str(v, "schema_dialect").unwrap_or("shexc") {
        "shexj" => (Dialect::ShExJ, crate::shex::parse_shexj(text, base)),
        _ => (Dialect::ShExC, crate::shex::parse_shexc(text, base)),
    };
    let schema = schema.map_err(|e| Refusal::engine(dialect, e))?;
    crate::shex::check_structure(&schema).map_err(|e| Refusal::engine(dialect, e))?;
    let map = crate::shex::validate_shape_map(
        &schema,
        data.dataset(),
        str_field(v, "map")?,
        base,
        &crate::shex::ValidationOptions::default(),
    )
    .map_err(|e| Refusal::engine(dialect, e))?;
    Ok(json!({
        "conforms": map.all_conformant(),
        "entries": map.entries.iter().map(|e| json!({
            "node": term_json(&e.node),
            "shape": format!("{:?}", e.shape),
            "status": format!("{:?}", e.status),
            "reason": e.reason,
        })).collect::<Vec<_>>(),
    }))
}

fn op_entail(v: &Value) -> Res<Value> {
    use crate::entailment::Materialization as M;
    let data = state_field(v, "data")?;
    let plan = match str_field(v, "regime")? {
        "simple" => M::Simple,
        "rdf" => M::Rdf,
        "rdfs" => M::Rdfs,
        "owl-rl" => M::OwlRl,
        "d" => M::D,
        other => return Err(bad(format!("unknown regime `{other}`"))),
    };
    let (closure, _report) = crate::entailment::materialize(data.dataset().as_ref(), plan)
        .map_err(|e| Refusal::engine(Dialect::Turtle, e))?;
    Ok(json!({
        "nquads": nquads_of(&closure)?,
        "added": closure.quad_count().saturating_sub(data.quad_count()),
    }))
}

fn clause_term(t: &Value) -> Res<crate::datalog::clause::ClauseTerm> {
    use crate::datalog::clause::ClauseTerm;
    let s = t.as_str().ok_or_else(|| bad("datalog terms are strings"))?;
    Ok(if let Some(var) = s.strip_prefix('?') {
        ClauseTerm::var(var)
    } else if s.starts_with('"') {
        ClauseTerm::literal(s)
    } else {
        ClauseTerm::iri(s)
    })
}

fn triple<'a>(t: &'a Value) -> Res<[&'a Value; 3]> {
    match t.as_array().map(Vec::as_slice) {
        Some([s, p, o]) => Ok([s, p, o]),
        _ => Err(bad("expected a [subject, predicate, object] triple")),
    }
}

fn atom(t: &Value) -> Res<crate::datalog::clause::ClauseAtom> {
    let [s, p, o] = triple(t)?;
    let p = p.as_str().ok_or_else(|| bad("datalog predicate must be an IRI string"))?;
    Ok(crate::datalog::clause::ClauseAtom::positive(clause_term(s)?, p, clause_term(o)?))
}

fn surface(t: &Value) -> Res<String> {
    let s = t.as_str().ok_or_else(|| bad("datalog fact terms are strings"))?;
    Ok(if s.starts_with('"') { s.to_string() } else { format!("<{s}>") })
}

fn unsurface(s: &str) -> String {
    s.strip_prefix('<').and_then(|s| s.strip_suffix('>')).unwrap_or(s).to_string()
}

fn op_datalog(v: &Value) -> Res<Value> {
    use crate::datalog::{clause::DlClause, seminaive::{compile, evaluate}, store::RelationStore};
    let dl = |e: &dyn std::fmt::Debug| Refusal {
        kind: RefusalKind::EngineRejected,
        dialect: None,
        engine: Some(Engine::PurRdf),
        message: format!("datalog: {e:?}"),
    };
    let mut rules = Vec::new();
    for r in v.get("rules").and_then(Value::as_array).ok_or_else(|| bad("missing `rules` array"))? {
        let body = r
            .get("body")
            .and_then(Value::as_array)
            .ok_or_else(|| bad("rule needs a `body` array"))?
            .iter()
            .map(atom)
            .collect::<Res<Vec<_>>>()?;
        rules.push(DlClause::datalog(atom(r.get("head").ok_or_else(|| bad("rule needs a `head`"))?)?, body));
    }
    let exe = compile(rules).map_err(|e| dl(&e))?;
    let mut facts = RelationStore::new();
    for f in v.get("facts").and_then(Value::as_array).ok_or_else(|| bad("missing `facts` array"))? {
        let [s, p, o] = triple(f)?;
        facts.insert(&surface(s)?, &surface(p)?, &surface(o)?, RelationStore::DEFAULT_GRAPH);
    }
    let ev = evaluate(&exe, facts).map_err(|e| dl(&e))?;
    let all = ev.facts().facts_sorted();
    Ok(json!({
        "count": all.len(),
        "facts": all.iter().map(|f| json!([unsurface(&f.subject), unsurface(&f.predicate), unsurface(&f.object)])).collect::<Vec<_>>(),
    }))
}

fn op_hooks(v: &Value) -> Res<Value> {
    let data = state_field(v, "data")?;
    let pack = HookPack::load(&state_field(v, "pack")?)?;
    let m = pack.materialize(&data)?;
    Ok(json!({
        "id": m.state.id(),
        "rounds": m.rounds,
        "quads": m.state.quad_count(),
        "nquads": nquads_of(m.state.dataset())?,
        "firings": m.firings.iter().map(|f| json!({
            "hook": f.hook, "round": f.round, "added": f.added,
            "row": f.row.iter().map(|(k, t)| json!([k, t])).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    }))
}

fn op_law(v: &Value) -> Res<Value> {
    let mut state = state_field(v, "data")?;
    let mut ids = vec![state.id().to_string()];
    let mut receipts = Vec::new();
    for step in v.get("steps").and_then(Value::as_array).ok_or_else(|| bad("missing `steps` array"))? {
        let pack;
        let shapes;
        let rules;
        let step_ref = match str_field(step, "step")? {
            "shacl" => {
                shapes = str_field(step, "shapes")?.to_string();
                Step::AdmitShacl { shapes_ttl: &shapes }
            }
            "n3" => {
                rules = str_field(step, "rules")?.to_string();
                Step::DeriveN3 { rules: &rules }
            }
            "rdfs" => Step::EntailRdfs,
            "owl-rl" => Step::EntailOwlRl,
            "hooks" => {
                pack = HookPack::load(&state_field(step, "pack")?)?;
                Step::Hooks { pack: &pack }
            }
            other => return Err(bad(format!("unknown step `{other}`"))),
        };
        let (child, r) = state.transition(&step_ref).map_err(law_err)?;
        receipts.push(json!({
            "step": r.step, "parent": r.parent, "child": r.child, "added": r.added,
            "authority": r.authority.authority, "revision": r.authority.revision,
        }));
        ids.push(child.id().to_string());
        state = child;
    }
    Ok(json!({"states": ids, "receipts": receipts, "nquads": nquads_of(state.dataset())?}))
}

/// Convenience for native callers and tests: parse a response.
pub fn call_json(request: &Value) -> Value {
    serde_json::from_slice(&call(request.to_string().as_bytes())).expect("response is JSON")
}

