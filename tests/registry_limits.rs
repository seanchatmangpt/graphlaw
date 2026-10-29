//! Limits-registry parity court (feature `abi`, Chicago: real `graphlaw::registry`, no doubles).
//!
//! The `limits` map of `graphlaw.capability-registry/1` must list all 19 engine limits,
//! each read from its real source constant, and each entry in `limit_meta` must carry
//! `scope`, `unit`, `source` and (where one exists) the machine `refusal_name`.
#![cfg(feature = "abi")]

use graphlaw::registry;
use serde_json::{Value, json};

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
    ("n3_max_match_steps", graphlaw::law::N3_MAX_MATCH_STEPS),
    ("n3_max_total_steps", graphlaw::law::N3_MAX_TOTAL_STEPS),
    ("n3_max_backward_depth", graphlaw::law::N3_MAX_BACKWARD_DEPTH),
    (
        "n3_max_backward_solutions_per_goal",
        graphlaw::law::N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL,
    ),
    ("n3_max_term_depth", graphlaw::law::N3_MAX_TERM_DEPTH),
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

    assert_eq!(limits.len(), 19, "registry must surface exactly 19 limits");
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

    assert_eq!(meta.len(), 19);
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

/// F12: purrdf-n3 enforces a per-premise-search matcher step cap
/// (`ReasonerOptions::max_match_steps`, library default 200_000) that is
/// distinct from the run-wide `max_total_steps` cap. The registry's
/// `n3_max_match_steps` row must pin the per-search cap the reasoner
/// actually fires — not the 50M run-wide total (which is `abi_manifest`'s
/// correctly-named `n3_max_total_steps` row).
#[test]
fn n3_max_match_steps_emits_the_cap_the_reasoner_actually_fires() {
    // The wired per-search cap: `reason_n3_bounded` wires `N3_MAX_MATCH_STEPS`
    // into `ReasonerOptions::max_match_steps`; the mirror must keep matching
    // the library default it documents, so a purrdf-n3 default bump surfaces
    // here as a reviewed re-pin instead of silent divergence.
    let wired = graphlaw::n3::ReasonerOptions::default().max_match_steps;
    assert_eq!(
        wired, 200_000,
        "purrdf-n3 per-search default drifted; re-pin N3_MAX_MATCH_STEPS \
         and regenerate the registry artifacts"
    );
    assert_eq!(
        graphlaw::law::N3_MAX_MATCH_STEPS, wired,
        "N3_MAX_MATCH_STEPS mirror diverged from the wired purrdf-n3 default"
    );

    let doc = registry::registry_value();
    let emitted = doc
        .get("limits")
        .and_then(|l| l.get("n3_max_match_steps"))
        .and_then(Value::as_u64)
        .expect("registry limits carry `n3_max_match_steps`");
    assert_eq!(
        emitted, 200_000,
        "registry pins the run-wide total-steps figure (50M) instead of the \
         wired per-search match-steps cap (200k)"
    );
}

// ---------------------------------------------------------------------------
// F11 runtime pairing: every N3 limit identity must answer its TRUE machine
// name and the ceiling that actually fired on a real receipt (`{"op":"n3"}`),
// never the `n3_iterations`/4000 fallback and never the wrong-regime ceiling.
// The registry rows (courts above) pin the constants and their
// `limit_meta.refusal_name` strings; the courts below pin what a fired run
// actually answers, so the receipt pairing cannot drift from the registry
// pairing again.
// ---------------------------------------------------------------------------

/// Run one `{"op":"n3"}` request and require a typed `ResourceLimit` receipt.
fn n3_limit_receipt(text: &str) -> Value {
    let r = graphlaw::abi::call_json(&json!({"op": "n3", "text": text}));
    assert_eq!(
        r["ok"], false,
        "every fixture in this section must refuse with a receipt, got {r}"
    );
    assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
    assert_eq!(r["error"]["details"]["code"], "ResourceLimit", "{r}");
    r
}

/// The nonterminating blank-node rule: each firing mints a fresh node that
/// re-triggers it, so only the fixpoint-iteration cap (4000) can stop it.
#[test]
fn n3_iterations_receipt_carries_its_true_name_and_ceiling() {
    let r = n3_limit_receipt(
        "@prefix : <urn:n:> .\n:a :s :b .\n{ ?x :s ?y } => { ?y :s _:n } .\n",
    );
    let d = &r["error"]["details"];
    assert_eq!(d["limit"], "n3_iterations");
    assert_eq!(d["max"], graphlaw::law::N3_MAX_ITERATIONS);
    assert_eq!(
        d["observed"],
        graphlaw::law::N3_MAX_ITERATIONS,
        "the run-wide fixpoint counter must report the ceiling it breached"
    );
}

