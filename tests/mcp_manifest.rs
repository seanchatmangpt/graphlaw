#![cfg(feature = "abi")]
//! MCP manifest court (GL28-14): the committed `registry/mcp-manifest.json` is
//! the byte identity of the emitter's `--print-mcp` output, obeys the registry
//! file laws, projects the 14-op registry table onto MCP tools that are live
//! dispatch arms, and its negative controls trip on mutated scratch copies —
//! never on the tree. Real binary, real files, no mocks.

use std::path::PathBuf;
use std::process::Command;

use serde_json::{Value, json};

fn registry_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("registry")
}

fn read(name: &str) -> String {
    let p = registry_dir().join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

fn emit(flag: &str) -> String {
    let exe = env!("CARGO_BIN_EXE_graphlaw-registry");
    let out = Command::new(exe).arg(flag).output().expect("emitter runs");
    assert!(
        out.status.success(),
        "graphlaw-registry {flag} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("emitter output is UTF-8")
}

/// Independent pretty-form derivation (sorted keys, 2-space indent) used to
/// prove the committed bytes are the canonical pretty serialization.
fn pretty(v: &Value, depth: usize, out: &mut String) {
    let pad = |n: usize| "  ".repeat(n);
    match v {
        Value::Object(m) if !m.is_empty() => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            out.push_str("{\n");
            for (i, k) in keys.iter().enumerate() {
                out.push_str(&pad(depth + 1));
                out.push_str(&serde_json::to_string(k).unwrap());
                out.push_str(": ");
                pretty(&m[*k], depth + 1, out);
                if i + 1 < keys.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad(depth));
            out.push('}');
        }
        Value::Array(a) if !a.is_empty() => {
            out.push_str("[\n");
            for (i, x) in a.iter().enumerate() {
                out.push_str(&pad(depth + 1));
                pretty(x, depth + 1, out);
                if i + 1 < a.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad(depth));
            out.push(']');
        }
        other => out.push_str(&serde_json::to_string(other).unwrap()),
    }
}

/// File laws shared with the registry JSON: ASCII-only, LF, single trailing
/// newline, sorted-key 2-space pretty form, float-free.
fn file_laws(text: &str) {
    assert!(text.is_ascii(), "manifest must be ASCII-only");
    assert!(!text.contains('\r'), "manifest must use LF");
    assert!(
        text.ends_with("}\n") && !text.ends_with("\n\n"),
        "single trailing newline"
    );
    let doc: Value = serde_json::from_str(text).expect("manifest parses");
    let mut expect = String::new();
    pretty(&doc, 0, &mut expect);
    expect.push('\n');
    assert_eq!(
        text, expect,
        "manifest is not sorted-key 2-space pretty form"
    );
    fn no_floats(v: &Value) {
        match v {
            Value::Number(n) => assert!(n.is_i64() || n.is_u64(), "float in manifest: {n}"),
            Value::Array(a) => a.iter().for_each(no_floats),
            Value::Object(m) => m.values().for_each(no_floats),
            _ => {}
        }
    }
    no_floats(&doc);
}

/// The court's independent request-type -> JSON Schema map. Total over the
/// closed registry vocabulary; an unmapped type fails loudly here just as a
/// missed enum variant fails the emitter's compile.
fn expected_type_schema(ty: &str) -> Value {
    match ty {
        "string" | "term" => json!({ "type": "string" }),
        "integer" => json!({ "type": "integer" }),
        "boolean" => json!({ "type": "boolean" }),
        "object" | "data_spec" => json!({ "type": "object" }),
        "any" => json!({}),
        "json_or_string" => json!({ "oneOf": [{ "type": "string" }, { "type": "object" }] }),
        "list<string>" | "list<term>" => {
            json!({ "type": "array", "items": { "type": "string" } })
        }
        "list<object>" => json!({ "type": "array", "items": { "type": "object" } }),
        "list<any>" => json!({ "type": "array", "items": {} }),
        "list<list<string>>" | "list<list<term>>" => {
            json!({ "type": "array", "items": { "type": "array", "items": { "type": "string" } } })
        }
        other => panic!("court: type `{other}` outside the closed registry vocabulary"),
    }
}

/// The expected inputSchema property for one registry request field,
/// independently derived from the registry JSON field row.
fn expected_property(field: &Value) -> Value {
    let name = field["name"].as_str().unwrap_or_default();
    let mut schema = expected_type_schema(field["type"].as_str().unwrap_or_default());
    if field["nullable"].as_bool().unwrap_or(false) {
        let t = schema
            .as_object_mut()
            .expect("type schema is an object")
            .remove("type")
            .unwrap_or_else(|| panic!("nullable field `{name}` has no type-keyed schema"));
        schema["type"] = json!([t, "null"]);
    }
    if let Some(values) = field["enum"].as_array() {
        schema["enum"] = json!(values);
    }
    if !field["default"].is_null() {
        schema["default"] = field["default"].clone();
    }
    schema
}

/// The closed type vocabulary, read from the committed registry schema file.
fn type_vocabulary() -> Vec<String> {
    let schema: Value = serde_json::from_str(&read("capability-registry.schema.json"))
        .expect("registry schema parses");
    schema["$defs"]["field_type"]["enum"]
        .as_array()
        .expect("schema field_type enum")
        .iter()
        .map(|v| v.as_str().expect("type name string").to_string())
        .collect()
}

/// Tool names must satisfy the MCP wire-name law.
fn wire_name_ok(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// Content laws over a parsed manifest, checked against the registry document
/// (`--print-json`). Panics on the first violation; used as the positive
/// control on the committed file and as the tripwire on mutants.
fn content_laws(manifest: &Value, registry: &Value) {
    assert_eq!(
        manifest["schema"],
        json!("graphlaw.mcp-manifest/1"),
        "schema const"
    );
    assert_eq!(
        manifest["schema"],
        json!(graphlaw::registry::MCP_MANIFEST_SCHEMA),
        "schema const matches the lib constant"
    );
    assert_eq!(manifest["protocolVersion"], json!("2025-06-18"));
    assert_eq!(manifest["serverInfo"]["name"], json!("graphlaw"));
    assert_eq!(
        manifest["serverInfo"]["version"], registry["graphlaw_version"],
        "serverInfo.version tracks the registry graphlaw_version"
    );
    assert_eq!(
        manifest["capabilities"],
        json!({ "tools": { "listChanged": false } })
    );

    let tools = manifest["tools"].as_array().expect("tools array");
    let ops = registry["ops"].as_array().expect("registry ops");
    assert_eq!(tools.len(), 14, "exactly 14 tools");
    assert_eq!(tools.len(), ops.len(), "one tool per registry op");
    let names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().expect("tool name string"))
        .collect();
    let op_names: Vec<&str> = ops
        .iter()
        .map(|o| o["name"].as_str().expect("op name string"))
        .collect();
    assert_eq!(
        names, op_names,
        "tool names equal registry op names in order"
    );
    for name in &names {
        assert!(
            wire_name_ok(name),
            "tool `{name}` violates the wire-name law"
        );
    }

    for (tool, op) in tools.iter().zip(ops.iter()) {
        let name = tool["name"].as_str().expect("tool name");
        assert_eq!(
            tool["description"], op["summary"],
            "{name}: description is the registry summary"
        );
        let schema = &tool["inputSchema"];
        assert_eq!(schema["type"], json!("object"), "{name}: inputSchema type");
        let fields = op["request"]["fields"].as_array().expect("request fields");
        let properties = schema["properties"].as_object().expect("properties object");
        assert_eq!(
            properties.len(),
            fields.len(),
            "{name}: one property per request field"
        );
        let required: Vec<String> = schema["required"]
            .as_array()
            .expect("required array")
            .iter()
            .map(|v| v.as_str().expect("required entry string").to_string())
            .collect();
        let want_required: Vec<&str> = fields
            .iter()
            .filter(|f| f["required"] == json!(true))
            .map(|f| f["name"].as_str().expect("field name"))
            .collect();
        assert_eq!(required, want_required, "{name}: required fields in order");
        for field in fields {
            let fname = field["name"].as_str().expect("field name");
            let property = properties
                .get(fname)
                .unwrap_or_else(|| panic!("{name}.{fname}: property missing"));
            assert_eq!(
                property,
                &expected_property(field),
                "{name}.{fname}: property schema"
            );
            // Nullable law, both directions, stated explicitly.
            let has_null = property["type"]
                .as_array()
                .is_some_and(|a| a.contains(&json!("null")));
            assert_eq!(
                has_null,
                field["nullable"] == json!(true),
                "{name}.{fname}: nullable <=> \"null\" in the type array"
            );
            // Enum law, both directions.
            assert_eq!(
                property.get("enum").is_some(),
                field["enum"].is_array(),
                "{name}.{fname}: enum <=> registry enum_values"
            );
            // Default law, both directions.
            assert_eq!(
                property.get("default").is_some(),
                !field["default"].is_null(),
                "{name}.{fname}: default <=> registry default"
            );
        }
    }
}

/// A scratch directory that removes itself, even on panic: mutants are built
/// on copies here, never on the tree.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!("gl2814-mcp-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&path).expect("scratch dir created");
        Scratch { path }
    }

    fn write(&self, name: &str, body: &str) -> PathBuf {
        let p = self.path.join(name);
        std::fs::write(&p, body).expect("scratch file written");
        p
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn trip_or_fail(description: &str, f: impl FnOnce() + std::panic::UnwindSafe) {
    let result = std::panic::catch_unwind(f);
    assert!(result.is_err(), "mutant must trip the law: {description}");
}

#[test]
fn committed_bytes_equal_emission() {
    assert_eq!(read("mcp-manifest.json"), emit("--print-mcp"));
}

#[test]
fn check_flag_accepts_committed_tree() {
    let exe = env!("CARGO_BIN_EXE_graphlaw-registry");
    let out = Command::new(exe)
        .arg("--check")
        .output()
        .expect("emitter runs");
    assert!(
        out.status.success(),
        "--check failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("ok      mcp-manifest.json"));
}

#[test]
fn manifest_laws() {
    let text = read("mcp-manifest.json");
    file_laws(&text);
    let manifest: Value = serde_json::from_str(&text).expect("manifest parses");
    let registry: Value =
        serde_json::from_str(&emit("--print-json")).expect("registry json parses");
    content_laws(&manifest, &registry);

    // The court's independent type map is total over the closed vocabulary:
    // every type the registry schema admits maps to a JSON Schema here.
    let vocab = type_vocabulary();
    assert_eq!(vocab.len(), 14, "closed type vocabulary size");
    for ty in &vocab {
        let _ = expected_type_schema(ty);
    }
    // And the wire-name law holds for every tool name (redundant with
    // content_laws, kept as the named court row).
    for tool in manifest["tools"].as_array().unwrap() {
        assert!(wire_name_ok(tool["name"].as_str().unwrap()));
    }
}

/// Arm names of `fn dispatch`, parsed from `src/abi.rs` (technique of
/// tests/registry_dispatch.rs).
fn dispatch_arms() -> Vec<String> {
    let src = include_str!("../src/abi.rs");
    let start = src
        .find("fn dispatch(")
        .expect("fn dispatch present in src/abi.rs");
    let body = &src[start..];
    let end = body
        .find("unknown op")
        .expect("dispatch fallthrough `unknown op` present");
    let mut arms = Vec::new();
    for line in body[..end].lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix('"')
            && let Some(q) = rest.find('"')
            && rest[q + 1..].trim_start().starts_with("=>")
        {
            arms.push(rest[..q].to_string());
        }
    }
    arms
}

#[test]
fn tools_are_live_dispatch_arms() {
    let arms = dispatch_arms();
    assert!(!arms.is_empty(), "no dispatch arms parsed from src/abi.rs");
    let manifest: Value =
        serde_json::from_str(&read("mcp-manifest.json")).expect("manifest parses");
    for tool in manifest["tools"].as_array().unwrap() {
        let name = tool["name"].as_str().expect("tool name");
        assert!(
            arms.iter().any(|a| a == name),
            "tool `{name}` is not a live dispatch arm in src/abi.rs"
        );
    }
}

#[test]
fn negative_controls_reject_mutations() {
    let text = read("mcp-manifest.json");
    let manifest: Value = serde_json::from_str(&text).expect("manifest parses");
    let registry: Value =
        serde_json::from_str(&emit("--print-json")).expect("registry json parses");

    // Positive controls: the committed bytes and content pass the same laws.
    file_laws(&text);
    content_laws(&manifest, &registry);
    let live_row = mcp_artifacts_row(&read("ARTIFACTS.blake3"));
    assert_eq!(
        artifacts_row(&live_row),
        artifacts_row(&format!(
            "mcp-manifest {} - registry/mcp-manifest.json",
            graphlaw::registry::mcp_manifest_blake3()
        )),
        "committed mcp-manifest ARTIFACTS row equals the live digest row"
    );

    let scratch = Scratch::new("negatives");

    // Mutant 1: rename a tool. Content laws must trip (name drift) and the
    // renamed tool must fail the dispatch-arm law.
    let mut renamed = manifest.clone();
    renamed["tools"][0]["name"] = json!("zz_renamed_tool");
    let renamed_text = serde_json::to_string_pretty(&renamed).unwrap();
    let p = scratch.write("renamed.json", &renamed_text);
    let renamed_reread: Value =
        serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    trip_or_fail("renamed tool", || content_laws(&renamed_reread, &registry));
    assert!(
        !dispatch_arms().iter().any(|a| a == "zz_renamed_tool"),
        "a renamed tool is not a dispatch arm"
    );

    // Mutant 2: drop a required entry (sparql.data). Content laws must trip.
    let mut dropped = manifest.clone();
    let required = dropped["tools"][5]["inputSchema"]["required"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(
        required[0],
        json!("data"),
        "control: sparql.data is required"
    );
    dropped["tools"][5]["inputSchema"]["required"] = json!(required[1..].to_vec());
    let p = scratch.write(
        "dropped-required.json",
        &serde_json::to_string_pretty(&dropped).unwrap(),
    );
    let dropped_reread: Value =
        serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    trip_or_fail("dropped required entry", || {
        content_laws(&dropped_reread, &registry)
    });

    // Mutant 3: stale the mcp-manifest ARTIFACTS row on a scratch copy of the
    // pin file. The row law must flag drift; --check's own acceptance of the
    // committed row is the positive control above and in
    // check_flag_accepts_committed_tree.
    let artifacts = read("ARTIFACTS.blake3");
    let stale = artifacts.replacen(
        &format!("mcp-manifest {}", graphlaw::registry::mcp_manifest_blake3()),
        &format!("mcp-manifest blake3:{}", "0".repeat(64)),
        1,
    );
    assert_ne!(stale, artifacts, "mutation applied");
    let p = scratch.write("ARTIFACTS.blake3", &stale);
    let stale_text = std::fs::read_to_string(&p).unwrap();
    trip_or_fail("stale mcp-manifest ARTIFACTS row", || {
        let row = mcp_artifacts_row(&stale_text);
        assert_eq!(
            row,
            artifacts_row(&format!(
                "mcp-manifest {} - registry/mcp-manifest.json",
                graphlaw::registry::mcp_manifest_blake3()
            )),
            "stale row must never equal the live digest row"
        );
    });
}

/// The `mcp-manifest` row of an ARTIFACTS.blake3 body, or a loud miss.
fn mcp_artifacts_row(text: &str) -> String {
    text.lines()
        .find(|l| l.starts_with("mcp-manifest "))
        .unwrap_or_else(|| panic!("ARTIFACTS.blake3 has no `mcp-manifest` row"))
        .to_string()
}

/// Normalized `<digest> <name>` pair of one ARTIFACTS row.
fn artifacts_row(row: &str) -> String {
    let fields: Vec<&str> = row.split_whitespace().collect();
    format!("{} {}", fields[1], fields[3])
}

#[test]
fn emission_is_deterministic() {
    let a = emit("--print-mcp");
    let b = emit("--print-mcp");
    assert_eq!(a, b, "two emissions must be byte-equal");
    assert_eq!(
        a,
        read("mcp-manifest.json"),
        "emission equals the committed bytes"
    );
}
