//! Limits-registry parity court (feature `abi`, Chicago: real `graphlaw::registry`, no doubles).
//!
//! The `limits` map of `graphlaw.capability-registry/1` must list all 15 engine limits,
//! each read from its real source constant, and each entry in `limit_meta` must carry
//! `scope`, `unit`, `source` and (where one exists) the machine `refusal_name`.
#![cfg(feature = "abi")]

use graphlaw::registry;
use serde_json::Value;

/// The 15 limits, each paired with the real constant it is emitted from.
const EXPECTED: &[(&str, usize)] = &[
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

#[test]
fn registry_surfaces_all_15_engine_limits_at_their_source_values() {
    let doc = registry::registry_value();
    let limits = doc
        .get("limits")
        .and_then(Value::as_object)
        .expect("registry document has a `limits` object");

    assert_eq!(limits.len(), 15, "registry must surface exactly 15 limits");
    for (name, value) in EXPECTED {
        let emitted = limits
            .get(*name)
            .and_then(Value::as_u64)
            .unwrap_or_else(|| panic!("missing limit `{name}`"));
        assert_eq!(
            emitted,
            u64::try_from(*value).unwrap(),
            "limit `{name}` drifted from its source constant"
        );
    }

    // The five scope families claimed in the CHANGELOG, each non-empty.
    let scopes: std::collections::BTreeSet<&str> = limits
        .keys()
        .filter_map(|n| {
            doc.get("limit_meta")
                .and_then(|m| m.get(n))
                .and_then(|m| m.get("scope"))
                .and_then(Value::as_str)
        })
        .collect();
    assert_eq!(
        scopes,
        ["abi", "hooks", "n3", "plan", "wasm"].into_iter().collect(),
        "limit scopes must cover abi, n3, plan, hooks and wasm"
    );
}

#[test]
fn limit_meta_carries_scope_unit_source_refusal_for_every_limit() {
    let doc = registry::registry_value();
    let meta = doc
        .get("limit_meta")
        .and_then(Value::as_object)
        .expect("registry document has a `limit_meta` object");

    assert_eq!(meta.len(), 15);
    let limits: Vec<&str> = EXPECTED.iter().map(|(n, _)| *n).collect();
    for name in limits {
        let entry = meta
            .get(name)
            .unwrap_or_else(|| panic!("limit_meta missing `{name}`"));
        for key in ["scope", "unit", "source"] {
            let v = entry
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("limit_meta[{name}].{key} must be a non-empty string"));
            assert!(!v.is_empty(), "limit_meta[{name}].{key} is empty");
        }
        // `source` points at a real repo path of the form src/<file>.rs.
        let source = entry["source"].as_str().unwrap();
        assert!(
            source.starts_with("src/") && source.ends_with(".rs"),
            "limit_meta[{name}].source `{source}` is not a src/ path"
        );
        // refusal_name is present exactly where the source emits a machine name.
        match entry.get("refusal_name") {
            Some(Value::String(r)) => assert!(!r.is_empty()),
            Some(Value::Null) | None => {}
            other => panic!("limit_meta[{name}].refusal_name has unexpected value {other:?}"),
        }
    }

    // Spot-check the documented metadata of two limits against LIMIT_META.
    let n3 = &meta["n3_max_iterations"];
    assert_eq!(n3["scope"], "n3");
    assert_eq!(n3["unit"], "count");
    assert_eq!(n3["source"], "src/law.rs");
    assert_eq!(n3["refusal_name"].as_str(), Some("n3_iterations"));
    let alloc = &meta["max_outstanding_alloc_bytes"];
    assert_eq!(alloc["scope"], "wasm");
    assert_eq!(alloc["refusal_name"], Value::Null);
}
