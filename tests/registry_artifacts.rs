#![cfg(feature = "abi")]
//! G2: the committed registry artifacts (`registry/*`) are the emission of the Rust
//! registry table, are internally consistent (digest, schema rules), and the Turtle
//! projection conforms to `registry/capability-registry.shapes.ttl` under graphlaw's own
//! `shacl` op. Real files, real emitter binary, real engine (no mocks).

use std::path::PathBuf;
use std::process::Command;

use graphlaw::abi::call_json;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const OPS: [&str; 14] = [
    "capabilities",
    "sniff",
    "parse",
    "convert",
    "canonical",
    "sparql",
    "shacl",
    "shex",
    "n3",
    "entail",
    "datalog",
    "hooks",
    "law",
    "policy",
];

const GAC: &str = "http://seanchatmangpt.github.io/packs/graphlaw-ash-capability#";

fn registry_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("registry")
}

fn read(name: &str) -> String {
    let p = registry_dir().join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

fn json_doc() -> Value {
    serde_json::from_str(&read("capability-registry.json")).expect("registry json parses")
}

// ---------- canonical JSON (digest rule, independent of serde_json map ordering) ----------

fn canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(k).unwrap());
                out.push(':');
                canonical(&m[*k], out);
            }
            out.push('}');
        }
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canonical(x, out);
            }
            out.push(']');
        }
        other => out.push_str(&serde_json::to_string(other).unwrap()),
    }
}

fn sha256_of(v: &Value) -> String {
    let mut s = String::new();
    canonical(v, &mut s);
    format!("sha256:{}", hex(&Sha256::digest(s.as_bytes())))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Pretty form required of the committed file: sorted keys, 2-space indent, LF.
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

// ---------- emitter ----------

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

#[test]
fn a_committed_json_and_ttl_equal_emission() {
    assert_eq!(
        read("capability-registry.json"),
        emit("--print-json"),
        "json drift"
    );
    assert_eq!(
        read("capability-registry.ttl"),
        emit("--print-ttl"),
        "ttl drift"
    );
}

#[test]
fn b_emitter_check_flag_accepts_committed_files() {
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
}

// ---------- digest and file format ----------

#[test]
fn c_registry_digest_and_surface_digest_recompute() {
    let doc = json_doc();
    let mut without = doc.clone();
    without.as_object_mut().unwrap().remove("registry_sha256");
    assert_eq!(
        doc["registry_sha256"],
        json!(sha256_of(&without)),
        "registry_sha256"
    );

    let names = |v: &Value| -> Vec<Value> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|x| x["name"].clone())
            .collect()
    };
    let surface = json!({
        "abi_version": doc["abi_version"],
        "ops": names(&doc["ops"]),
        "other_dialects": names(&doc["other_dialects"]),
        "rdf_dialects": names(&doc["rdf_dialects"]),
    });
    assert_eq!(
        doc["surface_sha256"],
        json!(sha256_of(&surface)),
        "surface_sha256"
    );
}

#[test]
fn d_json_file_format_is_canonical_pretty() {
    let text = read("capability-registry.json");
    assert!(text.is_ascii(), "registry json must be ASCII-only");
    assert!(!text.contains('\r'), "registry json must use LF");
    assert!(
        text.ends_with("}\n") && !text.ends_with("\n\n"),
        "single trailing newline"
    );
    let doc: Value = serde_json::from_str(&text).unwrap();
    let mut expect = String::new();
    pretty(&doc, 0, &mut expect);
    expect.push('\n');
    assert_eq!(text, expect, "json is not sorted-key 2-space pretty form");
    assert!(!text.contains("e+") && !text.contains("E+"), "no exponents");
    fn no_floats(v: &Value) {
        match v {
            Value::Number(n) => assert!(n.is_i64() || n.is_u64(), "float in registry: {n}"),
            Value::Array(a) => a.iter().for_each(no_floats),
            Value::Object(m) => m.values().for_each(no_floats),
            _ => {}
        }
    }
    no_floats(&doc);
}

// ---------- schema rules checked without a validator crate ----------

