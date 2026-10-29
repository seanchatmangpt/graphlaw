//! JSON request/response ABI over every GraphLaw capability.
//!
//! This is the safe, natively testable core of the WebAssembly module in
//! `./wasm`: hosts (for example Elixir through Wasmex) exchange UTF-8 JSON and
//! never touch Rust types. A request is `{"op": "...", ...}`; a response is
//! `{"ok": true, ...}` or `{"ok": false, "error": {kind, engine, dialect,
//! message}}`. Every op delegates to PurRDF (incl. purrdf::n3); nothing here parses RDF.
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
//! scope, expires_unix, issued_unix?}, "attestation": {key_id, payload_blake3,
//! signature}}` plus `"trusted_keys"` (hex) and optional `"max_skew_secs"` (default 60).
//! The signature is verified offline and expiry is judged by the module's own clock;
//! any `now_unix` in the request is ignored. An unsigned `"lease"` is refused
//! (`UnverifiedLeaseRefused`) unless the request sets `"unverified_lease": true`, which
//! also needs the caller's `now_unix` and proves nothing about who issued the lease.
//! A *data spec* is `{"text": "...", "dialect"?: "turtle", "hint"?: "ttl", "base"?: "..."}`;
//! without `dialect` the router sniffs the content.

use purrdf::{RdfDataset, TermValue};
use serde_json::{Value, json};

use crate::dialect::{Dialect, Refusal, RefusalKind, RefusalMessage, sniff};
use crate::law::LawState;

mod lease;
mod ops;

pub use ops::plan_from_json;

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
/// Most steps accepted in one `law` request.
pub const MAX_LAW_STEPS: usize = 1_000;
/// Most datalog rules accepted in one `datalog` request.
pub const MAX_DATALOG_RULES: usize = 10_000;
/// Most datalog facts accepted in one `datalog` request.
pub const MAX_DATALOG_FACTS: usize = 100_000;
/// Ceiling, in bytes, on host-visible buffers (`gl_alloc` buffers and `gl_call`
/// responses) the wasm module holds at once; `gl_alloc` returns null beyond it.
///
/// 16 MiB per GAP-1 (docs/wasm-duckdb-op15-design.md): under the smalt 64 MiB
/// sandbox wall, alloc-success-then-OOM-trap is impossible (DuckDB's 32 MiB
/// compile-time pragma leaves headroom). The value is target-uniform so the
/// hashed registry document (and therefore `registry_blake3`) is identical for
/// native and wasm builds — op_examples' native-vs-wasm byte court depends on
/// it. Enforcement lives in the wasm shell (`wasm/src/ffi.rs`); native callers
/// do not allocate through `gl_alloc`.
pub const MAX_OUTSTANDING_ALLOC_BYTES: usize = 16 * 1024 * 1024;

/// A refusal plus optional machine-readable `details` (`{"code": ...}`).
#[derive(Debug, Clone)]
pub struct Fail {
    pub(in crate::abi) refusal: Refusal,
    pub(in crate::abi) details: Option<Value>,
}

impl From<Refusal> for Fail {
    fn from(refusal: Refusal) -> Self {
        Fail {
            refusal,
            details: None,
        }
    }
}

/// Build a `Fail` from a refusal plus machine-readable `details`
/// (crate-internal; used by `duckdb_op`).
pub(crate) fn fail_with_details(refusal: Refusal, details: Value) -> Fail {
    Fail {
        refusal,
        details: Some(details),
    }
}

type Res<T> = Result<T, Fail>;

pub(crate) fn limit(name: &str, observed: usize, max: usize) -> Fail {
    Fail {
        refusal: Refusal {
            kind: RefusalKind::ResourceLimit,
            dialect: None,
            engine: None,
            message: format!("resource limit `{name}` exceeded: {observed} > {max}").into(),
        },
        details: Some(json!({
            "code": "ResourceLimit", "limit": name, "observed": observed, "max": max,
        })),
    }
}

pub(crate) fn check_limit(name: &str, observed: usize, max: usize) -> Res<()> {
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
    vec![1] /* ~ changed by cargo-mutants ~ */
}

/// Response for a call whose request buffer does not exist (`gl_alloc` refused).
pub fn missing_buffer_response() -> Vec<u8> {
    respond(Err(bad(
        "request buffer missing: gl_alloc refused or was never called",
    )))
}

/// Response when `gl_call` is given a pointer this module does not own, or a
/// length that is not the one paired with that allocation. The buffer is left
/// untouched, so a following `gl_free` with the original length can still
/// reclaim it. A second free is ignored by the shell rather than rebuilt.
pub fn unknown_buffer_response() -> Vec<u8> {
    respond(Err(bad(
        "request buffer is not a live allocation from gl_alloc",
    )))
}

fn bad_refusal(message: impl Into<String>) -> Refusal {
    Refusal {
        kind: RefusalKind::Unsupported,
        dialect: None,
        engine: None,
        message: RefusalMessage::Dynamic(message.into()),
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
    // `Value` from `json!` / `serde_json::Value` always serializes. With the
    // wasm profile's `panic = "abort"`, a failure here is a guest trap, not a
    // JSON refusal — it is a bug in the response builder, not a host input.
    serde_json::to_vec(&v).expect("json serializes")
}

fn refusal_json(f: &Fail) -> Value {
    let r = &f.refusal;
    let mut o = json!({
        "kind": format!("{:?}", r.kind),
        "engine": r.engine.map(|e| format!("{e:?}")),
        "dialect": r.dialect.map(|d| format!("{d:?}")),
        "message": r.message.as_str(),
    });
    if let Some(d) = &f.details {
        o["details"] = d.clone();
    }
    o
}

pub(crate) fn str_field<'a>(v: &'a Value, k: &str) -> Res<&'a str> {
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
        "sql" => Dialect::SqlRdf,
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

pub(crate) fn term_json(t: &TermValue) -> Value {
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
        "capabilities" => Ok(ops::meta::capabilities()),
        "sniff" => ops::meta::op_sniff(v),
        "parse" => ops::parse_convert::op_parse(v),
        "convert" => ops::parse_convert::op_convert(v),
        "canonical" => ops::meta::op_canonical(v),
        "sparql" => ops::query_engines::op_sparql(v),
        "shacl" => ops::query_engines::op_shacl(v),
        "shex" => ops::query_engines::op_shex(v),
        "n3" => ops::query_engines::op_n3(v),
        "entail" => ops::query_engines::op_entail(v),
        "datalog" => ops::query_engines::op_datalog(v),
        "hooks" => ops::governance::op_hooks(v),
        "law" => ops::governance::op_law(v),
        "policy" => ops::governance::op_policy(v),
        "duckdb" => crate::duckdb_op::dispatch(v),
        other => Err(bad(format!("unknown op `{other}`"))),
    }
}

/// Convenience for native callers and tests: parse a response.
pub fn call_json(request: &Value) -> Value {
    serde_json::from_slice(&call(request.to_string().as_bytes())).expect("response is JSON")
}
