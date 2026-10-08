//! JSON request/response ABI over every GraphLaw capability.
//!
//! This is the safe, natively testable core of the WebAssembly module in
//! `./wasm`: hosts (for example Elixir through Wasmex) exchange UTF-8 JSON and
//! never touch Rust types. A request is `{"op": "...", ...}`; a response is
//! `{"ok": true, ...}` or `{"ok": false, "error": {kind, engine, dialect,
//! message}}`. Every op delegates to PurRDF or Eyeron; nothing here parses RDF.
//!
//! Ops (14, in registry order; the machine-readable surface is [`crate::registry`]):
//! `capabilities`, `sniff`, `parse`, `convert`, `canonical`, `sparql`, `shacl`, `shex`,
//! `n3`, `entail`, `datalog`, `hooks`, `law`, `policy`.
//! `policy` admits a FOND policy as strong-cyclic against a planning problem.
//! `law` steps: `shacl`, `n3`, `rdfs`, `owl-rl`, `hooks`, and `plan`,
//! `record-receipts` (writes receipts produced so far into the state) and
//! `require-receipt` (`"step_name"`; refuses unless that step's receipt is recorded)
//! `require-signed-receipt` (`"step_name"`, `"trusted_keys"`: hex Ed25519 public keys;
//! refuses unless that step's receipt carries a valid attestation by a trusted key)
//! (`{"step":"plan","plan":{"actions":[{"name","pre","pre_not"?,"add","del"}],"goal","goal_not"?}}`,
//! N-Triples strings; `pre_not` per action and `goal_not` per plan list atoms that must be
//! ABSENT; one receipt per action, refused at the first unmet precondition or present
//! forbidden atom).
//!
//! Leases: a `law` request may carry `"signed_lease": {"lease": {id, holder, ceiling,
//! scope, expires_unix, issued_unix?}, "attestation": {key_id, payload_sha256,
//! signature}}` plus `"trusted_keys"` (hex) and optional `"max_skew_secs"` (default 60).
//! The signature is verified offline and expiry is judged by the module's own clock;
//! any `now_unix` in the request is ignored. An unsigned `"lease"` is refused
//! (`UnverifiedLeaseRefused`) unless the request sets `"unverified_lease": true`, which
//! also needs the caller's `now_unix` and proves nothing about who issued the lease.
//! A *data spec* is `{"text": "...", "dialect"?: "turtle", "hint"?: "ttl", "base"?: "..."}`;
//! without `dialect` the router sniffs the content.

use purrdf::{RdfDataset, SparqlEngine, SparqlRequest, SparqlResult, TermValue};
use serde_json::{Value, json};

use crate::attest::{Attestation, TrustedKeys};
use crate::dialect::{Dialect, Engine, Refusal, RefusalKind, check, sniff};
use crate::hooks::HookPack;
use crate::law::{
    Ceiling, DEFAULT_MAX_SKEW_SECS, LawError, LawState, Lease, Receipt, SignedLease, Step,
    SystemClock,
};

/// ABI revision; bumped on any incompatible request/response change.
pub const ABI_VERSION: u32 = 1;

/// Largest accepted request body, in bytes (16 MiB). Checked before parsing.
pub const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;
/// Deepest accepted JSON nesting (arrays/objects), checked before parsing.
pub const MAX_JSON_DEPTH: usize = 64;
/// Most actions accepted in one `plan` step.
pub const MAX_PLAN_ACTIONS: usize = 1_000;
/// Most N-Triples atoms (non-empty lines) in one plan `pre`/`add`/`del`/`goal` field.
pub const MAX_ATOMS_PER_FIELD: usize = 10_000;
/// Most entries accepted in one FOND policy.
pub const MAX_POLICY_ENTRIES: usize = 100_000;
/// Ceiling, in bytes, on host-visible buffers (`gl_alloc` buffers and `gl_call`
/// responses) the wasm module holds at once; `gl_alloc` returns null beyond it.
pub const MAX_OUTSTANDING_ALLOC_BYTES: usize = 256 * 1024 * 1024;

