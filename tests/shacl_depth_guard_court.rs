//! Courts for the bounded SHACL shape-nesting guard
//! ([`graphlaw::shacl_guard`], restored `MAX_SHACL_VALIDATION_DEPTH`).
//!
//! Positive control: a shapes graph nesting shapes *deeper* than
//! [`graphlaw::shacl_guard::MAX_SHACL_VALIDATION_DEPTH`] must surface as a
//! typed `ResourceLimit` refusal naming the cap — never as a stack overflow
//! (SIGABRT), and never as a successful validation. Negative control: a
//! legitimate shallow shapes graph must still validate (the guard must not
//! over-refuse). Real shapes graphs, real `LawState::transition` calls, real
//! subprocess isolation — no mocks.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawError, LawState};

fn shacl_prefix() -> &'static str {
    "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix ex: <http://example.org/> .\n"
}

/// `n` shapes linked by `sh:node` (an acyclic chain: s0 -> s1 -> ... -> s(n-1)).
fn node_chain(n: usize) -> String {
    let mut ttl = String::from(shacl_prefix());
    for i in 0..n {
        ttl.push_str(&format!(
            "ex:s{i} a sh:NodeShape ; sh:targetNode ex:x ; sh:property [ sh:path ex:p ] ; sh:node ex:s{} .\n",
            i + 1
        ));
    }
    ttl.push_str(&format!("ex:s{n} a sh:NodeShape .\n"));
    ttl
}

/// `n` shapes nested via `sh:property` chains (blank-node property shapes
/// each carrying the next node shape via `sh:node`), `n` levels deep.
fn property_chain(n: usize) -> String {
    // ex:s0 sh:property [ sh:path ; sh:node [ sh:path ; sh:node [ ... ] ] ]
    let mut ttl = String::from(shacl_prefix());
    ttl.push_str("ex:s0 a sh:NodeShape ; sh:targetNode ex:x ; sh:property [\n");
    for i in 1..n {
        ttl.push_str(&format!("  sh:path ex:p{i} ; sh:node [\n"));
    }
    ttl.push_str("  sh:path ex:ptop\n");
    for _ in 1..n {
        ttl.push_str("] ");
    }
    ttl.push_str("] .\n");
    ttl
}

/// Empty law state (existing-court convention: empty N-Triples parses).
fn empty_state() -> LawState {
    LawState::parse(b"", Dialect::NTriples, None).expect("empty state parses")
}

/// Real validation path: `LawState::transition(Step::AdmitShacl { .. })`.
fn admit(state: &LawState, shapes: &str) -> Result<(), LawError> {
    let step = graphlaw::law::Step::AdmitShacl { shapes_ttl: shapes };
    state.transition(&step).map(|_| ())
}

fn depth_guard_refused(err: &LawError) -> bool {
    // The guard surfaces as a non-conforms SHACL violation naming the
    // exceeded cap (consumer-facing contract: downstream reporters render
    // "SHACL validation failed ... constraint violated ... maximum safe
    // validation depth ...").
    match err {
        LawError::NotAdmitted {
            violations,
            results,
        } => {
            *violations == 1
                && results.len() == 1
                && results[0].component.contains("shacl_validation_depth")
                && results[0]
                    .component
                    .contains("maximum safe validation depth")
                && results[0].message.is_empty()
        }
        _ => false,
    }
}

#[test]
fn depth_guard_refuses_deep_node_chain_with_typed_error() {
    // 200-link chain: well past the purrdf overflow threshold (~500) is not
    // needed to prove the guard — crossing the cap alone must refuse, and
    // 200 > 64 exercises the same code path without building a 1000-shape
    // graph. Deep past the cap, shallow under the panic threshold.
    let shapes = node_chain(graphlaw::shacl_guard::MAX_SHACL_VALIDATION_DEPTH + 100);
    let state = empty_state();
    let err = admit(&state, &shapes).expect_err("deep chain must be refused");
    assert!(
        depth_guard_refused(&err),
        "expected typed ResourceLimit refusal naming shacl_validation_depth, got: {err:?}"
    );
}

#[test]
fn depth_guard_refuses_deep_property_chain_with_typed_error() {
    // cap + 24 = 40 nesting levels: past the guard cap, but under purrdf's
    // 128-level Turtle parser nesting limit — the refusal must come from the
    // depth guard, not the parser (the parser's own EngineRejected bound is
    // a separate, documented backstop).
    let shapes = property_chain(graphlaw::shacl_guard::MAX_SHACL_VALIDATION_DEPTH + 24);
    let state = empty_state();
    let err = admit(&state, &shapes).expect_err("deep property chain must be refused");
    assert!(
        depth_guard_refused(&err),
        "expected typed ResourceLimit refusal naming shacl_validation_depth, got: {err}"
    );
}

#[test]
fn shallow_shacl_still_admits_guard_does_not_over_refuse() {
    // Negative control: a legitimate, shallow shapes graph must validate.
    let shapes = concat!(
        "@prefix sh: <http://www.w3.org/ns/shacl#> .\n",
        "@prefix ex: <http://example.org/> .\n",
        "ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;\n",
        "  sh:property [ sh:path ex:name ; sh:minCount 1 ] .\n",
    );
    let state = empty_state();
    admit(&state, shapes).expect("shallow shapes must admit");
}

#[test]
fn chain_just_under_the_cap_still_admits() {
    // Chain of exactly the cap length must pass; cap + 1 must refuse.
    let under = node_chain(graphlaw::shacl_guard::MAX_SHACL_VALIDATION_DEPTH);
    let state = empty_state();
    admit(&state, &under).expect("chain at the cap must admit");

    let over = node_chain(graphlaw::shacl_guard::MAX_SHACL_VALIDATION_DEPTH + 1);
    let err = admit(&state, &over).expect_err("chain past the cap must refuse");
    assert!(depth_guard_refused(&err));
}

#[test]
fn guard_is_safe_on_a_small_stack_thread() {
    // The guard's own recursion is bounded at cap + 2 frames; prove it on a
    // 1 MiB stack against a graph deep past the cap.
    // 1_024 links stays under MAX_SHACL_SHAPES_BYTES (~118 KiB), so the
    // byte cap cannot fire before the depth cap here (P07 unblock: the
    // chain was 4_096 links ≈ 463 KiB, which the new measured byte cap
    // refuses first — disclosed in the P07-shacl lane receipt).
    let shapes = node_chain(1_024);
    let handle = std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(move || {
            let state = empty_state();
            let err = admit(&state, &shapes).expect_err("deep chain must be refused");
            assert!(depth_guard_refused(&err));
        })
        .expect("spawn");
    handle.join().expect("guard must not overflow the stack");
}

#[test]
fn cyclic_shapes_are_not_refused() {
    // purrdf tolerates shape revisits, so a pure cycle must NOT trip the
    // depth guard (only unbounded acyclic nesting is dangerous).
    let shapes = concat!(
        "@prefix sh: <http://www.w3.org/ns/shacl#> .\n",
        "@prefix ex: <http://example.org/> .\n",
        "ex:a a sh:NodeShape ; sh:node ex:b .\n",
        "ex:b a sh:NodeShape ; sh:node ex:a .\n",
        "ex:c a sh:NodeShape ; sh:targetNode ex:x ; sh:property [ sh:path ex:p ] .\n",
    );
    let state = empty_state();
    admit(&state, shapes).expect("cyclic shapes must not trip the depth guard");
}
