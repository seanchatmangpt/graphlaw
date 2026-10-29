//! MCP manifest emitter: `registry/mcp-manifest.json` is the projection of the
//! single Rust registry table (`definitions::OPS` via `emit_json::document`)
//! onto the Model Context Protocol tool surface.
//!
//! File laws are the registry-JSON laws: ASCII-only, float-free, sorted keys,
//! 2-space pretty, LF, single trailing newline. The `mcp-manifest` ARTIFACTS
//! row digests the canonical document (same rule as the `registry` and
//! `surface` rows), so the pretty bytes are never a digest input.

use std::fmt::Write as _;

use serde_json::{Map, Value, json};

use super::emit_json::{blake3_of, canonical, document, sorted_entries};

/// Schema identifier of the MCP manifest document.
pub const MCP_MANIFEST_SCHEMA: &str = "graphlaw.mcp-manifest/1";

/// MCP protocol revision the manifest targets.
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// The closed request-field type vocabulary as a Rust enum. The schema match
/// in [`type_schema`] is non-wildcard over this enum, so a vocabulary addition
/// that skips it is a compile error, never a silent schema hole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldType {
    Str,
    Int,
    Bool,
    Obj,
    Any,
    DataSpec,
    Term,
    JsonOrString,
    ListStr,
    ListObj,
    ListAny,
    ListTerm,
    ListListStr,
    ListListTerm,
}

fn parse_type(ty: &str) -> FieldType {
    match ty {
        "string" => FieldType::Str,
        "integer" => FieldType::Int,
        "boolean" => FieldType::Bool,
        "object" => FieldType::Obj,
        "any" => FieldType::Any,
        "data_spec" => FieldType::DataSpec,
        "term" => FieldType::Term,
        "json_or_string" => FieldType::JsonOrString,
        "list<string>" => FieldType::ListStr,
        "list<object>" => FieldType::ListObj,
        "list<any>" => FieldType::ListAny,
        "list<term>" => FieldType::ListTerm,
        "list<list<string>>" => FieldType::ListListStr,
        "list<list<term>>" => FieldType::ListListTerm,
        other => panic!("emit_mcp: field type `{other}` is outside the closed registry vocabulary"),
    }
}

/// JSON Schema for one closed-vocabulary type; total over the vocabulary.
fn type_schema(ty: FieldType) -> Value {
    match ty {
        FieldType::Str => json!({ "type": "string" }),
        FieldType::Int => json!({ "type": "integer" }),
        FieldType::Bool => json!({ "type": "boolean" }),
        FieldType::Obj | FieldType::DataSpec => json!({ "type": "object" }),
        FieldType::Any => json!({}),
        FieldType::Term => json!({ "type": "string" }),
        FieldType::JsonOrString => {
            json!({ "oneOf": [{ "type": "string" }, { "type": "object" }] })
        }
        FieldType::ListStr => array_of(json!({ "type": "string" })),
        FieldType::ListObj => array_of(json!({ "type": "object" })),
        FieldType::ListAny => array_of(json!({})),
        FieldType::ListTerm => array_of(json!({ "type": "string" })),
        FieldType::ListListStr => array_of(array_of(json!({ "type": "string" }))),
        FieldType::ListListTerm => array_of(array_of(json!({ "type": "string" }))),
    }
}

fn array_of(items: Value) -> Value {
    json!({ "type": "array", "items": items })
}

/// The inputSchema property for one registry request field: the type mapping,
/// plus the nullable, enum and default projections. Nullable fields are all
/// string-`type` schemas today; any other shape fails the emission loudly
/// instead of silently dropping the null case.
fn field_property(field: &Value) -> Value {
    let name = field["name"].as_str().unwrap_or_default();
    let mut schema = type_schema(parse_type(field["type"].as_str().unwrap_or_default()));
    if field["nullable"].as_bool().unwrap_or(false) {
        let t = schema
            .as_object_mut()
            .expect("type schema is an object")
            .remove("type")
            .unwrap_or_else(|| {
                panic!("emit_mcp: nullable field `{name}` has no type-keyed schema")
            });
        schema["type"] = json!([t, "null"]);
    }
    if let Some(values) = field.get("enum").and_then(Value::as_array) {
        schema["enum"] = Value::Array(values.clone());
    }
    match field.get("default") {
        None | Some(Value::Null) => {}
        Some(default) => schema["default"] = default.clone(),
    }
    schema
}

/// The MCP wire-name law for tool names.
fn assert_wire_name(name: &str) {
    let ok = !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    assert!(
        ok,
        "emit_mcp: op name `{name}` violates the wire-name law ^[a-zA-Z0-9_-]{{1,64}}$"
    );
}

/// The 14 registry ops in registry order, projected onto MCP tools.
fn tools_value() -> Vec<Value> {
    let ops = document()["ops"].as_array().expect("registry ops");
    ops.iter()
        .map(|op| {
            let name = op["name"].as_str().expect("op name");
            assert_wire_name(name);
            let fields = op["request"]["fields"].as_array().expect("request fields");
            let mut properties = Map::new();
            let mut required: Vec<Value> = Vec::new();
            for field in fields {
                let field_name = field["name"].as_str().expect("field name").to_owned();
                if field["required"].as_bool().unwrap_or(false) {
                    required.push(json!(field_name));
                }
                properties.insert(field_name, field_property(field));
            }
            json!({
                "name": name,
                "description": op["summary"].clone(),
                "inputSchema": {
                    "type": "object",
                    "properties": Value::Object(properties),
                    "required": Value::Array(required),
                },
            })
        })
        .collect()
}

/// The complete MCP manifest document: 14 tools from the single source table.
pub fn mcp_manifest_value() -> Value {
    let registry = document();
    json!({
        "schema": MCP_MANIFEST_SCHEMA,
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "serverInfo": {
            "name": "graphlaw",
            "version": registry["graphlaw_version"].clone(),
        },
        "capabilities": { "tools": { "listChanged": false } },
        "tools": tools_value(),
    })
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

/// Pretty form of the committed file: sorted keys, 2-space indent, LF. Mirrors
/// `emit_json::write_json` (that writer is private to its module) with the
/// same `sorted_entries` ordering, so the two artifacts share one file law.
fn write_pretty(v: &Value, depth: usize, out: &mut String) {
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
                out.push('\n');
                out.push_str(&" ".repeat(depth + 2));
                write_pretty(item, depth + 2, out);
            }
            out.push('\n');
            out.push_str(&" ".repeat(depth));
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
                out.push('\n');
                out.push_str(&" ".repeat(depth + 2));
                escape_json(k, out);
                out.push_str(": ");
                write_pretty(item, depth + 2, out);
            }
            out.push('\n');
            out.push_str(&" ".repeat(depth));
            out.push('}');
        }
    }
}

/// The committed `registry/mcp-manifest.json` bytes: sorted keys, 2-space
/// indent, LF, single trailing newline.
pub fn mcp_manifest_pretty() -> String {
    let mut out = String::new();
    write_pretty(&mcp_manifest_value(), 0, &mut out);
    out.push('\n');
    out
}

/// `blake3:<hex>` of the canonical manifest document, the ARTIFACTS
/// `mcp-manifest` row input (canonical bytes, never the pretty file bytes).
pub fn mcp_manifest_blake3() -> String {
    blake3_of(canonical(&mcp_manifest_value()).as_bytes())
}