/// A refusal plus optional machine-readable `details` (`{"code": ...}`).
#[derive(Debug, Clone)]
pub struct Fail {
    refusal: Refusal,
    details: Option<Value>,
}

impl From<Refusal> for Fail {
    fn from(refusal: Refusal) -> Self {
        Fail {
            refusal,
            details: None,
        }
    }
}

type Res<T> = Result<T, Fail>;

fn limit(name: &str, observed: usize, max: usize) -> Fail {
    Fail {
        refusal: Refusal {
            kind: RefusalKind::ResourceLimit,
            dialect: None,
            engine: None,
            message: format!("resource limit `{name}` exceeded: {observed} > {max}"),
        },
        details: Some(json!({
            "code": "ResourceLimit", "limit": name, "observed": observed, "max": max,
        })),
    }
}

fn check_limit(name: &str, observed: usize, max: usize) -> Res<()> {
    if observed > max {
        return Err(limit(name, observed, max));
    }
    Ok(())
}

/// Maximum bracket nesting of a JSON text (string-aware, no allocation).
fn json_depth(b: &[u8]) -> usize {
    let (mut depth, mut max, mut in_str, mut esc) = (0usize, 0usize, false, false);
    for &c in b {
        if in_str {
            if esc {
                esc = false;
            } else if c == b'\\' {
                esc = true;
            } else if c == b'"' {
                in_str = false;
            }
            continue;
        }
        match c {
            b'"' => in_str = true,
            b'[' | b'{' => {
                depth += 1;
                max = max.max(depth);
            }
            b']' | b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    max
}

/// Response for a request that exceeds `MAX_REQUEST_BYTES` (used by the wasm shim).
pub fn limit_response(name: &str, observed: usize, max: usize) -> Vec<u8> {
    respond(Err(limit(name, observed, max)))
}

/// Response for a call whose request buffer does not exist (`gl_alloc` refused).
pub fn missing_buffer_response() -> Vec<u8> {
    respond(Err(bad(
        "request buffer missing: gl_alloc refused or was never called",
    )))
}

fn bad_refusal(message: impl Into<String>) -> Refusal {
    Refusal {
        kind: RefusalKind::Unsupported,
        dialect: None,
        engine: None,
        message: message.into(),
    }
}

fn bad(message: impl Into<String>) -> Fail {
    bad_refusal(message).into()
}

/// Handle one request; always returns a JSON document.
pub fn call(request: &[u8]) -> Vec<u8> {
    let out = check_limit("request_bytes", request.len(), MAX_REQUEST_BYTES)
        .and_then(|()| check_limit("json_depth", json_depth(request), MAX_JSON_DEPTH))
        .and_then(|()| {
            serde_json::from_slice::<Value>(request)
                .map_err(|e| bad(format!("request is not JSON: {e}")))
        })
        .and_then(|v| dispatch(&v));
    respond(out)
}

fn respond(out: Res<Value>) -> Vec<u8> {
    let v = match out {
        Ok(mut v) => {
            if let Some(o) = v.as_object_mut() {
                o.insert("ok".into(), json!(true));
            }
            v
        }
        Err(f) => json!({"ok": false, "error": refusal_json(&f)}),
    };
    serde_json::to_vec(&v).expect("json serializes")
}

fn refusal_json(f: &Fail) -> Value {
    let r = &f.refusal;
    let mut o = json!({
        "kind": format!("{:?}", r.kind),
        "engine": r.engine.map(|e| format!("{e:?}")),
        "dialect": r.dialect.map(|d| format!("{d:?}")),
        "message": r.message,
    });
    if let Some(d) = &f.details {
        o["details"] = d.clone();
    }
    o
}

/// `(name, observed, max)` from a `resource limit `name` exceeded: a > b` message.
fn parse_limit_message(message: &str) -> Option<(String, usize, usize)> {
    let rest = message.strip_prefix("resource limit `")?;
    let (name, rest) = rest.split_once("` exceeded: ")?;
    let mut nums = rest.split_whitespace();
    let observed = nums.next()?.parse().ok()?;
    if nums.next()? != ">" {
        return None;
    }
    let max = nums.next()?.parse().ok()?;
    Some((name.to_string(), observed, max))
}

fn law_err(e: LawError) -> Fail {
    let (refusal, details) = match e {
        LawError::Refused(r) => {
            let d = if r.kind == RefusalKind::ResourceLimit {
                // The refusal message is `resource limit `<name>` exceeded: <observed> > <max> ...`.
                match parse_limit_message(&r.message) {
                    Some((name, observed, max)) => json!({"code": "ResourceLimit", "limit": name,
                           "observed": observed, "max": max}),
                    None => json!({"code": "ResourceLimit", "limit": "n3_iterations",
                           "max": crate::law::N3_MAX_ITERATIONS}),
                }
            } else {
                json!({"code": "Refused", "kind": format!("{:?}", r.kind)})
            };
            (r, d)
        }
        LawError::PlanRefused {
            index,
            action,
            missing,
            violated_absent,
        } => {
            let mut message = format!(
                "plan refused at step {index} (`{action}`): unmet {}",
                missing.join(" ")
            );
            if !violated_absent.is_empty() {
                message = format!(
                    "plan refused at step {index} (`{action}`): forbidden triple(s) present {}",
                    violated_absent.join(" ")
                );
                if !missing.is_empty() {
                    message.push_str(&format!("; unmet {}", missing.join(" ")));
                }
            }
            (
                Refusal {
                    kind: RefusalKind::EngineRejected,
                    dialect: Some(Dialect::NTriples),
                    engine: Some(Engine::PurRdf),
                    message,
                },
                json!({"code": "PlanRefused", "index": index, "action": action,
                       "unmet": missing, "violated_absent": violated_absent}),
            )
        }
        LawError::ReceiptRequired { step } => (
            Refusal {
                kind: RefusalKind::EngineRejected,
                dialect: None,
                engine: Some(Engine::PurRdf),
                message: format!("receipt required: no recorded receipt for step `{step}`"),
            },
            json!({"code": "ReceiptRequired", "step": step}),
        ),
        LawError::ReceiptRefused { step, reason } => (
            Refusal {
                kind: RefusalKind::EngineRejected,
                dialect: None,
                engine: None,
                message: format!("receipt for step `{step}` refused: {}", reason.as_str()),
            },
            json!({"code": "ReceiptRefused", "step": step, "reason": reason.as_str()}),
        ),
        e @ LawError::LeaseRefused { .. } => {
            let d = match &e {
                LawError::LeaseRefused {
                    lease_id,
                    step,
                    reason,
                } => json!({
                    "code": "LeaseRefused",
                    "reason": reason.as_str(),
                    "lease_id": lease_id,
                    "step": step,
                }),
                _ => unreachable!("matched LeaseRefused above"),
            };
            (
                Refusal {
                    kind: RefusalKind::EngineRejected,
                    dialect: None,
                    engine: None,
                    message: e.to_string(),
                },
                d,
            )
        }
        LawError::NotAdmitted {
            violations,
            results,
        } => (
            Refusal {
                kind: RefusalKind::EngineRejected,
                dialect: Some(Dialect::Turtle),
                engine: Some(Engine::PurRdf),
                message: format!("SHACL admission refused: {violations} violation(s)"),
            },
            json!({
                "code": "NotAdmitted",
                "violations": results.iter().map(|v| json!({
                    "focus": v.focus, "path": v.path, "component": v.component,
                    "message": v.message, "severity": v.severity,
                })).collect::<Vec<_>>(),
            }),
        ),
    };
    Fail {
        refusal,
        details: Some(details),
    }
}

fn str_field<'a>(v: &'a Value, k: &str) -> Res<&'a str> {
    v.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| bad(format!("missing string field `{k}`")))
}

