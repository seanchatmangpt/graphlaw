//! Capability registry: the single Rust source of truth for the public GraphLaw
//! ABI surface (schema `graphlaw.capability-registry/1`).
//!
//! The static tables below describe every ABI op, its request and response
//! fields, dialects, refusal codes, law steps, closed vocabularies and limits.
//! `registry/capability-registry.json` and `registry/capability-registry.ttl`
//! are emitted from these tables by the `graphlaw-registry` binary and are never
//! hand-edited. The `capabilities` ABI op builds its `ops`, `rdf_dialects` and
//! `other_dialects` lists from here, so there is no second literal.
//!
//! Digest rule: `canonical(v)` is compact JSON with object keys sorted by byte
//! order; `surface_blake3` hashes the surface document (ABI version, op names,
//! dialect names) and `registry_blake3` hashes the whole registry document with
//! the `registry_blake3` key removed. Digests are over canonical bytes, never
//! over the pretty file bytes.

mod definitions;
mod emit_json;
mod emit_mcp;
mod emit_ttl;

pub use definitions::*;
pub use emit_json::*;
pub use emit_mcp::*;
pub use emit_ttl::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn surface_matches_contract_shape() {
        assert_eq!(OPS.len(), 14);
        assert_eq!(rdf_dialect_names().len(), 9);
        assert_eq!(other_dialect_names().len(), 5);
        assert_eq!(op_names()[0], "capabilities");
        assert_eq!(op_names()[13], "policy");
        assert!(surface_blake3().starts_with("blake3:"));
        assert_eq!(surface_blake3().len(), 71);
        assert_eq!(registry_blake3().len(), 71);
    }

    #[test]
    fn document_is_ascii_and_float_free() {
        fn walk(v: &Value) {
            match v {
                Value::Number(n) => assert!(!n.is_f64(), "float in registry: {n}"),
                Value::String(s) => assert!(s.is_ascii(), "non-ascii string: {s}"),
                Value::Array(a) => a.iter().for_each(walk),
                Value::Object(m) => {
                    for (k, v) in m {
                        assert!(k.is_ascii());
                        walk(v);
                    }
                }
                _ => {}
            }
        }
        walk(super::emit_json::document());
        assert!(registry_turtle().is_ascii());
    }

    #[test]
    fn registry_digest_recomputes_from_canonical_bytes() {
        let mut doc = registry_value();
        let stated = doc
            .as_object_mut()
            .and_then(|o| o.remove("registry_blake3"))
            .and_then(|v| v.as_str().map(str::to_owned))
            .expect("registry_blake3 present");
        assert_eq!(
            super::emit_json::blake3_of(canonical(&doc).as_bytes()),
            stated
        );
        assert_eq!(stated, registry_blake3());
    }

    #[test]
    fn canonical_sorts_keys_and_escapes_minimally() {
        let v = json!({"b": 1, "a": ["x\"y", "t\tz\u{1}"], "c": {"z": null, "y": true}});
        assert_eq!(
            canonical(&v),
            "{\"a\":[\"x\\\"y\",\"t\\tz\\u0001\"],\"b\":1,\"c\":{\"y\":true,\"z\":null}}"
        );
    }

    #[test]
    fn pretty_json_is_sorted_lf_and_reparses_to_the_document() {
        let pretty = registry_json_pretty();
        assert!(pretty.ends_with("}\n"));
        assert!(!pretty.contains('\r'));
        let reparsed: Value = serde_json::from_str(&pretty).expect("pretty json parses");
        assert_eq!(
            canonical(&reparsed),
            canonical(super::emit_json::document())
        );
        let first_keys: Vec<&str> = pretty
            .lines()
            .filter(|l| l.starts_with("  \"") && !l.starts_with("   "))
            .filter_map(|l| l.trim_start().split('"').nth(1))
            .collect();
        let mut sorted = first_keys.clone();
        sorted.sort_unstable();
        assert_eq!(first_keys, sorted);
    }

    // The src-side dispatch-arm census was removed as the weaker duplicate of
    // tests/registry_dispatch.rs: that court parses the same `fn dispatch`
    // arms from src/abi.rs and asserts (a) no duplicate arms, (b) set equality
    // with the registry op names in BOTH directions — which together imply
    // exactly one arm per registry op — plus (c) live dispatch of every
    // registry op and (d) the unknown-op fallthrough behavior through
    // `call_json`. This module keeps the registry-document shape and digest
    // laws only.

    #[test]
    fn turtle_is_deterministic_and_counts_ops() {
        let a = registry_turtle();
        assert_eq!(a, registry_turtle());
        assert_eq!(a.matches("a gac:Capability ;").count(), OPS.len());
        assert!(a.contains("gac:opCount 14"));
    }
}