/// String doubling: the derived literal crosses the 64 KiB per-term cap after
/// 17 firings, far under every run-wide cap — `n3_term_bytes`/65536. The
/// receipt's `observed` carries the mapping's at-cap sentinel (cap+1), so the
/// court pins it at or above the breached ceiling.
#[test]
fn n3_term_bytes_receipt_carries_its_true_name_and_ceiling() {
    let r = n3_limit_receipt(concat!(
        "@prefix : <urn:n:> .\n",
        "@prefix string: <http://www.w3.org/2000/10/swap/string#> .\n",
        ":a :s \"yy\" .\n",
        "{ :a :s ?x . (?x ?x) string:concatenation ?y } => { :a :s ?y } .\n",
    ));
    let d = &r["error"]["details"];
    assert_eq!(d["limit"], "n3_term_bytes");
    assert_eq!(d["max"], graphlaw::law::N3_MAX_TERM_BYTES);
    let observed = d["observed"]
        .as_u64()
        .expect("receipt observed is a number") as usize;
    assert!(
        observed >= graphlaw::law::N3_MAX_TERM_BYTES,
        "observed {observed} must sit at or above the 64 KiB term cap"
    );
}

/// 800 facts joined pairwise derive 640k distinct facts inside far fewer than
/// 4000 iterations — only the run-wide fact cap can stop it:
/// `n3_derived_facts`/500_000 with `observed` exactly at the cap.
#[test]
fn n3_derived_facts_receipt_carries_its_true_name_and_ceiling() {
    let facts: String = (0..800).map(|i| format!(":n{i} :p :m{i} .\n")).collect();
    let text =
        format!("@prefix : <urn:n:> .\n{facts}{{ ?a :p ?x . ?b :p ?y }} => {{ ?a :q ?y }} .\n");
    let r = n3_limit_receipt(&text);
    let d = &r["error"]["details"];
    assert_eq!(d["limit"], "n3_derived_facts");
    assert_eq!(d["max"], graphlaw::law::N3_MAX_DERIVED_FACTS);
    assert_eq!(
        d["observed"],
        graphlaw::law::N3_MAX_DERIVED_FACTS,
        "the derived-fact counter must report the ceiling it breached"
    );
}

/// A saturating self-join plus one backward rule with a variable-predicate
/// head. That wildcard backward head (`has_wild_backward_head`) keeps every
/// forward rule off the semi-naive agenda, so the join runs as ONE slow-path
/// search whose ~480x480 premise bindings cross the 200k per-search cap
/// (N3_MAX_MATCH_STEPS) while nothing else is in reach: the join's conclusions
/// are all duplicates (no derived facts), the run-wide step sum stays under
/// 1M and the iteration counter stays far under 4000.
/// `ReasonerLimit::MatchSteps` is raised for BOTH ceilings (per-search and
/// run-wide), so the receipt must name the one that actually fired — the
/// per-search 200k, not the run-wide 50M total.
#[test]
fn n3_match_steps_receipt_carries_the_per_search_ceiling_that_fired() {
    let facts: String = (0..480).map(|i| format!(":n{i} :p :m .\n")).collect();
    let text = format!(
        "@prefix : <urn:n:> .\n\
         {facts}\
         {{ ?a :p ?x . ?b :p ?x }} => {{ ?a :p ?x }} .\n\
         {{ ?u ?w ?v }} <= {{ ?u :s ?v }} .\n"
    );
    let r = n3_limit_receipt(&text);
    let d = &r["error"]["details"];
    assert_eq!(d["limit"], "n3_match_steps");
    assert_eq!(
        d["max"], graphlaw::law::N3_MAX_MATCH_STEPS,
        "a per-search breach must report the per-search ceiling, got {r}"
    );
    let observed = d["observed"]
        .as_u64()
        .expect("receipt observed is a number") as usize;
    assert!(
        observed >= graphlaw::law::N3_MAX_MATCH_STEPS,
        "the run-wide step sum consumed at least the fired ceiling"
    );
}