fn opt_str<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str)
}

/// Dialect names accepted on the wire.
pub fn dialect_by_name(name: &str) -> Result<Dialect, Refusal> {
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
        other => return Err(bad_refusal(format!("unknown dialect `{other}`"))),
    })
}

fn spec_dialect(spec: &Value) -> Res<Dialect> {
    match opt_str(spec, "dialect") {
        Some(name) => Ok(dialect_by_name(name)?),
        None => Ok(sniff(
            str_field(spec, "text")?.as_bytes(),
            opt_str(spec, "hint"),
        )?),
    }
}

fn spec_of<'a>(v: &'a Value, k: &str) -> Res<&'a Value> {
    match v.get(k) {
        Some(s) if s.is_object() => Ok(s),
        Some(Value::String(_)) => Err(bad(format!(
            "`{k}` must be a data spec object {{\"text\": ...}}"
        ))),
        _ => Err(bad(format!("missing data spec `{k}`"))),
    }
}

fn state_of(spec: &Value) -> Res<LawState> {
    let dialect = spec_dialect(spec)?;
    Ok(LawState::parse(
        str_field(spec, "text")?.as_bytes(),
        dialect,
        opt_str(spec, "base"),
    )?)
}

fn state_field(v: &Value, k: &str) -> Res<LawState> {
    state_of(spec_of(v, k)?)
}

