use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

use serde_json::{Value, json};

use crate::abi::{
    ABI_VERSION, MAX_ATOMS_PER_FIELD, MAX_JSON_DEPTH, MAX_OUTSTANDING_ALLOC_BYTES,
    MAX_PLAN_ACTIONS, MAX_POLICY_ENTRIES, MAX_REQUEST_BYTES,
};

use super::definitions::*;

fn json_string_array(items: &[&str]) -> Value {
    Value::Array(items.iter().map(|s| json!(s)).collect())
}

fn field_json(f: &Field, order: usize) -> Value {
    let enum_v = if f.enum_values.is_empty() {
        Value::Null
    } else {
        json_string_array(f.enum_values)
    };
    let default = match f.default {
        FieldDefault::None => Value::Null,
        FieldDefault::Str(s) => json!(s),
        FieldDefault::Int(i) => json!(i),
        FieldDefault::Bool(b) => json!(b),
    };
    json!({
        "name": f.name, "order": order, "type": f.ty, "required": f.required,
        "nullable": f.nullable, "doc": f.doc, "enum": enum_v, "default": default,
    })
}

fn fields_json(fields: &[Field]) -> Value {
    Value::Array(
        fields
            .iter()
            .enumerate()
            .map(|(i, f)| field_json(f, i + 1))
            .collect(),
    )
}

fn dialects_json(rows: &[DialectRow], kind: &str, first_order: usize) -> Value {
    Value::Array(
        rows.iter()
            .enumerate()
            .map(|(i, d)| {
                json!({
                    "name": d.name, "order": first_order + i, "aliases": json_string_array(d.aliases),
                    "response_name": d.response_name,
                    "media_type": d.media_type, "kind": kind,
                })
            })
            .collect(),
    )
}

fn surface_document() -> Value {
    json!({
        "abi_version": ABI_VERSION,
        "ops": op_names(),
        "other_dialects": other_dialect_names(),
        "rdf_dialects": rdf_dialect_names(),
    })
}

/// Canonical receipt-surface document: BLAKE3 digests of the receipt-critical
/// compiler/eval sources, pinned so a receipt's standing can be tied to the
/// exact bytes of the code that minted it.
fn receipt_surface_document() -> Value {
    const FILES: &[(&str, &[u8])] = &[
        ("src/duckdb_eval.rs", include_bytes!("../duckdb_eval.rs")),
        ("src/sql_ast.rs", include_bytes!("../sql_ast.rs")),
        ("src/sql_ast/ast.rs", include_bytes!("../sql_ast/ast.rs")),
        (
            "src/sql_ast/refusals.rs",
            include_bytes!("../sql_ast/refusals.rs"),
        ),
        (
            "src/sql_ast/serializer.rs",
            include_bytes!("../sql_ast/serializer.rs"),
        ),
        // .ttl is pure UTF-8 data with no build-time transform, so byte
        // identity is the invariant the receipt-surface pin needs; include_str!
        // would additionally assert valid UTF-8 at compile time but yield an
        // identical &[u8] via .as_bytes(). include_bytes! keeps every entry in
        // FILES the same type (&[u8]) without a per-entry map, and digests the
        // exact on-disk bytes either way.
        (
            "ontologies/sql-ast.ttl",
            include_bytes!("../../ontologies/sql-ast.ttl"),
        ),
    ];
    let files: Vec<Value> = FILES
        .iter()
        .map(|(path, bytes)| {
            json!({
                "path": path,
                "blake3": blake3_of(bytes),
                "bytes": bytes.len(),
            })
        })
        .collect();
    json!({
        "schema": "graphlaw.receipt-surface/1",
        "files": files,
    })
}