fn schema() -> Value {
    serde_json::from_str(&read("capability-registry.schema.json")).expect("schema parses")
}

fn is_sha(s: &Value) -> bool {
    s.as_str().is_some_and(|s| {
        s.strip_prefix("sha256:").is_some_and(|h| {
            h.len() == 64 && h.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        })
    })
}

fn type_vocab(schema: &Value) -> Vec<String> {
    schema["$defs"]["field_type"]["enum"]
        .as_array()
        .expect("schema field_type enum")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn require_keys(obj: &Value, schema_obj: &Value, ctx: &str) {
    for k in schema_obj["required"]
        .as_array()
        .expect("schema required list")
    {
        let k = k.as_str().unwrap();
        assert!(obj.get(k).is_some(), "{ctx}: missing required key `{k}`");
    }
}

fn check_fields(fields: &Value, schema: &Value, vocab: &[String], ctx: &str) {
    let arr = fields
        .as_array()
        .unwrap_or_else(|| panic!("{ctx}: fields not an array"));
    let mut seen = std::collections::BTreeSet::new();
    for (i, f) in arr.iter().enumerate() {
        require_keys(f, &schema["$defs"]["field"], ctx);
        let name = f["name"].as_str().expect("field name");
        assert!(
            seen.insert(name.to_string()),
            "{ctx}: duplicate field `{name}`"
        );
        assert_eq!(
            f["order"],
            json!(i + 1),
            "{ctx}.{name}: order not contiguous from 1"
        );
        let ty = f["type"].as_str().expect("field type");
        assert!(
            vocab.iter().any(|t| t == ty),
            "{ctx}.{name}: type `{ty}` outside closed vocabulary"
        );
        assert!(
            f["required"].is_boolean() && f["nullable"].is_boolean(),
            "{ctx}.{name}: flags"
        );
        assert!(f["doc"].is_string(), "{ctx}.{name}: doc");
        assert!(
            f["enum"].is_null() || f["enum"].is_array(),
            "{ctx}.{name}: enum"
        );
        let d = &f["default"];
        assert!(
            d.is_null() || d.is_string() || d.is_boolean() || d.is_i64() || d.is_u64(),
            "{ctx}.{name}: default type"
        );
    }
}

#[test]
fn e_schema_file_is_2020_12_and_pins_the_contract() {
    let s = schema();
    assert_eq!(s["$schema"], "https://json-schema.org/draft/2020-12/schema");
    assert_eq!(
        s["properties"]["schema"]["const"],
        "graphlaw.capability-registry/1"
    );
    assert_eq!(s["properties"]["ops"]["minItems"], 14);
    assert_eq!(s["$defs"]["sha256"]["pattern"], "^sha256:[0-9a-f]{64}$");
    assert_eq!(type_vocab(&s).len(), 14, "closed type vocabulary size");
}

#[test]
fn f_registry_json_satisfies_schema_rules() {
    let doc = json_doc();
    let s = schema();
    let vocab = type_vocab(&s);
    require_keys(&doc, &s, "registry");
    assert_eq!(doc["schema"], s["properties"]["schema"]["const"]);
    assert_eq!(doc["abi_version"], 1);
    assert!(
        is_sha(&doc["surface_sha256"]) && is_sha(&doc["registry_sha256"]),
        "sha256 pattern"
    );
    assert_eq!(doc["graphlaw_version"], env!("CARGO_PKG_VERSION"));

    let limits = doc["limits"].as_object().expect("limits object");
    for k in s["properties"]["limits"]["required"].as_array().unwrap() {
        let v = &limits[k.as_str().unwrap()];
        assert!(v.is_u64(), "limit `{k}` must be a non-negative integer");
    }

    let ops = doc["ops"].as_array().unwrap();
    assert!(ops.len() >= 14, "at least 14 ops");
    let names: Vec<&str> = ops.iter().map(|o| o["name"].as_str().unwrap()).collect();
    assert_eq!(&names[..14], &OPS, "first 14 ops in frozen order");
    for (i, op) in ops.iter().enumerate() {
        let n = names[i];
        require_keys(op, &s["$defs"]["op"], n);
        assert_eq!(op["order"], json!(i + 1), "{n}: order");
        assert!(
            op["summary"].as_str().is_some_and(|x| !x.is_empty()),
            "{n}: summary"
        );
        check_fields(
            &op["request"]["fields"],
            &s,
            &vocab,
            &format!("{n}.request"),
        );
        let variants = op["responses"].as_array().unwrap();
        assert!(!variants.is_empty(), "{n}: responses");
        for v in variants {
            require_keys(v, &s["$defs"]["variant"], n);
            let tagged = v["tag"].is_string();
            assert_eq!(
                tagged,
                v["tag_field"].is_string(),
                "{n}: tag/tag_field consistency"
            );
            if variants.len() > 1 {
                assert!(tagged, "{n}: multi-variant ops tag every variant");
            } else {
                assert!(!tagged, "{n}: single-variant ops are untagged");
            }
            check_fields(&v["fields"], &s, &vocab, &format!("{n}.response"));
        }
        for k in ["refusal_kinds", "refusal_codes"] {
            assert!(
                op[k].as_array().unwrap().iter().all(Value::is_string),
                "{n}: {k} strings"
            );
        }
    }
    assert_eq!(
        doc["ops"][5]["responses"].as_array().unwrap().len(),
        3,
        "sparql: 3 variants"
    );

    let all_dialects: Vec<&Value> = doc["rdf_dialects"]
        .as_array()
        .unwrap()
        .iter()
        .chain(doc["other_dialects"].as_array().unwrap())
        .collect();
    for (i, d) in all_dialects.iter().enumerate() {
        require_keys(d, &s["$defs"]["dialect"], "dialect");
        assert_eq!(d["order"], json!(i + 1), "dialect global order");
    }
    for d in doc["rdf_dialects"].as_array().unwrap() {
        assert_eq!(d["kind"], "rdf");
    }
    for d in doc["other_dialects"].as_array().unwrap() {
        assert_eq!(d["kind"], "other");
    }

    for c in doc["refusal_codes"].as_array().unwrap() {
        require_keys(c, &s["$defs"]["refusal_code"], "refusal_code");
        check_fields(&c["fields"], &s, &vocab, "refusal_code");
    }
    for st in doc["law_steps"].as_array().unwrap() {
        require_keys(st, &s["$defs"]["law_step"], "law_step");
        check_fields(&st["fields"], &s, &vocab, "law_step");
    }
}

#[test]
fn g_negative_controls_for_schema_rules_reject_mutations() {
    // Positive control first: the committed document passes the same rule set.
    let s = schema();
    let vocab = type_vocab(&s);
    let doc = json_doc();
    check_fields(&doc["ops"][0]["request"]["fields"], &s, &vocab, "control");
    assert!(is_sha(&doc["registry_sha256"]));

    // A bad digest string and an out-of-vocabulary type are both rejected by the rules.
    assert!(!is_sha(&json!("sha256:ABC")));
    assert!(!is_sha(&json!("md5:00")));
    let bad = json!([{"name":"x","order":1,"type":"quaternion","required":true,
                      "nullable":false,"doc":"d","enum":null,"default":null}]);
    let r = std::panic::catch_unwind(|| {
        let s = schema();
        check_fields(&bad, &s, &type_vocab(&s), "mutant");
    });
    assert!(r.is_err(), "out-of-vocabulary field type must be rejected");
}

// ---------- TTL against SHACL shapes, through graphlaw's own ops ----------

fn shacl_report(ttl: &str) -> Value {
    let resp = call_json(&json!({
        "op": "shacl",
        "data": {"text": ttl, "dialect": "turtle"},
        "shapes": read("capability-registry.shapes.ttl"),
    }));
    assert_eq!(resp["ok"], true, "shacl op refused: {resp}");
    resp
}

fn messages(report: &Value) -> Vec<String> {
    report["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["message"].as_str().unwrap_or_default().to_string())
        .collect()
}

fn has_message(report: &Value, needle: &str) -> bool {
    messages(report).iter().any(|m| m.contains(needle))
}

#[test]
fn h_ttl_parses_with_quads() {
    let resp = call_json(&json!({
        "op": "parse",
        "text": read("capability-registry.ttl"),
        "dialect": "turtle",
    }));
    assert_eq!(resp["ok"], true, "parse refused: {resp}");
    assert!(resp["quads"].as_u64().unwrap() > 0, "quads > 0: {resp}");
}

#[test]
fn i_committed_ttl_conforms_to_shapes() {
    let report = shacl_report(&read("capability-registry.ttl"));
    assert_eq!(
        report["conforms"],
        true,
        "committed ttl must conform; violations: {:#?}",
        messages(&report)
    );
}

fn with_appended(extra: &str) -> String {
    format!(
        "{}\n@prefix gac: <{GAC}> .\n{extra}\n",
        read("capability-registry.ttl")
    )
}

#[test]
fn j_duplicate_op_order_is_non_conforming() {
    // Control: unmodified ttl conforms.
    assert_eq!(
        shacl_report(&read("capability-registry.ttl"))["conforms"],
        true
    );
    let mutant = with_appended(
        "<https://graphlaw.dev/registry#op/zz_dup> a gac:Capability ;\n  \
         gac:capabilityOf <https://graphlaw.dev/registry#registry> ;\n  \
         gac:opName \"zz_dup\" ; gac:opOrder 1 ; gac:opSummary \"duplicate order\" .",
    );
    let report = shacl_report(&mutant);
    assert_eq!(report["conforms"], false);
    assert!(
        has_message(&report, "opOrder must be unique"),
        "{:#?}",
        messages(&report)
    );
}

#[test]
fn j2_model_shapes_reject_malformed_model_rows() {
    let mutant = with_appended(
        "<https://graphlaw.dev/registry#model/ZzBad> a gac:Model ;\n  \
         gac:modelName \"ZzBad\" ; gac:modelOrder 0 .\n\
         <https://graphlaw.dev/registry#model/ZzBad/f> a gac:ModelField ;\n  \
         gac:modelFieldOf <https://graphlaw.dev/registry#model/ZzBad> ; gac:modelFieldName \"f\" .",
    );
    let report = shacl_report(&mutant);
    assert_eq!(report["conforms"], false, "{:#?}", messages(&report));
    assert!(report["results"].as_array().unwrap().len() >= 5);
}

#[test]
fn k_unknown_field_type_is_non_conforming() {
    assert_eq!(
        shacl_report(&read("capability-registry.ttl"))["conforms"],
        true
    );
    let mutant = with_appended(
        "<https://graphlaw.dev/registry#op/capabilities/request/zz> a gac:CapabilityField ;\n  \
         gac:fieldOwner <https://graphlaw.dev/registry#op/capabilities> ;\n  \
         gac:fieldSide \"request\" ; gac:fieldOrder 99 ; gac:fieldName \"zz\" ;\n  \
         gac:fieldType \"quaternion\" ; gac:fieldRequired false ; gac:fieldNullable false ;\n  \
         gac:fieldDoc \"unknown type\" .",
    );
    let report = shacl_report(&mutant);
    assert_eq!(report["conforms"], false);
    let hit = report["results"].as_array().unwrap().iter().any(|r| {
        r["path"]
            .as_str()
            .is_some_and(|p| p.ends_with("fieldType>"))
            && r["component"]
                .as_str()
                .is_some_and(|c| c.ends_with("InConstraintComponent>"))
            && r["value"] == "\"quaternion\""
    });
    assert!(
        hit,
        "expected a fieldType violation: {:#?}",
        report["results"]
    );
}

#[test]
fn l_op_count_mismatch_is_non_conforming() {
    assert_eq!(
        shacl_report(&read("capability-registry.ttl"))["conforms"],
        true
    );
    // A capability that is otherwise valid but not counted by gac:opCount.
    let mutant = with_appended(
        "<https://graphlaw.dev/registry#op/zz_extra> a gac:Capability ;\n  \
         gac:capabilityOf <https://graphlaw.dev/registry#registry> ;\n  \
         gac:opName \"zz_extra\" ; gac:opOrder 9999 ; gac:opSummary \"uncounted\" .",
    );
    let report = shacl_report(&mutant);
    assert_eq!(report["conforms"], false);
    assert!(
        has_message(&report, "opCount must equal"),
        "{:#?}",
        messages(&report)
    );
}

#[test]
fn m_inconsistent_variant_tagging_is_non_conforming() {
    assert_eq!(
        shacl_report(&read("capability-registry.ttl"))["conforms"],
        true
    );
    let mutant = with_appended(
        "<https://graphlaw.dev/registry#op/sniff/response/zz> a gac:ResponseVariant ;\n  \
         gac:variantOf <https://graphlaw.dev/registry#op/sniff> ;\n  \
         gac:variantTag \"kind\" ; gac:variantTagField \"\" ; gac:variantOrder 99 .",
    );
    let report = shacl_report(&mutant);
    assert_eq!(report["conforms"], false);
    assert!(
        has_message(&report, "both be empty or both be non-empty"),
        "{:#?}",
        messages(&report)
    );
}

#[test]
fn artifacts_pin_file_carries_the_live_registry_and_surface_digests() {
    let text = read("ARTIFACTS.sha256");
    let rows: Vec<Vec<&str>> = text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.split_whitespace().collect())
        .collect();
    let find = |kind: &str| {
        rows.iter()
            .find(|r| r[0] == kind)
            .unwrap_or_else(|| panic!("ARTIFACTS.sha256 has no `{kind}` row"))
    };
    assert_eq!(find("registry")[1], graphlaw::registry::registry_sha256());
    assert_eq!(find("surface")[1], graphlaw::registry::surface_sha256());
    let wasm = find("wasm");
    assert_eq!(wasm[1].len(), 64, "wasm digest is 64 hex chars");
    assert!(wasm[1].bytes().all(|b| b.is_ascii_hexdigit()));
    assert!(wasm[2].parse::<u64>().unwrap() > 0);
    // Negative control: a stale registry digest must not equal the live one.
    assert_ne!(
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        graphlaw::registry::registry_sha256()
    );
    // When the pinned build itself is under test, the pin must match its bytes.
    if std::env::var_os("GRAPHLAW_REQUIRE_WASM_PIN").is_some() {
        let path = std::env::var("GRAPHLAW_WASM").expect("GRAPHLAW_WASM set with the pin check");
        let bytes = std::fs::read(path).expect("wasm readable");
        assert_eq!(hex(&Sha256::digest(&bytes)), wasm[1], "wasm pin is stale");
    }
}

#[test]
fn limits_track_source_constants() {
    let doc = json_doc();
    let s = schema();
    let limits = doc["limits"].as_object().expect("limits object");
    let expected: [(&str, usize); 15] = [
        ("max_request_bytes", graphlaw::abi::MAX_REQUEST_BYTES),
        ("max_json_depth", graphlaw::abi::MAX_JSON_DEPTH),
        ("max_plan_actions", graphlaw::abi::MAX_PLAN_ACTIONS),
        ("max_atoms_per_field", graphlaw::abi::MAX_ATOMS_PER_FIELD),
        ("max_policy_entries", graphlaw::abi::MAX_POLICY_ENTRIES),
        ("n3_max_iterations", graphlaw::law::N3_MAX_ITERATIONS),
        ("n3_max_derived_facts", graphlaw::law::N3_MAX_DERIVED_FACTS),
        ("n3_max_total_bytes", graphlaw::law::N3_MAX_TOTAL_BYTES),
        ("n3_max_term_bytes", graphlaw::law::N3_MAX_TERM_BYTES),
        ("n3_max_match_steps", graphlaw::law::N3_MAX_TOTAL_STEPS),
        ("max_plan_total_atoms", graphlaw::plan::MAX_PLAN_TOTAL_ATOMS),
        ("hooks_max_rounds", graphlaw::hooks::MAX_ROUNDS),
        ("hooks_max_firings", graphlaw::hooks::MAX_FIRINGS),
        ("hooks_max_state_quads", graphlaw::hooks::MAX_STATE_QUADS),
        (
            "max_outstanding_alloc_bytes",
            graphlaw::abi::MAX_OUTSTANDING_ALLOC_BYTES,
        ),
    ];
    for (name, value) in expected {
        assert_eq!(
            limits[name],
            json!(value),
            "limit `{name}` tracks its constant"
        );
    }
    assert_eq!(
        limits.len(),
        s["properties"]["limits"]["required"]
            .as_array()
            .unwrap()
            .len(),
        "every registry limit is schema-required and vice versa"
    );
    assert_eq!(limits.len(), expected.len());
}

#[test]
fn limit_meta_keys_equal_limit_keys() {
    let doc = json_doc();
    let limits: Vec<&String> = doc["limits"].as_object().unwrap().keys().collect();
    let meta: Vec<&String> = doc["limit_meta"].as_object().unwrap().keys().collect();
    assert_eq!(limits, meta, "limit_meta covers exactly the limits");
    for (name, m) in doc["limit_meta"].as_object().unwrap() {
        for k in ["scope", "unit", "source"] {
            assert!(
                m[k].as_str().is_some_and(|v| !v.is_empty()),
                "limit_meta.{name}.{k}"
            );
        }
        // refusal_name is present only where the source emits a machine name.
        let unnamed = matches!(
            name.as_str(),
            "hooks_max_rounds" | "hooks_max_firings" | "max_outstanding_alloc_bytes"
        );
        match m.get("refusal_name") {
            None => assert!(unnamed, "limit_meta.{name} lost its refusal_name"),
            Some(v) => {
                assert!(!unnamed, "limit_meta.{name} has no source refusal name");
                assert!(v.as_str().is_some_and(|v| !v.is_empty()), "{name}");
            }
        }
    }
}

#[test]
fn models_cover_the_typed_surface() {
    let doc = json_doc();
    let names: Vec<&str> = doc["models"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "Lease",
            "SignedLease",
            "Receipt",
            "Attestation",
            "Plan",
            "Action",
            "PolicyEntry",
            "PolicyOutcome"
        ]
    );
    let enums: Vec<&str> = doc["model_enums"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        enums,
        [
            "Ceiling",
            "LeaseReason",
            "ReceiptReason",
            "PolicyRefusalKind"
        ]
    );
    // Every model/enum reference resolves.
    for m in doc["models"].as_array().unwrap() {
        assert_eq!(
            m["elixir_module"],
            json!(format!("AshGraphLaw.Model.{}", m["name"].as_str().unwrap()))
        );
        for (i, f) in m["fields"].as_array().unwrap().iter().enumerate() {
            assert_eq!(f["order"], json!(i + 1));
            let ty = f["type"].as_str().unwrap();
            let target = ty
                .strip_prefix("list<model:")
                .and_then(|t| t.strip_suffix('>'))
                .map(|t| ("model", t))
                .or_else(|| ty.strip_prefix("model:").map(|t| ("model", t)))
                .or_else(|| ty.strip_prefix("enum:").map(|t| ("enum", t)));
            if let Some((kind, t)) = target {
                let pool = if kind == "model" { &names } else { &enums };
                assert!(pool.contains(&t), "{ty} unresolved");
            }
        }
    }
    assert_eq!(doc["model_enums"][0]["values"], doc["lease_ceilings"]);
    assert_eq!(doc["model_enums"][3]["values"], doc["policy_refusal_kinds"]);
}

#[test]
fn wasm_outstanding_cap_has_one_source_of_truth() {
    let src =
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wasm/src/lib.rs"))
            .expect("wasm source readable");
    assert!(
        !src.contains("const MAX_OUTSTANDING_BYTES"),
        "wasm/src/lib.rs must use graphlaw::abi::MAX_OUTSTANDING_ALLOC_BYTES"
    );
    assert!(src.contains("graphlaw::abi::MAX_OUTSTANDING_ALLOC_BYTES"));
}