fn term_json(t: &TermValue) -> Value {
    match t {
        TermValue::Iri(i) => json!({"type": "uri", "value": i}),
        TermValue::Blank { label, .. } => json!({"type": "bnode", "value": label}),
        TermValue::Literal {
            lexical_form,
            datatype,
            language,
            ..
        } => {
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
    let bytes =
        purrdf::serialize_dataset(ds, "application/n-quads", purrdf::SerializeGraph::Dataset)
            .map_err(|e| Refusal::engine(Dialect::NQuads, e))?;
    Ok(String::from_utf8(bytes).map_err(|e| Refusal::engine(Dialect::NQuads, e))?)
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
            let out =
                crate::law::reason_n3_bounded(str_field(v, "text")?).map_err(|e| match e {
                    crate::law::N3Error::Refused(r) => Fail::from(r),
                    crate::law::N3Error::Limit {
                        limit: name,
                        observed,
                        max,
                        ..
                    } => limit(name, observed, max),
                })?;
            Ok(json!({"derived": out}))
        }
        "entail" => op_entail(v),
        "datalog" => op_datalog(v),
        "hooks" => op_hooks(v),
        "law" => op_law(v),
        "policy" => op_policy(v),
        other => Err(bad(format!("unknown op `{other}`"))),
    }
}

fn capabilities() -> Value {
    json!({
        "abi": ABI_VERSION,
        "abi_version": ABI_VERSION,
        "crate": env!("CARGO_PKG_VERSION"),
        "authorities": crate::BACKEND_AUTHORITIES.iter().map(|a| json!({
            "capability": a.capability, "authority": a.authority, "revision": a.revision,
        })).collect::<Vec<_>>(),
        "rdf_dialects": crate::registry::rdf_dialect_names(),
        "other_dialects": crate::registry::other_dialect_names(),
        "ops": crate::registry::op_names(),
        "registry_schema": crate::registry::REGISTRY_SCHEMA,
        "registry_sha256": crate::registry::registry_sha256(),
        "surface_sha256": crate::registry::surface_sha256(),
    })
}