pub(super) fn blake3_of(bytes: &[u8]) -> String {
    let digest = blake3::hash(bytes);
    let mut out = String::with_capacity(7 + 64);
    out.push_str("blake3:");
    for b in digest.as_bytes() {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Registry document without the `registry_blake3` key.
fn base_document() -> Value {
    let ops: Vec<Value> = OPS
        .iter()
        .enumerate()
        .map(|(i, op)| {
            json!({
                "name": op.name, "order": i + 1, "summary": op.summary,
                "request": {"fields": fields_json(op.request)},
                "responses": op.responses.iter().map(|v| json!({
                    "tag": v.tag, "tag_field": v.tag_field, "fields": fields_json(v.fields),
                })).collect::<Vec<_>>(),
                "refusal_kinds": json_string_array(op.refusal_kinds),
                "refusal_codes": json_string_array(op.refusal_codes),
            })
        })
        .collect();
    json!({
        "schema": REGISTRY_SCHEMA,
        "graphlaw_version": env!("CARGO_PKG_VERSION"),
        "abi_version": ABI_VERSION,
        "surface_blake3": surface_blake3(),
        "receipt_surface_blake3": receipt_surface_blake3(),
        "receipt_surface": receipt_surface_document(),
        "authorities": crate::BACKEND_AUTHORITIES.iter().map(|a| json!({
            "capability": a.capability, "authority": a.authority,
        })).collect::<Vec<_>>(),
        "engines": json_string_array(ENGINES),
        "refusal_kinds": json_string_array(REFUSAL_KINDS),
        "rdf_dialects": dialects_json(RDF_DIALECTS, "rdf", 1),
        "other_dialects": dialects_json(OTHER_DIALECTS, "other", RDF_DIALECTS.len() + 1),
        "regimes": json_string_array(REGIMES),
        "lease_ceilings": json_string_array(LEASE_CEILINGS),
        "lease_reasons": json_string_array(LEASE_REASONS),
        "receipt_reasons": json_string_array(RECEIPT_REASONS),
        "policy_refusal_kinds": json_string_array(POLICY_REFUSAL_KINDS),
        "limits": {
            "max_request_bytes": MAX_REQUEST_BYTES,
            "max_json_depth": MAX_JSON_DEPTH,
            "max_plan_actions": MAX_PLAN_ACTIONS,
            "max_atoms_per_field": MAX_ATOMS_PER_FIELD,
            "max_policy_entries": MAX_POLICY_ENTRIES,
            "n3_max_iterations": crate::law::N3_MAX_ITERATIONS,
            "n3_max_derived_facts": crate::law::N3_MAX_DERIVED_FACTS,
            "n3_max_total_bytes": crate::law::N3_MAX_TOTAL_BYTES,
            "n3_max_term_bytes": crate::law::N3_MAX_TERM_BYTES,
            "n3_max_match_steps": crate::law::N3_MAX_MATCH_STEPS,
            "n3_max_total_steps": crate::law::N3_MAX_TOTAL_STEPS,
            "n3_max_backward_depth": crate::law::N3_MAX_BACKWARD_DEPTH,
            "n3_max_backward_solutions_per_goal": crate::law::N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL,
            "n3_max_term_depth": crate::law::N3_MAX_TERM_DEPTH,
            "max_plan_total_atoms": crate::plan::MAX_PLAN_TOTAL_ATOMS,
            "hooks_max_rounds": crate::hooks::MAX_ROUNDS,
            "hooks_max_firings": crate::hooks::MAX_FIRINGS,
            "hooks_max_state_quads": crate::hooks::MAX_STATE_QUADS,
            "max_outstanding_alloc_bytes": MAX_OUTSTANDING_ALLOC_BYTES,
        },
        "limit_meta": limit_meta_json(),
        "models": models_json(),
        "model_enums": model_enums_json(),
        "refusal_codes": REFUSAL_CODES.iter().enumerate().map(|(i, c)| json!({
            "code": c.code, "order": i + 1, "kind": c.kind, "fields": fields_json(c.fields),
        })).collect::<Vec<_>>(),
        "law_steps": LAW_STEPS.iter().enumerate().map(|(i, s)| json!({
            "name": s.name, "order": i + 1, "ceiling": s.ceiling, "fields": fields_json(s.fields),
        })).collect::<Vec<_>>(),
        "ops": ops,
    })
}

/// One limit's metadata: (name, scope, unit, source path, machine refusal name).
/// The refusal name is `None` where the source emits no machine name for the
/// limit (hook round/firing caps are plain `Unsupported` refusals; the wasm
/// allocation cap returns a null pointer).
const LIMIT_META: &[(&str, &str, &str, &str, Option<&str>)] = &[
    (
        "max_request_bytes",
        "abi",
        "bytes",
        "src/abi.rs",
        Some("request_bytes"),
    ),
    (
        "max_json_depth",
        "abi",
        "count",
        "src/abi.rs",
        Some("json_depth"),
    ),
    (
        "max_plan_actions",
        "abi",
        "count",
        "src/abi.rs",
        Some("plan_actions"),
    ),
    (
        "max_atoms_per_field",
        "abi",
        "count",
        "src/abi.rs",
        Some("atoms_per_field"),
    ),
    (
        "max_policy_entries",
        "abi",
        "count",
        "src/abi.rs",
        Some("policy_entries"),
    ),
    (
        "n3_max_iterations",
        "n3",
        "count",
        "src/law.rs",
        Some("n3_iterations"),
    ),
    (
        "n3_max_derived_facts",
        "n3",
        "count",
        "src/law.rs",
        Some("n3_derived_facts"),
    ),
    (
        "n3_max_total_bytes",
        "n3",
        "bytes",
        "src/law.rs",
        Some("n3_total_bytes"),
    ),
    (
        "n3_max_term_bytes",
        "n3",
        "bytes",
        "src/law.rs",
        Some("n3_term_bytes"),
    ),
    (
        "n3_max_match_steps",
        "n3",
        "steps",
        "src/law.rs",
        Some("n3_match_steps"),
    ),
    (
        "n3_max_total_steps",
        "n3",
        "steps",
        "src/law.rs",
        // `ReasonerLimit::MatchSteps` answers `n3_match_steps` for both the
        // per-search and the run-wide ceiling; the receipt's `max`
        // discriminates them.
        Some("n3_match_steps"),
    ),
    (
        "n3_max_backward_depth",
        "n3",
        "count",
        "src/law.rs",
        Some("n3_backward_depth"),
    ),
    (
        "n3_max_backward_solutions_per_goal",
        "n3",
        "count",
        "src/law.rs",
        Some("n3_backward_solutions_per_goal"),
    ),
    (
        "n3_max_term_depth",
        "n3",
        "count",
        "src/law.rs",
        Some("n3_term_depth"),
    ),
    (
        "max_plan_total_atoms",
        "plan",
        "count",
        "src/plan.rs",
        Some("plan_total_atoms"),
    ),
    ("hooks_max_rounds", "hooks", "rounds", "src/hooks.rs", None),
    (
        "hooks_max_firings",
        "hooks",
        "firings",
        "src/hooks.rs",
        None,
    ),
    (
        "hooks_max_state_quads",
        "hooks",
        "quads",
        "src/hooks.rs",
        Some("state_quads"),
    ),
    (
        "max_outstanding_alloc_bytes",
        "wasm",
        "bytes",
        "src/abi.rs",
        None,
    ),
];

fn limit_meta_json() -> Value {
    let mut m = serde_json::Map::new();
    for (name, scope, unit, source, refusal) in LIMIT_META {
        let mut entry = json!({"scope": scope, "unit": unit, "source": source});
        if let Some(r) = refusal {
            entry["refusal_name"] = json!(r);
        }
        m.insert((*name).into(), entry);
    }
    Value::Object(m)
}

/// A model field: (name, type, required, nullable, doc).
type ModelField = (&'static str, &'static str, bool, bool, &'static str);
/// A model: (name, rust path, doc, fields).
type ModelRow = (
    &'static str,
    &'static str,
    &'static str,
    &'static [ModelField],
);

const MODELS: &[ModelRow] = &[
    (
        "Lease",
        "graphlaw::law::Lease",
        "Time-boxed authority grant for law steps.",
        &[
            ("id", "string", true, false, "Lease identifier."),
            (
                "holder",
                "string",
                true,
                false,
                "Identity of the lease holder.",
            ),
            (
                "ceiling",
                "enum:Ceiling",
                true,
                false,
                "Highest authority the lease grants.",
            ),
            (
                "scope",
                "list<string>",
                true,
                false,
                "Step names the lease covers.",
            ),
            (
                "expires_unix",
                "integer",
                true,
                false,
                "Expired when now >= expires_unix.",
            ),
            (
                "issued_unix",
                "integer",
                true,
                false,
                "Issuer-stated issue time; 0 means not stated.",
            ),
        ],
    ),
    (
        "SignedLease",
        "graphlaw::law::SignedLease",
        "A lease with its Ed25519 attestation.",
        &[
            (
                "lease",
                "model:Lease",
                true,
                false,
                "The lease being attested.",
            ),
            (
                "attestation",
                "model:Attestation",
                true,
                false,
                "Signature over the lease.",
            ),
        ],
    ),
    (
        "Receipt",
        "graphlaw::law::Receipt",
        "Record of one executed law step.",
        &[
            ("parent", "string", true, false, "Id of the parent state."),
            ("child", "string", true, false, "Id of the child state."),
            (
                "step",
                "string",
                true,
                false,
                "Step name, for example derive:rdfs.",
            ),
            (
                "authority",
                "string",
                true,
                false,
                "Backend authority capability.",
            ),
            (
                "revision",
                "string",
                true,
                false,
                "Backend authority revision.",
            ),
            ("added", "integer", true, false, "Quads added by the step."),
            (
                "lease_id",
                "string",
                false,
                true,
                "Lease id when the step ran under a lease.",
            ),
            (
                "plan_blake3",
                "string",
                false,
                true,
                "Plan digest for plan steps.",
            ),
            (
                "subject_blake3",
                "string",
                false,
                true,
                "Digest of the external subject, when bound.",
            ),
        ],
    ),
    (
        "Attestation",
        "graphlaw::attest::Attestation",
        "Ed25519 attestation over a canonical payload.",
        &[
            (
                "payload_blake3",
                "string",
                true,
                false,
                "Lowercase hex BLAKE3 of the canonical payload.",
            ),
            (
                "key_id",
                "string",
                true,
                false,
                "Identifier of the signing key.",
            ),
            (
                "signature",
                "string",
                true,
                false,
                "Lowercase hex Ed25519 signature over the payload.",
            ),
        ],
    ),
    (
        "Plan",
        "graphlaw::plan::Plan",
        "Ordered candidate plan and the goal it claims to reach.",
        &[
            (
                "actions",
                "list<model:Action>",
                true,
                false,
                "Actions in replay order.",
            ),
            (
                "goal",
                "string",
                true,
                false,
                "Triples that must hold after the last action.",
            ),
            (
                "goal_not",
                "string",
                true,
                false,
                "Triples that must be absent after the last action.",
            ),
        ],
    ),
    (
        "Action",
        "graphlaw::plan::Action",
        "One plan action over ground triples.",
        &[
            (
                "name",
                "string",
                true,
                false,
                "Action name, for example go a b.",
            ),
            (
                "pre",
                "string",
                true,
                false,
                "Triples that must hold before the action.",
            ),
            (
                "pre_not",
                "string",
                true,
                false,
                "Triples that must be absent before the action.",
            ),
            ("add", "string", true, false, "Triples added by the action."),
            (
                "del",
                "string",
                true,
                false,
                "Triples removed by the action.",
            ),
        ],
    ),
    (
        "PolicyEntry",
        "graphlaw::policy::Entry",
        "One FOND policy entry.",
        &[
            (
                "state",
                "string",
                true,
                false,
                "State the entry applies to.",
            ),
            (
                "action",
                "string",
                true,
                false,
                "Action chosen in the state.",
            ),
            (
                "outcomes",
                "list<model:PolicyOutcome>",
                true,
                false,
                "Possible outcomes of the action.",
            ),
        ],
    ),
    (
        "PolicyOutcome",
        "graphlaw::policy::Outcome",
        "One possible outcome of a policy action.",
        &[
            (
                "state",
                "string",
                true,
                false,
                "State reached by the outcome.",
            ),
            (
                "probability_ppm",
                "integer",
                true,
                false,
                "Probability in parts per million.",
            ),
        ],
    ),
];

fn models_json() -> Value {
    Value::Array(
        MODELS
            .iter()
            .enumerate()
            .map(|(i, (name, rust_path, doc, fields))| {
                json!({
                    "name": name,
                    "order": i + 1,
                    "elixir_module": format!("AshGraphLaw.Model.{name}"),
                    "rust_path": rust_path,
                    "doc": doc,
                    "fields": fields.iter().enumerate().map(|(j, (fname, ty, required, nullable, fdoc))| json!({
                        "name": fname, "order": j + 1, "type": ty,
                        "required": required, "nullable": nullable, "doc": fdoc,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect(),
    )
}

fn model_enums_json() -> Value {
    let rows: [(&str, &[&str]); 4] = [
        ("Ceiling", LEASE_CEILINGS),
        ("LeaseReason", LEASE_REASONS),
        ("ReceiptReason", RECEIPT_REASONS),
        ("PolicyRefusalKind", POLICY_REFUSAL_KINDS),
    ];
    Value::Array(
        rows.iter()
            .enumerate()
            .map(|(i, (name, values))| {
                json!({"name": name, "order": i + 1, "values": json_string_array(values)})
            })
            .collect(),
    )
}

static DOCUMENT: OnceLock<Value> = OnceLock::new();

pub(super) fn document() -> &'static Value {
    DOCUMENT.get_or_init(|| {
        let mut doc = base_document();
        let digest = blake3_of(canonical(&doc).as_bytes());
        if let Some(o) = doc.as_object_mut() {
            o.insert("registry_blake3".into(), json!(digest));
        }
        doc
    })
}

/// `blake3:<hex>` of the canonical surface document (ABI version, op names,
/// RDF dialect names, other dialect names).
pub fn surface_blake3() -> String {
    blake3_of(canonical(&surface_document()).as_bytes())
}

/// `blake3:<hex>` of the canonical receipt-surface document (per-file digests of
/// `src/duckdb_eval.rs` and `src/sql_ast.rs`, the receipt-minting surface).
/// Same invariants as [`surface_blake3`]: digest over canonical bytes, never
/// over any pretty-printed serialization.
pub fn receipt_surface_blake3() -> String {
    blake3_of(canonical(&receipt_surface_document()).as_bytes())
}

/// `blake3:<hex>` of the canonical registry document with `registry_blake3` removed.
pub fn registry_blake3() -> String {
    document()
        .get("registry_blake3")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_default()
}

/// The complete registry document, including `registry_blake3`.
pub fn registry_value() -> Value {
    document().clone()
}

fn escape_json(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

pub(super) fn sorted_entries(m: &serde_json::Map<String, Value>) -> BTreeMap<&str, &Value> {
    m.iter().map(|(k, v)| (k.as_str(), v)).collect()
}

fn write_json(v: &Value, pretty: Option<usize>, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => escape_json(s, out),
        Value::Array(a) => {
            if a.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, item) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if let Some(depth) = pretty {
                    out.push('\n');
                    out.push_str(&" ".repeat(depth + 2));
                }
                write_json(item, pretty.map(|d| d + 2), out);
            }
            if let Some(depth) = pretty {
                out.push('\n');
                out.push_str(&" ".repeat(depth));
            }
            out.push(']');
        }
        Value::Object(m) => {
            if m.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (k, item)) in sorted_entries(m).into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if let Some(depth) = pretty {
                    out.push('\n');
                    out.push_str(&" ".repeat(depth + 2));
                }
                escape_json(k, out);
                out.push(':');
                if pretty.is_some() {
                    out.push(' ');
                }
                write_json(item, pretty.map(|d| d + 2), out);
            }
            if let Some(depth) = pretty {
                out.push('\n');
                out.push_str(&" ".repeat(depth));
            }
            out.push('}');
        }
    }
}

/// Canonical JSON: compact, object keys sorted by byte order, arrays in order,
/// minimal RFC 8259 escaping, non-ASCII raw. The digest input.
pub fn canonical(v: &Value) -> String {
    let mut out = String::new();
    write_json(v, None, &mut out);
    out
}

/// The committed `registry/capability-registry.json` bytes: sorted keys,
/// 2-space indent, LF, trailing newline.
pub fn registry_json_pretty() -> String {
    let mut out = String::new();
    write_json(document(), Some(0), &mut out);
    out.push('\n');
    out
}

pub(super) type Triple = (String, String, String);
