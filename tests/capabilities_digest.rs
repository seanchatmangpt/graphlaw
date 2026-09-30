//! Capabilities-digest court (feature `abi`, Chicago: real `graphlaw::abi::call_json`).
//!
//! The `capabilities` response exposes `registry_schema`, `registry_sha256`, `surface_sha256`.
//! The surface digest is recomputed here independently from the live response with a local
//! canonical-JSON encoder and SHA-256, so the court does not trust the registry's own helper.
#![cfg(feature = "abi")]

use graphlaw::abi::call_json;
use graphlaw::registry;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// Canonical JSON per the registry contract: sorted keys, compact, RFC 8259 minimal escapes,
/// raw UTF-8, integers decimal.
fn canonical(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            assert!(
                n.is_i64() || n.is_u64(),
                "floats are not admitted in canonical JSON"
            );
            n.to_string()
        }
        Value::String(s) => {
            let mut o = String::from("\"");
            for c in s.chars() {
                match c {
                    '"' => o.push_str("\\\""),
                    '\\' => o.push_str("\\\\"),
                    '\u{08}' => o.push_str("\\b"),
                    '\t' => o.push_str("\\t"),
                    '\n' => o.push_str("\\n"),
                    '\u{0c}' => o.push_str("\\f"),
                    '\r' => o.push_str("\\r"),
                    c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
                    c => o.push(c),
                }
            }
            o.push('"');
            o
        }
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(canonical).collect::<Vec<_>>().join(",")
        ),
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            let parts: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    format!(
                        "{}:{}",
                        canonical(&Value::String(k.clone())),
                        canonical(&m[k])
                    )
                })
                .collect();
            format!("{{{}}}", parts.join(","))
        }
    }
}

fn sha256(v: &Value) -> String {
    let mut h = Sha256::new();
    h.update(canonical(v).as_bytes());
    let d = h.finalize();
    let hex: String = d.iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{hex}")
}

fn surface_of(caps: &Value) -> Value {
    json!({
        "abi_version": caps["abi_version"],
        "ops": caps["ops"],
        "other_dialects": caps["other_dialects"],
        "rdf_dialects": caps["rdf_dialects"],
    })
}

fn live() -> Value {
    let caps = call_json(&json!({"op": "capabilities"}));
    assert_eq!(caps["ok"], json!(true), "capabilities refused: {caps}");
    caps
}

fn registry_doc() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/registry/capability-registry.json"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("emitted registry missing at {path}: {e} (run graphlaw-registry --write)")
    });
    serde_json::from_str(&text).expect("registry JSON parses")
}

#[test]
fn positive_control_capabilities_carries_registry_fields_equal_to_registry() {
    let caps = live();
    assert_eq!(
        caps["registry_schema"],
        json!("graphlaw.capability-registry/1")
    );
    let reg = registry::registry_sha256().to_string();
    let surf = registry::surface_sha256().to_string();
    assert_eq!(caps["registry_sha256"], json!(reg));
    assert_eq!(caps["surface_sha256"], json!(surf));
    assert!(
        reg.starts_with("sha256:") && reg.len() == 7 + 64,
        "bad registry digest {reg}"
    );
    assert!(
        surf.starts_with("sha256:") && surf.len() == 7 + 64,
        "bad surface digest {surf}"
    );
}

#[test]
fn surface_digest_recomputed_independently_from_live_response_matches() {
    let caps = live();
    let recomputed = sha256(&surface_of(&caps));
    assert_eq!(recomputed, registry::surface_sha256().to_string());
    assert_eq!(json!(recomputed), caps["surface_sha256"]);
}

#[test]
fn emitted_registry_digests_recompute_and_match_live() {
    let caps = live();
    let mut reg = registry_doc();
    assert_eq!(reg["surface_sha256"], caps["surface_sha256"]);
    assert_eq!(reg["registry_sha256"], caps["registry_sha256"]);
    // The registry document carries op and dialect *objects*; the surface document is
    // their wire names in order, exactly as the live `capabilities` response lists them.
    let names = |key: &str| {
        Value::Array(
            reg[key]
                .as_array()
                .unwrap_or_else(|| panic!("registry `{key}` is an array"))
                .iter()
                .map(|row| row["name"].clone())
                .collect(),
        )
    };
    let surface = json!({
        "abi_version": reg["abi_version"],
        "ops": names("ops"),
        "other_dialects": names("other_dialects"),
        "rdf_dialects": names("rdf_dialects"),
    });
    assert_eq!(
        surface,
        surface_of(&caps),
        "registry names differ from live surface"
    );
    assert_eq!(sha256(&surface), reg["surface_sha256"].as_str().unwrap());
    let claimed = reg
        .as_object_mut()
        .unwrap()
        .remove("registry_sha256")
        .unwrap();
    assert_eq!(
        json!(sha256(&reg)),
        claimed,
        "registry_sha256 is not the digest of the canonical document"
    );
}

#[test]
fn existing_capabilities_fields_are_unchanged() {
    let caps = live();
    assert_eq!(caps["abi"], json!(1));
    assert_eq!(caps["abi_version"], json!(1));
    assert_eq!(caps["crate"], json!(env!("CARGO_PKG_VERSION")));
    let auth: Vec<&str> = caps["authorities"]
        .as_array()
        .expect("authorities array")
        .iter()
        .map(|a| a["capability"].as_str().expect("capability name"))
        .collect();
    let expected: Vec<&str> = graphlaw::BACKEND_AUTHORITIES
        .iter()
        .map(|a| a.capability)
        .collect();
    assert_eq!(auth, expected);
    assert_eq!(
        caps["rdf_dialects"],
        json!([
            "turtle",
            "trig",
            "ntriples",
            "nquads",
            "rdfxml",
            "jsonld",
            "yamlld",
            "trix",
            "hextuples"
        ])
    );
    assert_eq!(
        caps["other_dialects"],
        json!(["n3", "sparql", "shexc", "shexj"])
    );
    assert_eq!(caps["ops"].as_array().unwrap().len(), 14);
}

#[test]
fn negative_mutating_an_op_name_changes_surface_and_registry_digests() {
    let caps = live();
    let base_surface = sha256(&surface_of(&caps));
    assert_eq!(base_surface, registry::surface_sha256().to_string());

    let mut mutated_caps = caps.clone();
    mutated_caps["ops"][2] = json!("parse_mutated");
    let mutated_surface = sha256(&surface_of(&mutated_caps));
    assert_ne!(mutated_surface, base_surface);

    let mut reg = registry_doc();
    reg.as_object_mut().unwrap().remove("registry_sha256");
    let base_reg = sha256(&reg);
    assert_eq!(json!(base_reg), caps["registry_sha256"]);
    reg["ops"][2]["name"] = json!("parse_mutated");
    reg["surface_sha256"] = json!(mutated_surface);
    assert_ne!(sha256(&reg), base_reg);
}

#[test]
fn negative_reordering_ops_changes_surface_digest() {
    let caps = live();
    let base = sha256(&surface_of(&caps));
    let mut swapped = caps.clone();
    let ops = swapped["ops"].as_array_mut().unwrap();
    ops.swap(0, 1);
    assert_ne!(sha256(&surface_of(&swapped)), base);
}