fn op_parse(v: &Value) -> Res<Value> {
    let text = str_field(v, "text")?;
    let dialect = spec_dialect(v)?;
    if dialect.media_type().is_some() {
        let s = LawState::parse(text.as_bytes(), dialect, opt_str(v, "base"))?;
        return Ok(
            json!({"dialect": format!("{dialect:?}"), "quads": s.quad_count(), "id": s.id()}),
        );
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
    let media = to
        .media_type()
        .ok_or_else(|| bad("`to` must be an RDF dialect"))?;
    let bytes = purrdf::serialize_dataset(s.dataset(), media, purrdf::SerializeGraph::Dataset)
        .map_err(|e| Refusal::engine(to, e))?;
    Ok(json!({"text": String::from_utf8(bytes).map_err(|e| Refusal::engine(to, e))?, "id": s.id()}))
}

fn op_sparql(v: &Value) -> Res<Value> {
    let data = state_field(v, "data")?;
    let query = str_field(v, "query")?;
    let result = crate::sparql::NativeSparqlEngine::new()
        .query(
            data.dataset(),
            SparqlRequest {
                query,
                base_iri: opt_str(v, "base"),
                substitutions: &[],
            },
        )
        .map_err(|e| Refusal::engine(Dialect::Sparql, e))?;
    Ok(match result {
        SparqlResult::Solutions {
            variables, rows, ..
        } => json!({
            "kind": "solutions",
            "variables": variables,
            "rows": rows.iter().map(|r| r.iter().map(|c| c.as_ref().map_or(Value::Null, term_json)).collect::<Vec<_>>()).collect::<Vec<_>>(),
        }),
        SparqlResult::Graph(g) => {
            json!({"kind": "graph", "nquads": nquads_of(&g)?, "quads": g.quad_count()})
        }
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
        "shexc" => (Dialect::ShExC, crate::shex::parse_shexc(text, base)),
        "shexj" => (Dialect::ShExJ, crate::shex::parse_shexj(text, base)),
        other => return Err(bad(format!("unknown schema_dialect `{other}`"))),
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

fn triple(t: &Value) -> Res<[&Value; 3]> {
    match t.as_array().map(Vec::as_slice) {
        Some([s, p, o]) => Ok([s, p, o]),
        _ => Err(bad("expected a [subject, predicate, object] triple")),
    }
}

fn atom(t: &Value) -> Res<crate::datalog::clause::ClauseAtom> {
    let [s, p, o] = triple(t)?;
    let p = p
        .as_str()
        .ok_or_else(|| bad("datalog predicate must be an IRI string"))?;
    Ok(crate::datalog::clause::ClauseAtom::positive(
        clause_term(s)?,
        p,
        clause_term(o)?,
    ))
}

fn surface(t: &Value) -> Res<String> {
    let s = t
        .as_str()
        .ok_or_else(|| bad("datalog fact terms are strings"))?;
    Ok(if s.starts_with('"') {
        s.to_string()
    } else {
        format!("<{s}>")
    })
}

fn unsurface(s: &str) -> String {
    s.strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(s)
        .to_string()
}

fn op_datalog(v: &Value) -> Res<Value> {
    use crate::datalog::{
        clause::DlClause,
        seminaive::{compile, evaluate},
        store::RelationStore,
    };
    let dl = |e: &dyn std::fmt::Debug| Refusal {
        kind: RefusalKind::EngineRejected,
        dialect: None,
        engine: Some(Engine::PurRdf),
        message: format!("datalog: {e:?}"),
    };
    let mut rules = Vec::new();
    for r in v
        .get("rules")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing `rules` array"))?
    {
        let body = r
            .get("body")
            .and_then(Value::as_array)
            .ok_or_else(|| bad("rule needs a `body` array"))?
            .iter()
            .map(atom)
            .collect::<Res<Vec<_>>>()?;
        rules.push(DlClause::datalog(
            atom(r.get("head").ok_or_else(|| bad("rule needs a `head`"))?)?,
            body,
        ));
    }
    let exe = compile(rules).map_err(|e| dl(&e))?;
    let mut facts = RelationStore::new();
    for f in v
        .get("facts")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing `facts` array"))?
    {
        let [s, p, o] = triple(f)?;
        facts.insert(
            &surface(s)?,
            &surface(p)?,
            &surface(o)?,
            RelationStore::DEFAULT_GRAPH,
        );
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

fn plan_field(step: &Value) -> Res<crate::plan::Plan> {
    let p = step.get("plan").ok_or_else(|| bad("missing `plan`"))?;
    let text = |o: &Value, k: &str| -> Res<String> {
        match o.get(k) {
            None | Some(Value::Null) => Ok(String::new()),
            Some(Value::String(t)) => {
                let atoms = t.lines().filter(|l| !l.trim().is_empty()).count();
                check_limit("atoms_per_field", atoms, MAX_ATOMS_PER_FIELD)?;
                Ok(t.clone())
            }
            Some(_) => Err(bad(format!("`{k}` must be an N-Triples string"))),
        }
    };
    let raw = p
        .get("actions")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("plan needs an `actions` array"))?;
    check_limit("plan_actions", raw.len(), MAX_PLAN_ACTIONS)?;
    let actions = raw
        .iter()
        .map(|a| {
            Ok(crate::plan::Action {
                name: text(a, "name")?,
                pre: text(a, "pre")?,
                pre_not: text(a, "pre_not")?,
                add: text(a, "add")?,
                del: text(a, "del")?,
            })
        })
        .collect::<Res<Vec<_>>>()?;
    Ok(crate::plan::Plan {
        actions,
        goal: text(p, "goal")?,
        goal_not: text(p, "goal_not")?,
    })
}

fn lease_obj(l: &Value) -> Res<Lease> {
    let ceiling = Ceiling::parse(str_field(l, "ceiling")?)
        .ok_or_else(|| bad("lease `ceiling` must be observe|select|construct"))?;
    let scope = l
        .get("scope")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("lease `scope` must be an array of step names"))?
        .iter()
        .map(|s| {
            s.as_str()
                .map(str::to_string)
                .ok_or_else(|| bad("lease `scope` entries are strings"))
        })
        .collect::<Res<Vec<_>>>()?;
    let expires_unix = l
        .get("expires_unix")
        .and_then(Value::as_u64)
        .ok_or_else(|| bad("lease `expires_unix` must be an unsigned integer"))?;
    let issued_unix = match l.get("issued_unix") {
        None | Some(Value::Null) => 0,
        Some(n) => n
            .as_u64()
            .ok_or_else(|| bad("lease `issued_unix` must be an unsigned integer"))?,
    };
    Ok(Lease {
        id: str_field(l, "id")?.to_string(),
        holder: str_field(l, "holder")?.to_string(),
        ceiling,
        scope,
        expires_unix,
        issued_unix,
    })
}

/// How a `law` request's steps are authorized.
enum LeaseAuth {
    /// No lease: steps run unleased.
    None,
    /// `"lease"` + `"now_unix"` + `"unverified_lease": true`: unsigned lease,
    /// caller-chosen time. Proves nothing about authority.
    Unverified(Lease, u64),
    /// `"signed_lease"` + `"trusted_keys"`: Ed25519-verified, expiry judged by
    /// the module's own clock (the request cannot supply a time).
    Signed {
        signed: SignedLease,
        trusted: TrustedKeys,
        max_skew_secs: u64,
    },
}

impl LeaseAuth {
    fn id(&self) -> Option<String> {
        match self {
            LeaseAuth::None => None,
            LeaseAuth::Unverified(l, _) => Some(l.id.clone()),
            LeaseAuth::Signed { signed, .. } => Some(signed.lease.id.clone()),
        }
    }

    fn authorize(&self, step: &str, required: Ceiling) -> Result<(), LawError> {
        match self {
            LeaseAuth::None => Ok(()),
            LeaseAuth::Unverified(l, now) => l.authorize(step, required, *now),
            LeaseAuth::Signed {
                signed,
                trusted,
                max_skew_secs,
            } => signed.authorize(step, required, trusted, &SystemClock, *max_skew_secs),
        }
    }

    fn transition(
        &self,
        state: &LawState,
        step: &Step<'_>,
    ) -> Result<(LawState, Receipt), LawError> {
        match self {
            LeaseAuth::None => state.transition(step),
            LeaseAuth::Unverified(l, now) => state.transition_leased_unverified(l, step, *now),
            LeaseAuth::Signed {
                signed,
                trusted,
                max_skew_secs,
            } => state.transition_authorized(signed, trusted, &SystemClock, *max_skew_secs, step),
        }
    }
}

fn trusted_keys_of(v: &Value) -> Res<TrustedKeys> {
    let arr = v
        .get("trusted_keys")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("`trusted_keys` must be an array of hex Ed25519 public keys"))?;
    let hexes = arr
        .iter()
        .map(|k| {
            k.as_str()
                .ok_or_else(|| bad("`trusted_keys` entries are hex strings"))
        })
        .collect::<Res<Vec<_>>>()?;
    TrustedKeys::from_hex(hexes).map_err(|e| bad(format!("`trusted_keys`: {e}")))
}

fn lease_auth_of(v: &Value) -> Res<LeaseAuth> {
    match (v.get("signed_lease"), v.get("lease")) {
        (None, None) => Ok(LeaseAuth::None),
        (Some(_), Some(_)) => Err(bad("give `signed_lease` or `lease`, not both")),
        (Some(sl), None) => {
            let a = sl
                .get("attestation")
                .ok_or_else(|| bad("`signed_lease` needs an `attestation`"))?;
            let attestation = Attestation::from_json(
                &json!({
                    "key_id": str_field(a, "key_id")?,
                    "payload_sha256": str_field(a, "payload_sha256")?,
                    "signature": str_field(a, "signature")?,
                })
                .to_string(),
            )
            .map_err(|e| bad(format!("`attestation`: {e}")))?;
            let max_skew_secs = match v.get("max_skew_secs") {
                None | Some(Value::Null) => DEFAULT_MAX_SKEW_SECS,
                Some(n) => n
                    .as_u64()
                    .ok_or_else(|| bad("`max_skew_secs` must be an unsigned integer"))?,
            };
            Ok(LeaseAuth::Signed {
                signed: SignedLease {
                    lease: lease_obj(
                        sl.get("lease")
                            .ok_or_else(|| bad("`signed_lease` needs a `lease`"))?,
                    )?,
                    attestation,
                },
                trusted: trusted_keys_of(v)?,
                max_skew_secs,
            })
        }
        (None, Some(l)) => {
            if v.get("unverified_lease") != Some(&Value::Bool(true)) {
                return Err(Fail {
                    refusal: bad_refusal(
                        "an unsigned `lease` is refused: supply `signed_lease` + `trusted_keys`, \
                         or set `\"unverified_lease\": true` to accept a lease no one signed",
                    ),
                    details: Some(json!({"code": "UnverifiedLeaseRefused"})),
                });
            }
            let now = v
                .get("now_unix")
                .and_then(Value::as_u64)
                .ok_or_else(|| bad("an unverified `lease` requires integer `now_unix`"))?;
            Ok(LeaseAuth::Unverified(lease_obj(l)?, now))
        }
    }
}

/// Parse a plan document (`{"actions":[{name,pre,add,del}],"goal"}`) with the
/// ABI's size limits. Used by the `graphlaw-verify` binary.
pub fn plan_from_json(plan: &Value) -> Result<crate::plan::Plan, String> {
    plan_field(&json!({ "plan": plan })).map_err(|f| f.refusal.message)
}

/// `{"op":"policy","problem":<PlanningProblem JSON|string>,"policy":<UniversalPlan|entries JSON|string>}`:
/// independent strong-cyclic admission of a FOND policy (see `policy`).
fn op_policy(v: &Value) -> Res<Value> {
    let text = |k: &str| -> Res<String> {
        match v.get(k) {
            Some(Value::String(t)) => Ok(t.clone()),
            Some(o) if o.is_object() || o.is_array() => Ok(o.to_string()),
            _ => Err(bad(format!("missing `{k}` (JSON object or string)"))),
        }
    };
    let count = |p: &Value| match p {
        Value::Array(a) => a.len(),
        o @ Value::Object(_) => o
            .get("policy")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        _ => 0,
    };
    let entries = match v.get("policy") {
        Some(Value::String(t)) => serde_json::from_str::<Value>(t).map_or(0, |p| count(&p)),
        Some(p) => count(p),
        None => 0,
    };
    check_limit("policy_entries", entries, MAX_POLICY_ENTRIES)?;
    let admitted = crate::policy::admit(&text("problem")?, &text("policy")?).map_err(|r| Fail {
        refusal: Refusal {
            kind: RefusalKind::EngineRejected,
            dialect: None,
            engine: None,
            message: format!(
                "policy refused ({}) at state `{}` action `{}`: {}",
                r.kind.as_str(),
                r.state,
                r.action,
                r.message
            ),
        },
        details: Some(json!({
            "code": "PolicyRefused", "policy_kind": r.kind.as_str(),
            "state": r.state, "action": r.action,
        })),
    })?;
    Ok(json!({
        "initial_states": admitted.initial_states,
        "reachable": admitted.reachable,
        "goal_states": admitted.goal_states,
        "entries": admitted.entries.iter().map(|(s, a)| json!([s, a])).collect::<Vec<_>>(),
        "ntriples": admitted.to_ntriples(),
    }))
}

fn op_law(v: &Value) -> Res<Value> {
    let lease = lease_auth_of(v)?;
    let lease_id = lease.id();
    let mut state = state_field(v, "data")?;
    let mut ids = vec![state.id().to_string()];
    let mut receipts = Vec::new();
    let mut produced: Vec<crate::law::Receipt> = Vec::new();
    let mut recorded = 0usize;
    for step in v
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing `steps` array"))?
    {
        if str_field(step, "step")? == "plan" {
            let plan = plan_field(step)?;
            lease
                .authorize("admit:plan", Ceiling::Select)
                .map_err(law_err)?;
            let admitted = plan.admit(&state).map_err(law_err)?;
            for (index, r) in admitted.receipts.iter().enumerate() {
                let mut rj = json!({
                    "step": r.step, "parent": r.parent, "child": r.child, "added": r.added,
                    "authority": r.authority.authority, "revision": r.authority.revision,
                    "plan_sha256": admitted.plan_digest, "index": index,
                });
                if let Some(id) = &lease_id {
                    rj["lease_id"] = json!(id);
                }
                receipts.push(rj);
                ids.push(r.child.clone());
                let mut r = r.clone();
                r.lease_id = lease_id.clone();
                produced.push(r);
            }
            state = admitted.state;
            continue;
        }
        if str_field(step, "step")? == "record-receipts" {
            for r in &produced[recorded..] {
                state = crate::receipt::record(&state, r).map_err(law_err)?;
            }
            recorded = produced.len();
            ids.push(state.id().to_string());
            continue;
        }
        let pack;
        let shapes;
        let rules;
        let step_name;
        let step_trusted;
        let step_ref = match str_field(step, "step")? {
            "shacl" => {
                shapes = str_field(step, "shapes")?.to_string();
                Step::AdmitShacl {
                    shapes_ttl: &shapes,
                }
            }
            "n3" => {
                rules = str_field(step, "rules")?.to_string();
                Step::DeriveN3 { rules: &rules }
            }
            "rdfs" => Step::EntailRdfs,
            "owl-rl" => Step::EntailOwlRl,
            "require-receipt" => {
                step_name = str_field(step, "step_name")?.to_string();
                Step::RequireReceipt { step: &step_name }
            }
            "require-signed-receipt" => {
                step_name = str_field(step, "step_name")?.to_string();
                step_trusted = trusted_keys_of(step)?;
                Step::RequireSignedReceipt {
                    step: &step_name,
                    trusted: &step_trusted,
                }
            }
            "hooks" => {
                pack = HookPack::load(&state_field(step, "pack")?)?;
                Step::Hooks { pack: &pack }
            }
            other => return Err(bad(format!("unknown step `{other}`"))),
        };
        let (child, r) = lease.transition(&state, &step_ref).map_err(law_err)?;
        let mut rj = json!({
            "step": r.step, "parent": r.parent, "child": r.child, "added": r.added,
            "authority": r.authority.authority, "revision": r.authority.revision,
        });
        if let Some(id) = &r.lease_id {
            rj["lease_id"] = json!(id);
        }
        receipts.push(rj);
        ids.push(child.id().to_string());
        produced.push(r);
        state = child;
    }
    Ok(json!({"states": ids, "receipts": receipts, "nquads": nquads_of(state.dataset())?}))
}

/// Convenience for native callers and tests: parse a response.
pub fn call_json(request: &Value) -> Value {
    serde_json::from_slice(&call(request.to_string().as_bytes())).expect("response is JSON")
}
